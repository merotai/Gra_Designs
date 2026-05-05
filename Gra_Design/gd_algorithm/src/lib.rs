
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
    env,
};
use std::fmt::format;
use gd_common::{
    BMI270Samples,
    IMUData,
    SMSType,
};
use tokio::{
    sync::{
        mpsc,
    },
    time::timeout
};
use anyhow::{
    Result,
    anyhow
};


pub enum RockAlert {
    None,

    Info{content: String},
    /// 缓慢倾斜 (滑坡预警)
    TiltWarning {
        current_pitch: f32,
        current_roll: f32,
        since_baseline_hours: f32,
    },
    /// 倾斜报警
    TiltAlarm {
        current_pitch: f32,
        current_roll: f32,
        since_baseline_hours: f32,

    },
    /// 微震频率增加 (岩石崩裂预警)
    MicroseismIncrease {
        rate_per_min: f32,
        threshold: f32,
    },

    /// 方差持续扩大 (裂缝扩展)
    VarianceGrowth {
        current: f32,
        baseline: f32,
        ratio: f32,
    },
    /// 突然沉降 (塌陷预警)
    RapidSettlement {
        magnitude_g: f32,
    },
    /// 设备离线/故障
    SensorAnomaly {
        reason: String,
    }
}

pub struct AlgorParams{
    /// 对应IMU编号
    imu_number: usize,

    /// 长期趋势检测，这里只存加速度幅度
    long_term_window: VecDeque<f32>,
    long_term_size: usize,

    /// 短期震动检测
    short_term_window: VecDeque<BMI270Samples>,
    short_term_size: usize,

    //基准值,开始时初始化为正常状态
    /// pitch
    baseline_pitch: f32,
    /// roll
    baseline_roll: f32,
    /// 初始方差
    baseline_variance: f32,
    /// 校准判断
    calibration_complete: bool,

    // 微震统计
    /// 微震事件计数
    microseism_count: u32,
    /// 每600样本检查一次
    microseism_check_interval: u64,

    /// 连续突变计数
    consecutive_anomalies: u32,
    /// 连续三次才算
    consecutive_anomaly_threshold: u32,


    // 岩体监测阈值
    /// 倾斜警告阈值 5角度
    tilt_warning_deg: f32,
    /// 倾斜报警阈值 10角度
    tilt_alarm_deg: f32,
    /// 微震频率阈值，指偏离重力加速度多少g
    microseism_threshold_g: f32,
    /// 微震频率报警阈值 次/min
    microseism_rate_per_min: f32,
    /// 方差增长倍数报警
    variance_growth_ratio: f32,
    /// 快速沉降阈值
    settlement_threshold_g: f32,

    //统计
    /// 样本计数
    sample_count: u64,
    ///上次的微震复位样本号
    last_microseism_reset: u64,
    /// 上次上报的样本号
    last_log_sample: u64,

}


impl AlgorParams {
    /// 本次采用的10Hz采样频率
    pub fn new(sample_rate_hz: f32) -> AlgorParams {
        assert!(sample_rate_hz > 0.0, "采样必须大于0");
        let short_term_seconds = 10.0;  // 10s
        let long_term_seconds = 3600.0; //1h

        let short_term_size_h = (sample_rate_hz * short_term_seconds) as usize;   // 100样本长度
        let long_term_size_h = (sample_rate_hz * long_term_seconds) as usize;     // 36000样本长度

        let microseism_check_interval_h = (sample_rate_hz * 60.0) as u64;         // 600样本/min

        Self {
            imu_number: 0,
            long_term_window: VecDeque::with_capacity(long_term_size_h),
            short_term_window: VecDeque::with_capacity(short_term_size_h),
            long_term_size: long_term_size_h,
            short_term_size: short_term_size_h,

            baseline_pitch: 0.0,
            baseline_roll: 0.0,
            baseline_variance: 0.0,
            calibration_complete: false,

            microseism_count: 0,
            microseism_check_interval:microseism_check_interval_h,
            consecutive_anomalies: 0,
            consecutive_anomaly_threshold: 3,

            tilt_warning_deg: 5.0,
            tilt_alarm_deg: 10.0,
            microseism_threshold_g: 0.05,
            microseism_rate_per_min: 10.0,
            variance_growth_ratio: 5.0,
            settlement_threshold_g: 0.1,

            sample_count: 0,
            last_microseism_reset: 0,
            last_log_sample: 0,
        }
    }
    // 处理一个样本
    pub fn process_single_imu_data(&mut self, sample: BMI270Samples) -> RockAlert {

        self.sample_count += 1;

        // 更新窗口
        self.short_term_window.push_back(sample.clone());
        if self.short_term_window.len() > self.short_term_size {
            self.short_term_window.pop_front();
        }

        let acc_mag = sample.acc_magnitude();
        self.long_term_window.push_back(acc_mag);
        if self.long_term_window.len() > self.long_term_size {
            self.long_term_window.pop_front();
        }

        // 第一阶段初始化基准,一个小时初始化
        if !self.calibration_complete && self.sample_count >= self.long_term_size as u64 {
            //TODO
            self.calibrate_baseline();
            self.calibration_complete = true;
            let hours = self.sample_count as f32 / 36000.0;

            return RockAlert::Info {
                content: format!(
                    "基准建立完成，({}小时数据) | 初始姿态: pitch = {:.2}°, roll = {:.2}° | 正常微震方差: {:.6})",
                    hours, self.baseline_pitch, self.baseline_roll, self.baseline_variance
                )
            };
        }
        // 校准未完成，只采集数据
        if !self.calibration_complete {
            if self.sample_count - self.last_log_sample >= 36000 {
                self.last_log_sample = self.sample_count;
                let hours = self.sample_count as f32 / 3600.0;
                println!("数据校准中...已采集{:.1}小时的数据", hours);
            }
            return RockAlert::None;
        }

        //-- 检测1，缓慢倾斜
        let (pitch,roll) = self.compute_tilt_angles(&sample);
        let pitch_change = (pitch - self.baseline_pitch).abs();
        let roll_change = (roll - self.baseline_roll).abs();
        let max_tilt = pitch_change.max(roll_change);

        if max_tilt > self.tilt_alarm_deg {
            return RockAlert::TiltAlarm {
                current_pitch: pitch,
                current_roll: roll,
                since_baseline_hours: self.sample_count as f32 / 36000.0,

            };
        }
        if max_tilt > self.tilt_warning_deg {
            return RockAlert::TiltWarning {
                current_pitch: pitch,
                current_roll: roll,
                since_baseline_hours: self.sample_count as f32 / 36000.0,
            }
        }

        //-- 检测2 微震计数，岩爆预警
        if (acc_mag -1.0).abs() > self.microseism_threshold_g {
            self.microseism_count += 1;
        }
        // 每60s检测一次微震频率
        if self.sample_count - self.last_microseism_reset >= self.microseism_check_interval {
            let rate = self.microseism_count as f32;
            if rate > self.microseism_rate_per_min {
                let alert = RockAlert::MicroseismIncrease {
                    rate_per_min: rate,
                    threshold: self.microseism_rate_per_min,
                };
                self.microseism_count = 0;
                self.last_microseism_reset = self.sample_count;
                return alert;
            }
            self.microseism_count = 0;
            self.last_microseism_reset = self.sample_count;
        }
        //-- 检测3 方差增长 裂缝扩展
        // 每10分钟检查一次长期方差
        if self.sample_count % 6000 == 0 && self.long_term_window.len() >= self.long_term_size / 2{
            let current_variance = self.compute_long_term_variance();
            if self.baseline_variance > 0.0 {
                let ratio = current_variance / self.baseline_variance;

                if ratio > self.variance_growth_ratio {
                    return RockAlert::VarianceGrowth {
                        current: current_variance,
                        baseline: self.baseline_variance,
                        ratio: ratio,
                    };
                }
            }
        }

        //-- 检测4 突然沉降塌陷
        let deviation = (acc_mag - 1.0).abs();
        if deviation > self.settlement_threshold_g {
            self.consecutive_anomalies += 1;
            if self.consecutive_anomalies >= self.consecutive_anomaly_threshold {
                self.consecutive_anomalies = 0;
                return RockAlert::RapidSettlement {
                    magnitude_g: deviation,
                };
            }
        } else {
            self.consecutive_anomalies = 0;
        }

        //-- 检测5 传感器异常
        // 加速度全0或长时间不变
        if acc_mag < 0.01 {
            return RockAlert::SensorAnomaly {
                reason: "加速度变化值接近0，传感器可能故障".to_string(),
            };
        }
        if acc_mag > 10.0 {
            return RockAlert::SensorAnomaly {
                reason: format!("加速度变化值异常过大: {:2}g", acc_mag),
            };
        }
        RockAlert::None

    }

    fn calibrate_baseline(&mut self) {
        let n = self.short_term_window.len() as f32;
        if n == 0.0 {
            return;
        }

        let avg_ax = self.short_term_window.iter()
                            .map(|s| {s.ax_g}).sum::<f32>() / n;
        let avg_ay = self.short_term_window.iter()
                            .map(|s| {s.ay_g}).sum::<f32>() / n;
        let avg_az = self.short_term_window.iter()
                            .map(|s| {s.az_g}).sum::<f32>() / n;

        let avg_sample = BMI270Samples {
            ax_g: avg_ax,
            ay_g: avg_ay,
            az_g: avg_az,
            gx_dps: 0.0,
            gy_dps: 0.0,
            gz_dps: 0.0,
        };
        let (pitch,roll) = self.compute_tilt_angles(&avg_sample);
        self.baseline_pitch = pitch;
        self.baseline_roll = roll;

        //计算正常微震水平
        self.baseline_variance = self.compute_long_term_variance();

    }


    /// 计算倾斜角度（pitch 和 roll）
    fn compute_tilt_angles(&self,sample: &BMI270Samples) -> (f32,f32) {
        let pitch = sample.ax_g.atan2(
            (sample.ay_g.powi(2) + sample.az_g.powi(2)).sqrt()
        ).to_degrees();
        let roll = sample.ay_g.atan2(
            (sample.ax_g.powi(2) + sample.az_g.powi(2)).sqrt()
        ).to_degrees();

        (pitch, roll)
    }
    /// 计算长期窗口偏差 只算加速度幅度值（变化值）
    fn compute_long_term_variance(&self) ->f32 {
        let n = self.long_term_window.len() as f32;

        if n < 2.0 {
            return 0.0;
        }
        let mean = self.long_term_window.iter().sum::<f32>() / n;
        self.long_term_window.iter()
            .map(|&m| {(m - mean).powi(2)})
            .sum::<f32>() / n

    }
    /// 获取当前状态摘要
    pub fn status(&self) -> String {
        let hours = self.sample_count as f32 / 36000.0;

        format!(
            "运行{:.1}h | 样本{} | 校准{} | 姿态(pitch = {:.2}°,roll = {:.2}° ) | 当前微震计数: {}",
            hours, self.sample_count, self.calibration_complete,
            self.baseline_pitch, self.baseline_roll, self.microseism_count
        )
    }
}


/// 算法分析任务
pub async fn run_algor_analyse_task(mut rx: mpsc::Receiver<IMUData>, tx: mpsc::Sender<SMSType>) -> Result<()> {
    let mut sensor_imu_1 = AlgorParams::new(10.0);
    let mut sensor_imu_2 = AlgorParams::new(10.0);
    let mut sensor_imu_3 = AlgorParams::new(10.0);
    println!("[algor_task]算法分析任务启动");
    loop {
        match timeout(Duration::from_secs(20),rx.recv()).await {
            Ok(Some(data_h)) => {
                match data_h.imu_num {
                    1 => {
                        // 1号传感器
                        for sample in data_h.imu_dataset.into_iter() {
                            match sensor_imu_1.process_single_imu_data(sample) {
                                RockAlert::None => continue,
                                RockAlert::Info {content} => {
                                    println!("消息如下: {}", content);
                                },
                                RockAlert::TiltWarning {..} => {
                                    match tx.try_send(SMSType::TiltWarnSensor(1)) {
                                        Ok(_) => {
                                            println!("1号，发送倾斜预警");
                                        },
                                        Err(_) => {
                                            eprintln!("1号，发送倾斜预警失败");
                                        }
                                    }
                                },
                                RockAlert::TiltAlarm {..} => {
                                    match tx.try_send(SMSType::TiltAlarmSensor(1)) {
                                        Ok(_) => {
                                            println!("1号，发送倾斜报警");
                                        },
                                        Err(_) => {
                                            eprintln!("1号，发送倾斜报警失败");
                                        }
                                    }
                                },
                                RockAlert::MicroseismIncrease {rate_per_min,..} => {
                                    match tx.try_send(SMSType::MicroseismSensor(1)) {
                                        Ok(_) => {
                                            println!("1号，发送微震报警");
                                        },
                                        Err(_) => {
                                            eprintln!("1号，发送微震报警失败");
                                        }
                                    }
                                },
                                RockAlert::VarianceGrowth {..} => {
                                    match tx.try_send(SMSType::VarianceGrowSensor(1)) {
                                        Ok(_) => {
                                            println!("1号，发送裂隙扩展报警");
                                        },
                                        Err(_) => {
                                            eprintln!("1号，发送裂隙扩展报警失败");
                                        }
                                    }
                                },
                                RockAlert::RapidSettlement {..} => {
                                    match tx.try_send(SMSType::RapidSettleSensor(1)) {
                                        Ok(_) => {
                                            println!("1号，发送沉降报警");
                                        },
                                        Err(_) => {
                                            eprintln!("1号，发送沉降报警失败");
                                        }
                                    }
                                },
                                RockAlert::SensorAnomaly {reason} => {
                                    println!("传感器异常: {}", reason);
                                    match tx.try_send(SMSType::WrongSensor(1)) {
                                        Ok(_) => {
                                            println!("1号，发送传感器异常报警");
                                        },
                                        Err(_) => {
                                            println!("1号，发送传感器异常报警失败");
                                        }
                                    }
                                }
                            }
                        }
                    },
                    2 => {
                        // 2号传感器
                        for sample in data_h.imu_dataset.into_iter() {
                            match sensor_imu_2.process_single_imu_data(sample) {
                                RockAlert::None => continue,
                                RockAlert::Info {content} => {
                                    println!("消息如下: {}", content);
                                },
                                RockAlert::TiltWarning {..} => {
                                    match tx.try_send(SMSType::TiltWarnSensor(2)) {
                                        Ok(_) => {
                                            println!("2号，发送倾斜预警");
                                        },
                                        Err(_) => {
                                            eprintln!("2号，发送倾斜预警失败");
                                        }
                                    }
                                },
                                RockAlert::TiltAlarm {..} => {
                                    match tx.try_send(SMSType::TiltAlarmSensor(2)) {
                                        Ok(_) => {
                                            println!("2号，发送倾斜报警");
                                        },
                                        Err(_) => {
                                            eprintln!("2号，发送倾斜报警失败");
                                        }
                                    }
                                },
                                RockAlert::MicroseismIncrease {rate_per_min,..} => {
                                    match tx.try_send(SMSType::MicroseismSensor(2)) {
                                        Ok(_) => {
                                            println!("2号，发送微震报警");
                                        },
                                        Err(_) => {
                                            eprintln!("2号，发送微震报警失败");
                                        }
                                    }
                                },
                                RockAlert::VarianceGrowth {..} => {
                                    match tx.try_send(SMSType::VarianceGrowSensor(2)) {
                                        Ok(_) => {
                                            println!("2号，发送裂隙扩展报警");
                                        },
                                        Err(_) => {
                                            eprintln!("2号，发送裂隙扩展报警失败");
                                        }
                                    }
                                },
                                RockAlert::RapidSettlement {..} => {
                                    match tx.try_send(SMSType::RapidSettleSensor(2)) {
                                        Ok(_) => {
                                            println!("2号，发送沉降报警");
                                        },
                                        Err(_) => {
                                            eprintln!("2号，发送沉降报警失败");
                                        }
                                    }
                                },
                                RockAlert::SensorAnomaly {reason} => {
                                    println!("传感器异常: {}", reason);
                                    match tx.try_send(SMSType::WrongSensor(2)) {
                                        Ok(_) => {
                                            println!("2号，发送传感器异常报警");
                                        },
                                        Err(_) => {
                                            println!("2号，发送传感器异常报警失败");
                                        }
                                    }
                                }
                            }
                        }
                    },
                    3 => {
                        // 3号传感器
                        for sample in data_h.imu_dataset.into_iter() {
                            match sensor_imu_3.process_single_imu_data(sample) {
                                RockAlert::None => continue,
                                RockAlert::Info {content} => {
                                    println!("消息如下: {}", content);
                                },
                                RockAlert::TiltWarning {..} => {
                                    match tx.try_send(SMSType::TiltWarnSensor(3)) {
                                        Ok(_) => {
                                            println!("3号，发送倾斜预警");
                                        },
                                        Err(_) => {
                                            eprintln!("3号，发送倾斜预警失败");
                                        }
                                    }
                                },
                                RockAlert::TiltAlarm {..} => {
                                    match tx.try_send(SMSType::TiltAlarmSensor(3)) {
                                        Ok(_) => {
                                            println!("3号，发送倾斜报警");
                                        },
                                        Err(_) => {
                                            eprintln!("3号，发送倾斜报警失败");
                                        }
                                    }
                                },
                                RockAlert::MicroseismIncrease {rate_per_min,..} => {
                                    match tx.try_send(SMSType::MicroseismSensor(3)) {
                                        Ok(_) => {
                                            println!("3号，发送微震报警");
                                        },
                                        Err(_) => {
                                            eprintln!("3号，发送微震报警失败");
                                        }
                                    }
                                },
                                RockAlert::VarianceGrowth {..} => {
                                    match tx.try_send(SMSType::VarianceGrowSensor(3)) {
                                        Ok(_) => {
                                            println!("3号，发送裂隙扩展报警");
                                        },
                                        Err(_) => {
                                            eprintln!("3号，发送裂隙扩展报警失败");
                                        }
                                    }
                                },
                                RockAlert::RapidSettlement {..} => {
                                    match tx.try_send(SMSType::RapidSettleSensor(3)) {
                                        Ok(_) => {
                                            println!("3号，发送沉降报警");
                                        },
                                        Err(_) => {
                                            eprintln!("3号，发送沉降报警失败");
                                        }
                                    }
                                },
                                RockAlert::SensorAnomaly {reason} => {
                                    println!("传感器异常: {}", reason);
                                    match tx.try_send(SMSType::WrongSensor(3)) {
                                        Ok(_) => {
                                            println!("3号，发送传感器异常报警");
                                        },
                                        Err(_) => {
                                            println!("3号，发送传感器异常报警失败");
                                        }
                                    }
                                }
                            }
                        }
                    },

                    _ => {
                        // 未知编号传感器
                        println!("未知的传感器编号");
                    },
                }
            },
            Ok(None) => {
                // IMU通道关闭
                break
            },
            Err(_) => {
                eprintln!("从IMUData通道获取数据失败！");
            }
        }
    }
    Ok(())
}

use lettre::{
    Message,
    transport::smtp::authentication::Credentials,
    SmtpTransport,
    Transport,
};
use dotenv::dotenv;

/// 报警信息发送任务
pub async fn run_sms_send_task(mut rx: mpsc::Receiver<SMSType>) -> Result<()> {
    // dotenv().ok();
    // let mail_author_code = env::var("MAIL_AUTHORIZE_CODE")?;
    // let from_email = env::var("FROM_EMAIL")?;
    // let to_email = env::var("TO_EMAIL")?;

    let mail_author_code = "soyikaigzoewbfbc";
    let from_email = "592778939@qq.com";
    let to_email = "3870180466@qq.com";


    // let email = Message::builder()
    //     .from("592778939@qq.com".parse()?)
    //     .to("3870180466@qq.com".parse()?)
    //     .subject("来自服务端的警告信息");

    let credits = Credentials::new(from_email.to_string(),mail_author_code.to_string());

    let mailer = SmtpTransport::starttls_relay("smtp.qq.com")?
        .credentials(credits)
        .build();
    println!("[send_task]报警推送任务启动");
    loop {
        match timeout(Duration::from_secs(20),rx.recv()).await {
            Ok(Some(sms_type)) => {
                match sms_type {
                    SMSType::TiltWarnSensor(num) => {
                        let email = match Message::builder()
                            .from(from_email.parse()?)
                            .to(to_email.parse()?)
                            .subject("来自服务端的警告信息")
                            .body(format!("-[警告]- {}号传感器的岩体倾斜预警",num))
                        {
                            Ok(email) => email,
                            Err(e) => {
                                eprintln!("邮件构建失败");
                                return Err(anyhow!(e))
                            }
                        };
                        match mailer.send(&email) {
                            Ok(_) => {
                                println!("邮件发送成功");
                            },
                            Err(e) => {
                                eprintln!("邮件发送失败，请检查配置内容")
                            }
                        }
                    }
                    SMSType::TiltAlarmSensor(num) => {
                        let email = match Message::builder()
                            .from(from_email.parse()?)
                            .to(to_email.parse()?)
                            .subject("来自服务端的警告信息")
                            .body(format!("-[危险]- {}号传感器的岩体倾斜报警",num))
                        {
                            Ok(email) => email,
                            Err(e) => {
                                eprintln!("邮件构建失败");
                                return Err(anyhow!(e))
                            }
                        };
                        match mailer.send(&email) {
                            Ok(_) => {
                                println!("邮件发送成功");
                            },
                            Err(e) => {
                                eprintln!("邮件发送失败，请检查配置内容")
                            }
                        }
                    },
                    SMSType::MicroseismSensor(num) => {
                        let email = match Message::builder()
                            .from(from_email.parse()?)
                            .to(to_email.parse()?)
                            .subject("来自服务端的警告信息")
                            .body(format!("-[危险]- {}号传感器的岩体微震报警",num))
                        {
                            Ok(email) => email,
                            Err(e) => {
                                eprintln!("邮件构建失败");
                                return Err(anyhow!(e))
                            }
                        };
                        match mailer.send(&email) {
                            Ok(_) => {
                                println!("邮件发送成功");
                            },
                            Err(e) => {
                                eprintln!("邮件发送失败，请检查配置内容")
                            }
                        }
                    },
                    SMSType::VarianceGrowSensor(num) => {
                        let email = match Message::builder()
                            .from(from_email.parse()?)
                            .to(to_email.parse()?)
                            .subject("来自服务端的警告信息")
                            .body(format!("-[危险]- {}号传感器的岩体裂缝扩展报警",num))
                        {
                            Ok(email) => email,
                            Err(e) => {
                                eprintln!("邮件构建失败");
                                return Err(anyhow!(e))
                            }
                        };
                        match mailer.send(&email) {
                            Ok(_) => {
                                println!("邮件发送成功");
                            },
                            Err(e) => {
                                eprintln!("邮件发送失败，请检查配置内容")
                            }
                        }
                    },
                    SMSType::RapidSettleSensor(num) => {
                        let email = match Message::builder()
                            .from(from_email.parse()?)
                            .to(to_email.parse()?)
                            .subject("来自服务端的警告信息")
                            .body(format!("-[危险]- {}号传感器的岩体沉降报警",num))
                        {
                            Ok(email) => email,
                            Err(e) => {
                                eprintln!("邮件构建失败");
                                return Err(anyhow!(e))
                            }
                        };
                        match mailer.send(&email) {
                            Ok(_) => {
                                println!("邮件发送成功");
                            },
                            Err(e) => {
                                eprintln!("邮件发送失败，请检查配置内容")
                            }
                        }
                    },
                    SMSType::WrongSensor(num) => {
                        let email = match Message::builder()
                            .from(from_email.parse()?)
                            .to(to_email.parse()?)
                            .subject("来自服务端的警告信息")
                            .body(format!("-[警告]- {}号传感器的岩体传感器异常报警",num))
                        {
                            Ok(email) => email,
                            Err(e) => {
                                eprintln!("邮件构建失败");
                                return Err(anyhow!(e))
                            }
                        };
                        match mailer.send(&email) {
                            Ok(_) => {
                                println!("邮件发送成功");
                            },
                            Err(e) => {
                                eprintln!("邮件发送失败，请检查配置内容")
                            }
                        }
                    },

                }
            },
            Ok(None) => {
                // 传输通道关闭
                break;
            },
            Err(e) => {
                eprintln!("[send_task] 任务在20s内未收到sms发送请求，继续等待");
                continue;
            }
        }
    }
    Ok(())
}

















