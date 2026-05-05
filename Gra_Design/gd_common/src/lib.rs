mod func;

mod data_base;

use std::net::SocketAddr;
use std::time::Instant;
use std::io::{Read,Cursor};

use anyhow::{anyhow, Result};
use bytemuck::{NoUninit, Zeroable};
use crate::func::crc32_check::{verify_samples_crc};



use rumqttc::{
    Publish,
};
///
/// 发来的数据帧是基于小端序构建的
///


trait ReadFrame {
    fn read_u8_le(&mut self) -> Result<u8>;
    fn read_u16_le(&mut self) -> Result<u16>;
    fn read_u32_le(&mut self) -> Result<u32>;
    fn read_u64_le(&mut self) -> Result<u64>;

    fn read_f32_le(&mut self) -> Result<f32>;
}

impl<R:Read> ReadFrame for R {
    fn read_u8_le(&mut self) -> Result<u8> {
        let mut buffer_t = [0u8; 1];
        self.read_exact(buffer_t.as_mut())?;
        Ok(buffer_t[0])
    }

    fn read_u16_le(&mut self) -> Result<u16> {
        let mut buffer_t = [0u8; 2];
        self.read_exact(buffer_t.as_mut())?;
        Ok(u16::from_le_bytes(buffer_t))
    }
    fn read_u32_le(&mut self) -> Result<u32> {
        let mut buffer_t = [0u8; 4];
        self.read_exact(buffer_t.as_mut())?;
        Ok(u32::from_le_bytes(buffer_t))
    }

    fn read_u64_le(&mut self) -> Result<u64> {
        let mut buffer_t = [0u8; 8];
        self.read_exact(buffer_t.as_mut())?;
        Ok(u64::from_le_bytes(buffer_t))
    }

    fn read_f32_le(&mut self) -> Result<f32> {
        let mut buffer_t = [0u8; 4];
        self.read_exact(buffer_t.as_mut())?;
        Ok(f32::from_le_bytes(buffer_t))
    }

}

///--------------------------------------------------------
#[derive(Debug,Clone,Copy,NoUninit,Zeroable)]
#[repr(C)]
pub struct BMI270Samples {
    pub ax_g:   f32,
    pub ay_g:   f32,
    pub az_g:   f32,
    pub gx_dps: f32,
    pub gy_dps: f32,
    pub gz_dps: f32,
}

impl BMI270Samples {

    fn new() -> BMI270Samples {
        Self{
            ax_g: 0.0,
            ay_g: 0.0,
            az_g: 0.0,
            gx_dps: 0.0,
            gy_dps: 0.0,
            gz_dps: 0.0,
        }
    }

    fn new_10_array() -> [BMI270Samples;10] {
        [BMI270Samples{
            ax_g: 0.0,
            ay_g: 0.0,
            az_g: 0.0,
            gx_dps: 0.0,
            gy_dps: 0.0,
            gz_dps: 0.0,
        };10]
    }

    /// 加速度幅度
    pub fn acc_magnitude(&self) -> f32 {
        (self.ax_g.powi(2) + self.ay_g.powi(2) + self.az_g.powi(2)).sqrt()
    }
    /// 角速度幅度
    pub fn gyro_magnitude(&self) -> f32 {
        (self.gx_dps.powi(2) + self.gy_dps.powi(2) + self.gz_dps.powi(2)).sqrt()
    }

    pub fn print_data(data:&[Self;10]) {
        let mut num = 1;
        println!("打印采集的IMU数据");
        for sample in data.iter()  {
            println!("第{}组数据",num);
            println!("{:2} {:2} {:2} {:2} {:2} {:2}",
                     sample.ax_g, sample.ay_g, sample.az_g,
                     sample.gx_dps,sample.gy_dps,sample.gz_dps
            );
            num += 1;

        }
    }
}


pub enum SMSType {
    // 报警类型1 倾斜预警
    TiltWarnSensor(u8),

    // 报警类型2 倾斜报警
    TiltAlarmSensor(u8),
    // 报警类型3 微震报警
    MicroseismSensor(u8),

    
    // 报警类型4 裂隙扩展报警
    VarianceGrowSensor(u8),
    
    // 报警类型5 沉降报警
    RapidSettleSensor(u8),
    
    // 报警类型6 传感器异常
    WrongSensor(u8),
}













///--------------------------------------------------------
/// MQTT

#[derive(Debug,Clone,Copy)]
pub struct IMUData {
    pub imu_num: u8,
    pub imu_dataset: [BMI270Samples;10],

}

impl IMUData {
    pub fn new() -> Self {
        IMUData {
            imu_num: 0,
            imu_dataset: BMI270Samples::new_10_array(),
        }
    }

    fn parse_payload<R: Read>(data: &mut R) -> Result<IMUData> {
        let imu_num_h = data.read_u8_le()?;
        let mut imu_dataset_h = BMI270Samples::new_10_array();
        for sample in imu_dataset_h.iter_mut()  {
            let ax = data.read_f32_le()?;
            let ay = data.read_f32_le()?;
            let az = data.read_f32_le()?;
            let gx = data.read_f32_le()?;
            let gy = data.read_f32_le()?;
            let gz = data.read_f32_le()?;

            *sample = BMI270Samples {
                ax_g: ax,
                ay_g: ay,
                az_g: az,
                gx_dps: gx,
                gy_dps: gy,
                gz_dps: gz,
            }
        }
        Ok(IMUData {
            imu_num: imu_num_h,
            imu_dataset: imu_dataset_h,
        })

    }

    pub fn get_dataset_from_publish(raw_data:Publish) -> Result<IMUData> {
        let payload_h = raw_data.payload;
        let topic_h = raw_data.topic;

        if topic_h.as_str() != "IMU" {
            Err(anyhow!("IMU publish topic expected"))
        } else {
            let mut data_cursor = Cursor::new(payload_h);
            match Self::parse_payload(&mut data_cursor) {
                Ok(data) => {
                    Ok(data)
                }
                Err(e) => {
                    Err(anyhow!("IMU publish 转换失败: {}", e))
                }
            }
        }

    }
    
}























































///--------------------------------------------------------
/// UDP

/// 用于服务端接收数据帧
#[derive(Debug,Clone)]
pub struct UdpPacket {
    pub data:           Vec<u8>,
    pub addr:           SocketAddr,
    pub time_stamp:     Instant,
}
///--------------------------------------------------------
/// 建立连接帧
#[derive(Clone,Debug)]
pub struct FrameConnection {
    header:             u16,
    device_id:          u16,
    frame_type:         u16,
    frame_len:          u16,
    timestamp:          u64,

    protocol_version:   u8,
    order:              u8,
    reserved:           u16,
    crc32_check:        u32,
}
impl FrameConnection {
    /// 该函数是将接收的UDPSocket里面的原始二进制数据解析为对应的结构体实例
    fn from_raw_data<R: Read>(data_h: &mut R) -> Result<FrameConnection> {
        let header_h = data_h.read_u16_le()?;
        let device_id_h= data_h.read_u16_le()?;
        let frame_type_h = data_h.read_u16_le()?;
        let frame_len_h = data_h.read_u16_le()?;
        let timestamp_h = data_h.read_u64_le()?;
        let protocol_version_h = data_h.read_u8_le()?;
        let order_h = data_h.read_u8_le()?;
        let reserved_h = data_h.read_u16_le()?;
        let crc32_check_h = data_h.read_u32_le()?;

        Ok(
            FrameConnection {
                header: header_h,
                device_id: device_id_h,
                frame_type: frame_type_h,
                frame_len: frame_len_h,
                timestamp: timestamp_h,
                protocol_version: protocol_version_h,
                order: order_h,
                reserved: reserved_h,
                crc32_check: crc32_check_h,
            }
        )


    }
    /// 该函数会消耗所有权
    pub fn get_frame_connection(raw_data: Vec<u8>) -> Result<FrameConnection>  {
        let mut data_h = Cursor::new(raw_data);
        match FrameConnection::from_raw_data(&mut data_h) {
            Ok(frame_connect) => {Ok(frame_connect)},
            Err(_) => {
                Err(anyhow!("[UDP-Listen]解析原始连接数据帧时出错"))
            }
        }

    }

    ///
    pub fn check_frame_content(&self) -> Result<()> {
        if self.header != 0xEC13 {
            return Err(anyhow!("[UDP-Listen]数据帧头内容错误"));
        }
        match self.device_id {
            0x0001 => {},
            0x0002 => {},
            _ => {
                return Err(anyhow!("[UDP-Listen]未知设备连接帧"));
            }
        }
        if self.frame_type != 0x0001 {
            return Err(anyhow!("[UDP-Listen]错误帧类型"));
        }
        if self.frame_len != 0x0000 {
            return Err(anyhow!("[UDP-Listen]帧长度错误"))
        }
        if self.protocol_version != 0x01 {
            return Err(anyhow!("[UPD-Listen]协议版本错误"))
        }
        match self.order {
            1 => {},
            2 => {},
            _ => {
                return Err(anyhow!("[UDP-Listen]未知的连接指令"));
            }
        }
        Ok(())
    }

}


///--------------------------------------------------------
/// 构建数据帧
#[derive(Debug,Clone)]
pub struct FrameData {
    header:         u16,
    device_id:      u16,
    frame_type:     u16,
    frame_len:      u16,
    timestamp:      u64,

    samples:        [BMI270Samples; 10],
    crc32_check:    u32,
}

impl FrameData {
    ///
    fn from_raw_data<R: Read>(data_h: &mut R) -> Result<FrameData> {
        let header_h = data_h.read_u16_le()?;
        let device_id_h = data_h.read_u16_le()?;
        let frame_type_h = data_h.read_u16_le()?;
        let frame_len_h = data_h.read_u16_le()?;
        let timestamp_h = data_h.read_u64_le()?;

        let mut samples_h= BMI270Samples::new_10_array();
        for i in 0..10 {
            let smp1 = data_h.read_f32_le()?;
            let smp2 = data_h.read_f32_le()?;
            let smp3 = data_h.read_f32_le()?;
            let smp4 = data_h.read_f32_le()?;
            let smp5 = data_h.read_f32_le()?;
            let smp6 = data_h.read_f32_le()?;
            samples_h[i] = BMI270Samples {
                ax_g: smp1,
                ay_g: smp2,
                az_g: smp3,
                gx_dps: smp4,
                gy_dps: smp5,
                gz_dps: smp6,
            }
        }

        let crc32_check_h = data_h.read_u32_le()?;

        Ok(FrameData {
            header: header_h,
            device_id: device_id_h,
            frame_type: frame_type_h,
            frame_len: frame_len_h,
            timestamp: timestamp_h,
            samples: samples_h,
            crc32_check: crc32_check_h,
        })
    }
    
    /// 该函数会消耗所有权
    pub fn get_frame_data(raw_data: Vec<u8>) -> Result<FrameData> {
        let mut data_h = Cursor::new(raw_data);
        match FrameData::from_raw_data(&mut data_h) {
            Ok(frame_data) => {Ok(frame_data)},
            Err(_) => {
                Err(anyhow!("解析数据帧出错"))
            }
        }
    }

    /// 该函数只进行比较(只读操作)，不会消耗所有权，通过则返回数据内容
    pub fn check_frame_content(&self) -> Result<[BMI270Samples; 10]> {
        if self.header != 0xEC14 {
           return  Err(anyhow!("[UDP-Listen] 数据帧头检测错误"))
        }
        if self.device_id != 0x0001 {
            return Err(anyhow!("[UDP-Listen] 设备ID不匹配"))
        }
        if self.frame_type != 0x0002 {
            return Err(anyhow!("[UDP-Listen] 帧类型不匹配"))
        }

        if self.frame_len != 0x00F0 {
            return Err(anyhow!("[UDP-Listen] 帧长度不匹配"))
        }
        
        if let Err(_) = verify_samples_crc(&self.samples,self.crc32_check) {
            return Err(anyhow!("[UDP-Listen] CRC32校验未通过"))
        }
        Ok(self.samples.clone())
    }

    pub fn print_sample_data(&self) {
        let num = 0;
        println!("打印采集的IMU数据");
        for sample in self.samples.iter() {
            println!("采集数据内容{}:",num);
            println!("{:2} {:2} {:2} {:2} {:2} {:2}",sample.ax_g, sample.ay_g, sample.az_g,sample.gx_dps,sample.gy_dps,sample.gz_dps);
        }
    }
}







pub struct UdpInfos {
    
}








