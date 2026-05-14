
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
    env,
    sync::Arc,

};
use std::fmt::format;
use gd_common::{
    BMI270Samples,
    IMUData,
    SMSType,
    SMSCooldownPeriod,



    data_base::{
        db_mongo::{
            MongoDBClient,
        },
        db_mongo_models::{
            AlarmInfo,
            InFo,
        }
    },

    func::{
        algor_web_share_state::{
            SharedState,
            AlarmState
        },

        print_color::{
            GREEN,
            BLUE,
            YELLOW,
            RED,
        },
    }
};
use tokio::{
    sync::{
        mpsc,
        RwLock,
    },
    time::timeout
};

use chrono::{ Utc};

use anyhow::{
    Result,
    anyhow
};

#[derive(Debug)]
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

    ///
    sms_state: SMSCooldownPeriod,

}


impl AlgorParams {
    /// 本次采用的10Hz采样频率
    pub fn new(sample_rate_hz: f32,imu_num: usize) -> AlgorParams {
        assert!(sample_rate_hz > 0.0, "采样必须大于0");
        let short_term_seconds = 10.0;  // 10s
        let long_term_seconds = 300.0; //5min

        let short_term_size_h = (sample_rate_hz * short_term_seconds) as usize;   // 100样本长度
        let long_term_size_h = (sample_rate_hz * long_term_seconds) as usize;     // 3000样本长度

        let microseism_check_interval_h = (sample_rate_hz * 60.0) as u64;         // 600样本/min

        Self {
            imu_number: imu_num,
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

            sms_state: SMSCooldownPeriod::new(10),
        }
    }
    // 处理一个样本
    pub fn process_single_imu_data(&mut self, sample: BMI270Samples,imu_number: u8) -> RockAlert {

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

        // 第一阶段初始化基准,5min的初始化
        if !self.calibration_complete && self.sample_count >= self.long_term_size as u64 {
            //TODO
            self.calibrate_baseline();
            self.calibration_complete = true;
            let hours = self.sample_count as f32 / 36000.0;
            println!("{} [algor_task] {}号传感器基准建立完成，({}小时数据) | 初始姿态: pitch = {:.2}°, roll = {:.2}° | 正常微震方差: {:.6})",
                   BLUE, imu_number, hours, self.baseline_pitch, self.baseline_roll, self.baseline_variance);

            return RockAlert::Info {
                content: format!(
                    "基准建立完成，({}小时数据) | 初始姿态: pitch = {:.2}°, roll = {:.2}° | 正常微震方差: {:.6})",
                    hours, self.baseline_pitch, self.baseline_roll, self.baseline_variance
                )
            };
        }
        // 校准未完成，只采集数据
        if !self.calibration_complete {
            if self.sample_count - self.last_log_sample >= 3000 {
                self.last_log_sample = self.sample_count;
                let hours = self.sample_count as f32 / 36000.0;
                println!(" {} [algor_task] 数据校准中...已采集{:.1}小时的数据",GREEN, hours);
            }
            return RockAlert::None;
        }
        //------------------------------------------------------------------------------------------
        //--------------------------------- 检测1，缓慢倾斜 -------------------------------------------
        //------------------------------------------------------------------------------------------
        let (pitch,roll) = self.compute_tilt_angles(&sample);
        let pitch_change = (pitch - self.baseline_pitch).abs();
        let roll_change = (roll - self.baseline_roll).abs();
        let max_tilt = pitch_change.max(roll_change);

        if max_tilt > self.tilt_alarm_deg {
            // 检测倾斜报警冷却期条件
            if self.sample_count >= self.sms_state.next_tilt_alarm_sample{
                self.sms_state.next_tilt_alarm_sample = self.sample_count + self.sms_state.tilt_alarm_cooldown_period;
                return RockAlert::TiltAlarm {
                    current_pitch: pitch,
                    current_roll: roll,
                    since_baseline_hours: self.sample_count as f32 / 36000.0,

                };
            } else {
                println!("{} [algor_task]倾斜报警冷却中",YELLOW);
            }

        }
        if max_tilt > self.tilt_warning_deg {
            // 检测倾斜预警冷却期条件
            if self.sample_count >= self.sms_state.next_tilt_warning_sample {
                self.sms_state.next_tilt_warning_sample = self.sample_count + self.sms_state.tilt_warning_cooldown_period;
                return RockAlert::TiltWarning {
                    current_pitch: pitch,
                    current_roll: roll,
                    since_baseline_hours: self.sample_count as f32 / 36000.0,
                }
            } else {
                println!("{} [algor_task]倾斜预警冷却中",YELLOW);
            }

        }
        //------------------------------------------------------------------------------------------
        //--------------------------------- 检测2 微震计数，岩爆预警 -----------------------------------
        //------------------------------------------------------------------------------------------
        if (acc_mag - 1.0).abs() > self.microseism_threshold_g {
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
        //------------------------------------------------------------------------------------------
        //-------------------------------- 检测3 方差增长 裂缝扩展 ------------------------------------
        //------------------------------------------------------------------------------------------
        // 每8分钟检查一次长期方差
        if self.sample_count % 4800 == 0 && self.long_term_window.len() >= self.long_term_size / 2{
            let current_variance = self.compute_long_term_variance();
            if self.baseline_variance > 0.0 {
                let ratio_l = current_variance / self.baseline_variance;

                if ratio_l > self.variance_growth_ratio {
                    return RockAlert::VarianceGrowth {
                        current: current_variance,
                        baseline: self.baseline_variance,
                        ratio: ratio_l,
                    };
                }
            }
        }
        //------------------------------------------------------------------------------------------
        //--------------------------------- 检测4 突然沉降塌陷 ----------------------------------------
        //------------------------------------------------------------------------------------------
        let deviation = (acc_mag - 1.0).abs();
        if deviation > self.settlement_threshold_g {
            self.consecutive_anomalies += 1;
            if self.consecutive_anomalies >= self.consecutive_anomaly_threshold {

                if self.sms_state.next_settlement_alarm_sample <= self.sample_count {
                    self.sms_state.next_settlement_alarm_sample = self.sample_count + self.sms_state.settlement_cooldown_period;
                    self.consecutive_anomalies = 0;
                    return RockAlert::RapidSettlement {
                        magnitude_g: deviation,
                    };
                } else {
                    println!("{} [algor_task]突然沉降报警冷却中",YELLOW);
                }

            }
        } else {
            self.consecutive_anomalies = 0;
        }
        //------------------------------------------------------------------------------------------
        //---------------------------------- 检测5 传感器异常 -----------------------------------------
        //------------------------------------------------------------------------------------------
        // 加速度全0或长时间不变
        if acc_mag < 0.01 {
            if self.sample_count >= self.sms_state.next_tilt_warning_sample {
                self.sms_state.next_sensor_anomaly_sample = self.sample_count + self.sms_state.settlement_cooldown_period;
                return RockAlert::SensorAnomaly {
                    reason: "加速度变化值接近0，传感器可能故障".to_string(),
                };
            }

        }
        if acc_mag > 10.0 {
            if self.sample_count >= self.sms_state.next_tilt_warning_sample {
                self.sms_state.next_sensor_anomaly_sample = self.sample_count + self.sms_state.settlement_cooldown_period;
                return RockAlert::SensorAnomaly {
                    reason: format!("加速度变化值异常过大: {:2}g", acc_mag),
                };
            }
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

    /// 获取当前窗口的角度值
    pub fn get_current_tilt_angles(&self) -> (f32,f32) {
        if let Some(latest) = self.short_term_window.back() {
            let pitch = latest.ax_g.atan2(
                (latest.ay_g.powi(2) + latest.az_g.powi(2)).sqrt()
            ).to_degrees();
            let roll = latest.ay_g.atan2(
                (latest.ax_g.powi(2) + latest.az_g.powi(2)).sqrt()
            ).to_degrees();
            (pitch, roll)
        } else {
            (0.0, 0.0)
        }
    }

} // impl

/// 处理 RockAlert类型，并发送到SMSType数据通道里面
fn handle_sensor_alert(
    sensor_id: usize,
    alert_type: RockAlert,
    tx: &mpsc::Sender<SMSType>
) {
    match alert_type {
        RockAlert::None => {},
        RockAlert::Info {content} => {
            println!("消息如下: {}", content);
        },
        RockAlert::TiltWarning {current_pitch,current_roll,since_baseline_hours} => {
            match tx.try_send(SMSType::TiltWarnSensor(
               sensor_id, current_pitch, current_roll, since_baseline_hours
            )) {
                Ok(_) => {
                    println!("{}号，发送倾斜预警",sensor_id);
                },
                Err(_) => {
                    eprintln!("{}号，发送倾斜预警失败",sensor_id);
                }
            }
        },
        RockAlert::TiltAlarm {current_pitch, current_roll, since_baseline_hours} => {
            match tx.try_send(SMSType::TiltAlarmSensor(
                sensor_id, current_pitch, current_roll, since_baseline_hours
            )) {
                Ok(_) => {
                    println!("{}号，发送倾斜报警",sensor_id)
                },
                Err(_) => {
                    eprintln!("{}号，发送倾斜报警失败",sensor_id);
                }
            }
        },
        RockAlert::MicroseismIncrease { rate_per_min, threshold } => {
            match tx.try_send(SMSType::MicroseismSensor(sensor_id, rate_per_min, threshold)) {
                Ok(_) => {
                    println!("{}号，发送微震报警", sensor_id)
                },
                Err(_) => {
                    eprintln!("{}号，发送微震报警失败", sensor_id)
                },
            }
        },
        RockAlert::VarianceGrowth { current, baseline, ratio } => {
            match tx.try_send(SMSType::VarianceGrowSensor(sensor_id, current, baseline, ratio)) {
                Ok(_) => {
                    println!("{}号，发送裂隙扩展报警", sensor_id)
                },
                Err(_) => {
                    eprintln!("{}号，发送裂隙扩展报警失败", sensor_id)
                },
            }
        },
        RockAlert::RapidSettlement { magnitude_g } => {
            match tx.try_send(SMSType::RapidSettleSensor(sensor_id, magnitude_g)) {
                Ok(_) => {
                    println!("{}号，发送沉降报警", sensor_id)
                },
                Err(_) => {
                    eprintln!("{}号，发送沉降报警失败", sensor_id)
                },
            }
        },

        RockAlert::SensorAnomaly { reason } => {
            println!("传感器异常: {}", reason);
            match tx.try_send(SMSType::WrongSensor(sensor_id, reason)) {
                Ok(_) => {
                    println!("{}号，发送传感器异常报警", sensor_id)
                },
                Err(_) => {
                    eprintln!("{}号，发送传感器异常报警失败", sensor_id)
                },
            }
        }
    }
}




/// 算法分析任务
pub async fn run_algor_analyse_task(mut rx: mpsc::Receiver<IMUData>, tx: mpsc::Sender<SMSType>,shared_state: SharedState) -> Result<()> {

    let mut sensors: Vec<AlgorParams> = (1..=3).map(|id| {AlgorParams::new(10.0,id)}).collect();
    println!("{} [algor_task]算法分析任务启动",BLUE);
    loop {
        // 等待20s，没有数据则退出任务，结束整个服务端
        match timeout(Duration::from_secs(20), rx.recv()).await {
            Ok(Some(data_h)) => {
                let imu_num = data_h.imu_num as usize;
                // 不是目标编号IMU,直接跳过看下一个
                if imu_num < 1 || imu_num > sensors.len() {
                    continue;
                }
                let sensor = &mut sensors[imu_num - 1];

                for sample in data_h.imu_dataset.into_iter() {
                    //
                    let alert = sensor.process_single_imu_data(sample,imu_num as u8);
                    let (pitch_h,roll_h) = sensor.get_current_tilt_angles();

                    // 采用局部作用域,操作作用域结束后就会释放相应的内存
                    // 采用共享变量获取实时数据，同步内容到web后端里面
                    // Arc<RwLock> 类型申请操作锁后进行修改就会同步到主进程当中
                    {
                        let mut state = shared_state.write().await;
                        if let Some(s) = state.sensors.get_mut(&imu_num) {
                            s.pitch = pitch_h;
                            s.roll = roll_h;
                            s.calibration_complete = sensor.calibration_complete;
                            s.sample_count = sensor.sample_count;
                            s.last_update = Utc::now()
                                                .format("%%Y-%m-%d %H:%M:%S")
                                                .to_string();
                            match &alert {
                                RockAlert::TiltAlarm {..} => {
                                    s.tilt_state = "alarm".to_string();
                                },
                                RockAlert::TiltWarning {..} => {
                                    s.tilt_state = "warning".to_string();
                                },
                                RockAlert::None => {
                                    if s.tilt_state != "alarm" || s.tilt_state != "warning" {
                                        s.tilt_state = if sensor.calibration_complete {
                                            "normal".to_string()
                                        } else {
                                            "calibrating".to_string()
                                        }
                                    }
                                },
                                _ => {
                                    // 这里忽略其他类型的RockAlert
                                }
                            }// match
                        }
                        // 填充alarm字段的内容
                        if !matches!(alert, RockAlert::None | RockAlert::Info {..} ) {
                            state.alarms.push(AlarmState {
                                timestamp: Utc::now()
                                                .format("%Y-%m-%d %H:%M:%S")
                                                .to_string(),
                                imu_number: imu_num ,
                                alarm_type: format!("{:?}",alert),
                                detail: format!("pitch = {:.2}, roll = {:.2}",pitch_h,roll_h),

                            });
                            if state.alarms.len() > 100 {
                                state.alarms.remove(0);
                            }
                        }

                    }// 局部作用域

                    // 发送RockAlert到数据通道
                    handle_sensor_alert(imu_num,alert,&tx);
                 }
            },
            Ok(None) => {
                break;
            },
            Err(_) => {
                eprintln!("{} [algor_task] 从IMUData通道获取数据失败！", RED);
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

use mongodb::{
    bson::{
        doc,
        oid::ObjectId,
        DateTime
    },

};
use tokio::sync::mpsc::error::TrySendError;

/// 报警信息发送任务
pub async fn run_sms_send_task(mut rx: mpsc::Receiver<SMSType>,db_client: MongoDBClient) -> Result<()> {
    // dotenv().ok();
    // let mail_author_code = env::var("MAIL_AUTHORIZE_CODE")?;
    // let from_email = env::var("FROM_EMAIL")?;
    // let to_email = env::var("TO_EMAIL")?;

    let mail_author_code = "gfufdphqdxxsbbje";
    let from_email = "592778939@qq.com";
    let to_email = "3870180466@qq.com";

    let db = db_client.get_db();
    let alarm_info_collection = db.collection::<AlarmInfo>(AlarmInfo::get_collection_name());
    
    // let email = Message::builder()
    //     .from("592778939@qq.com".parse()?)
    //     .to("3870180466@qq.com".parse()?)
    //     .subject("来自服务端的警告信息");

    let credits = Credentials::new(from_email.to_string(),mail_author_code.to_string());

    let mailer = SmtpTransport::starttls_relay("smtp.qq.com")?
        .credentials(credits)
        .port(587)
        .build();
    println!("{} [send_task]报警推送任务启动", BLUE);
    loop {
        match timeout(Duration::from_secs(20),rx.recv()).await {
            Ok(Some(sms_type)) => {
                match sms_type {
                    SMSType::TiltWarnSensor(num,pitch,roll,hours) => {
                        
                        let email = match Message::builder()
                            .from(from_email.parse()?)
                            .to(to_email.parse()?)
                            .subject("来自服务端的警告信息")
                            .body(format!("-[警告]- {}号传感器的岩体倾斜预警",num))
                        {
                            Ok(email) => email,
                            Err(e) => {
                                eprintln!("{} [send_task] 邮件构建失败",RED);
                                continue;
                                // return Err(anyhow!(e))
                            }
                        };
                        match mailer.send(&email) {
                            Ok(_) => {
                                // 发送成功，存到数据库里面
                                // 由于报警发送具有冷却期，所以在send_task里面可以直接存入
                                // 反之，则需要单开一个线程来处理数据库存入
                                println!("{} [send_task] 邮件发送成功",GREEN);

                                let alarm_data = AlarmInfo {
                                    alarm_id : None,
                                    alarm_type : "岩体倾斜预警".to_string(),
                                    alarm_time : Some(DateTime::now()), // 数据库里面显示的是0时区的时间，所以中国时间要加上8
                                    sensor_num : num,
                                    level : 2,
                                    infos : InFo::new_tilt(pitch,roll,hours),

                                };

                                match alarm_info_collection.insert_one(alarm_data).await {
                                    Ok(_result) => {
                                        println!("{} [send_task]成功将该日志存入数据库",GREEN)
                                    },
                                    Err(err) => {
                                        eprintln!("{} [send_task]日志存入数据库是失败，原因如下: {:?}",RED,err);
                                    }

                                }

                            },
                            Err(e) => {
                                eprintln!("{} [send_task]邮件发送失败，请检查配置内容",RED)
                            }
                        }
                    }
                    SMSType::TiltAlarmSensor(num,pitch,roll,hours) => {
                        let email = match Message::builder()
                            .from(from_email.parse()?)
                            .to(to_email.parse()?)
                            .subject("来自服务端的警告信息")
                            .body(format!("-[危险]- {}号传感器的岩体倾斜报警",num))
                        {
                            Ok(email) => email,
                            Err(e) => {
                                eprintln!("{} [send_task]邮件构建失败",RED);
                                continue;
                                // return Err(anyhow!(e))
                            }
                        };
                        match mailer.send(&email) {
                            Ok(_) => {
                                println!("{} [send_task] 邮件发送成功",GREEN);
                                let alarm_data = AlarmInfo {
                                    alarm_id : None,
                                    alarm_type : "岩体倾斜报警".to_string(),
                                    alarm_time : Some(DateTime::now()), // 数据库里面显示的是0时区的时间，所以中国时间要加上8
                                    sensor_num : num,
                                    level : 1,
                                    infos : InFo::new_tilt(pitch,roll,hours),

                                };

                                match alarm_info_collection.insert_one(alarm_data).await {
                                    Ok(_result) => {
                                        println!("{} [send_task]成功将倾斜报警日志存入数据库",GREEN)
                                    },
                                    Err(err) => {
                                        eprintln!("{} [send_task]倾斜报警日志存入数据库是失败，原因如下: {:?}",RED,err);
                                    }

                                }

                            },
                            Err(e) => {
                                eprintln!("{} [send_task]邮件发送失败，请检查配置内容",RED);
                            }
                        }
                    },
                    SMSType::MicroseismSensor(num,rate_per_min,threshold) => {
                        let email = match Message::builder()
                            .from(from_email.parse()?)
                            .to(to_email.parse()?)
                            .subject("来自服务端的警告信息")
                            .body(format!("-[危险]- {}号传感器的岩体微震报警",num))
                        {
                            Ok(email) => email,
                            Err(e) => {
                                eprintln!("{} [send_task]邮件构建失败",RED);
                                continue;
                                // return Err(anyhow!(e))
                            }
                        };
                        match mailer.send(&email) {
                            Ok(_) => {
                                println!("{} [send_task]邮件发送成功",GREEN);
                                let alarm_data = AlarmInfo {
                                    alarm_id : None,
                                    alarm_type : "岩体微震报警".to_string(),
                                    alarm_time : Some(DateTime::now()), // 数据库里面显示的是0时区的时间，所以中国时间要加上8
                                    sensor_num : num,
                                    level : 1,
                                    infos : InFo::new_microseism(rate_per_min,threshold),

                                };

                                match alarm_info_collection.insert_one(alarm_data).await {
                                    Ok(_result) => {
                                        println!("{} [send_task]成功将微震报警日志存入数据库",GREEN)
                                    },
                                    Err(err) => {
                                        eprintln!("{} [send_task]微震报警日志存入数据库是失败，原因如下: {:?}",RED,err);
                                    }

                                }
                            },
                            Err(e) => {
                                eprintln!("{} [send_task]邮件发送失败，请检查配置内容",RED)
                            }
                        }
                    },
                    SMSType::VarianceGrowSensor(num,current,baseline,ratio) => {
                        let email = match Message::builder()
                            .from(from_email.parse()?)
                            .to(to_email.parse()?)
                            .subject("来自服务端的警告信息")
                            .body(format!("-[危险]- {}号传感器的岩体裂缝扩展报警",num))
                        {
                            Ok(email) => email,
                            Err(e) => {
                                eprintln!("邮件构建失败");
                                continue;
                                // return Err(anyhow!(e))
                            }
                        };
                        match mailer.send(&email) {
                            Ok(_) => {
                                println!("{} [send_task] 邮件发送成功",GREEN);
                                let alarm_data = AlarmInfo {
                                    alarm_id : None,
                                    alarm_type : "岩体裂缝扩展报警".to_string(),
                                    alarm_time : Some(DateTime::now()), // 数据库里面显示的是0时区的时间，所以中国时间要加上8
                                    sensor_num : num,
                                    level : 1,
                                    infos : InFo::new_variance(current,baseline,ratio),

                                };

                                match alarm_info_collection.insert_one(alarm_data).await {
                                    Ok(_result) => {
                                        println!("{} [send_task]成功将裂隙扩展日志存入数据库",GREEN)
                                    },
                                    Err(err) => {
                                        eprintln!("{} [send_task]裂隙扩展日志存入数据库是失败，原因如下: {:?}",RED,err);
                                    }

                                }
                            },
                            Err(e) => {
                                eprintln!("{} [send_task]邮件发送失败，请检查配置内容",RED);
                            }
                        }
                    },
                    SMSType::RapidSettleSensor(num,magnitude_g) => {
                        let email = match Message::builder()
                            .from(from_email.parse()?)
                            .to(to_email.parse()?)
                            .subject("来自服务端的警告信息")
                            .body(format!("-[危险]- {}号传感器的岩体沉降报警",num))
                        {
                            Ok(email) => email,
                            Err(e) => {
                                eprintln!("{} [send_task]邮件构建失败",RED);
                                continue;
                                // return Err(anyhow!(e))
                            }
                        };
                        match mailer.send(&email) {
                            Ok(_) => {
                                println!("{} [send_task] 邮件发送成功",GREEN);
                                let alarm_data = AlarmInfo {
                                    alarm_id : None,
                                    alarm_type : "岩体沉降报警".to_string(),
                                    alarm_time : Some(DateTime::now()), // 数据库里面显示的是0时区的时间，所以中国时间要加上8
                                    sensor_num : num,
                                    level : 1,
                                    infos : InFo::new_settlement(magnitude_g),

                                };

                                match alarm_info_collection.insert_one(alarm_data).await {
                                    Ok(_result) => {
                                        println!("{} [send_task]成功将沉降报警日志存入数据库",GREEN)
                                    },
                                    Err(err) => {
                                        eprintln!("{} [send_task]沉降报警日志存入数据库是失败，原因如下: {:?}",RED,err);
                                    }

                                }
                            },
                            Err(e) => {
                                eprintln!("{} [send_task]邮件发送失败，请检查配置内容",RED)
                            }
                        }
                    },
                    SMSType::WrongSensor(num,reason) => {
                        let email = match Message::builder()
                            .from(from_email.parse()?)
                            .to(to_email.parse()?)
                            .subject("来自服务端的警告信息")
                            .body(format!("-[警告]- {}号传感器的岩体传感器异常报警",num))
                        {
                            Ok(email) => email,
                            Err(e) => {
                                eprintln!("{} [send_task]邮件构建失败",RED);
                                continue;
                                // return Err(anyhow!(e))
                            }
                        };
                        match mailer.send(&email) {
                            Ok(_) => {
                                println!("{} [send_task] 邮件发送成功",GREEN);
                                let alarm_data = AlarmInfo {
                                    alarm_id : None,
                                    alarm_type : "传感器问题报警".to_string(),
                                    alarm_time : Some(DateTime::now()), // 数据库里面显示的是0时区的时间，所以中国时间要加上8
                                    sensor_num : num,
                                    level : 1,
                                    infos : InFo::new_sensor_anomaly(reason),

                                };

                                match alarm_info_collection.insert_one(alarm_data).await {
                                    Ok(_result) => {
                                        println!("{} [send_task]成功将传感器问题报警日志存入数据库",GREEN)
                                    },
                                    Err(err) => {
                                        eprintln!("{} [send_task]传感器问题报警日志存入数据库是失败，原因如下: {:?}",RED,err);
                                    }

                                }
                            },
                            Err(e) => {
                                eprintln!("{} [send_task]邮件发送失败，请检查配置内容",RED)
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
                eprintln!("{} [send_task] 任务在20s内未收到sms发送请求，继续等待",YELLOW);
                continue;
            }
        }
    }
    Ok(())
}

















