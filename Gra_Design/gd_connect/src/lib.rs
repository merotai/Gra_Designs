/// ----------------------------------------------------------------------------
///
///

///
///
/// ----------------------------------------------------------------------------
use tokio::{
    net::{
        UdpSocket,
    },
    sync::{
        mpsc,

    },
    time::timeout,
};

use rumqttc::{
    MqttOptions, AsyncClient, QoS, Event, SubscribeFilter,
    Publish,
    Packet,
};

use anyhow::{anyhow, Result};
use tracing::{info,warn,error};
use std::time::{
    Instant,
    Duration,
};
use dotenv::dotenv;
use std::env;
use gd_common::{
    // UDP
    UdpPacket,
    FrameConnection,
    FrameData,
    BMI270Samples,

    // MQTT
    IMUData,
};



/// ----------------------------------------------------------------------------
/// MQTT

pub async fn run_mqtt_listen_task(
    tx: mpsc::Sender<Publish>
) -> Result<()>{

    // let mqtt_username = env::var("MQTT_USERNAME")?;
    // let mqtt_password = env::var("MQTT_PASSWORD")?;

    let mut gd_mqtt_options = MqttOptions::new(
        "mqtt_receiver",
        "127.0.0.1",
        2026
    );

    gd_mqtt_options.set_keep_alive(Duration::from_secs(5));
    gd_mqtt_options.set_credentials("mqtt_listen_wqy","2026gd_wqy");
    let (client, mut event_loop) = AsyncClient::new(gd_mqtt_options, 10);

    // let topics_h = [
    //     SubscribeFilter::new("IMU".to_string(),QoS::AtLeastOnce),
    //     SubscribeFilter::new("IMU/1".to_string(),QoS::AtLeastOnce),
    //     SubscribeFilter::new("IMU/2".to_string(),QoS::AtLeastOnce),
    //     SubscribeFilter::new("IMU/3".to_string(),QoS::AtLeastOnce),
    // ];
    //
    // client.subscribe_many(topics_h).await?;

    //订阅的单个topic
    client.subscribe("IMU", QoS::AtLeastOnce).await?;

    println!("[listen_task] 任务启动，tx的通道地址是{:p}",&tx);
    loop {
        match event_loop.poll().await {
            Ok(Event::Incoming(Packet::Publish(publish))) => {
                println!("接收到publish, 主题是{:?}", publish.topic);
                match tx.send(publish).await {
                    Ok(()) => {
                        println!("publish内容推送成功");
                    },
                    Err(e) => {
                        eprintln!("publish内容推送失败: {:?}",e);
                        break;
                    }
                }
            },
            Ok(Event::Incoming(Packet::ConnAck(_))) => {
                println!("MQTT 接收端初始化成功");
            },
            Ok(_) => {
                // println!("未处理的定义事件: {:?}", content);
            },
            Err(e) => {
                eprintln!("MQTT 连接错误: {:?}",e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }

        }
    }
    println!("MQTT 监听服务结束");
    Ok(())

}


pub async fn run_mqtt_handle_task(mut rx:mpsc::Receiver<Publish>,tx: mpsc::Sender<IMUData>) -> Result<()> {
    println!("[handle_task] 任务启动，rx的通道地址是{:p}",&rx);
    loop {
        match timeout(Duration::from_secs(20),rx.recv()).await {
            Ok(Some(publish)) => {
                // 成功在20s内收到数据,开始解析数据
                let parse_data = match IMUData::get_dataset_from_publish(publish) {
                    Ok(data) => data,
                    Err(_) => {
                        println!("解析数据失败");
                        continue
                    }
                };
                tx.send(parse_data).await?;

            },
            Ok(None) => {
                println!("监听传输通道关闭");
                break
            },
            Err(e) => {
                eprintln!("[handle_task]出现超时情况，20s内无数据发送过来，请检查数据通道以及采集端");
                return Err(anyhow!(e))
            }

        }
    }
    println!("解析任务执行完成");
    Ok(())
}




/// ----------------------------------------------------------------------------
/// UDP
pub async fn run_udp_listener(

    bind_addr: &str, // 自己设定的IP以及端口号
    tx_channel: mpsc::Sender<[BMI270Samples;10]>,

) -> Result<()>{

    eprintln!("准备绑定到的ip端口: {}",bind_addr);

    let socket_h = match UdpSocket::bind(bind_addr).await {
        Ok(s) => { s },
        Err(e) => {
            return Err(anyhow!(e));
        }
    };

    let addr_remote;

    let connect_commander_ok= "0xbcef019156040fecb-CONNECTION-OK".as_bytes();

    // 首次建立连接需要远端发来udp数据，进行首次握手，完成后即可建立端对端通道
    // A
    loop {
        let mut buffer_h =  [0u8; 1024]; // 设定的数据包大小不超过1024字节
        match socket_h.recv_from(&mut buffer_h).await {
            Ok((len_t, addr_t)) => {
                let connect_packet = UdpPacket {
                    data: buffer_h[..len_t].to_vec(),
                    addr: addr_t,
                    time_stamp: Instant::now(),
                };

                let frame_data = match FrameConnection::get_frame_connection(connect_packet.clone().data) {
                    Ok(frame_d) => { frame_d },
                    Err(_) => {
                        warn!("[UDP-Listen]-A: 解析连接帧数据失败，丢弃该帧接收下一个帧");
                        continue
                    }
                };

                // 数据校验成功则跳出循环，失败则等待下一次数据的校验
                match FrameConnection::check_frame_content(&frame_data) {
                    Ok(_) => {
                        info!("[UDP-Listen]-A: 连接帧数据校验成功，开始建立连接");
                        addr_remote = connect_packet.addr;
                        socket_h.connect(connect_packet.addr).await?;
                        break
                    },
                    Err(e) => {
                        warn!("[UDP-Listen]-A:连接帧数据校验失败: {:?}",e);
                        continue
                    }
                }
            },
            Err(err) => {
                warn!("数据接收出错: {:?}",err);
                continue
            }
        }// match
    } // loop

    // B
    match socket_h.send(connect_commander_ok).await {
        Ok(_) => {
            info!("[UDP-Listen]-B: 发送连接成功信号");
        },
        Err(_) => {
            warn!("[UDP-Listen]-B: 发送连接成功信号失败");
            return Err(anyhow!("[UDP-Listen]-B: 发送连接成功信号失败"))
        }
    }

    // C
    loop {
        let mut buffer_h =  [0u8; 1024]; // 设定的数据包大小不超过1024字节
        match socket_h.recv(&mut buffer_h).await {

            Ok(len_t) => {
                let cur_packet = UdpPacket {
                    data: buffer_h[..len_t].to_vec(),
                    addr: addr_remote,
                    time_stamp: Instant::now(), // 数据包里面还有时间戳，当前这个用于处理服务端逻辑
                };

                // 对数据帧的内容进行解析
                let frame_data = match FrameData::get_frame_data(cur_packet.data) {
                    Ok(frame_d) => { frame_d },
                    Err(_) => {
                        warn!("[UDP-Listen]-C 数据帧解析出错，尝试丢弃");
                        continue
                    }
                };

                // 对数据帧内容进行校验，成功则发向数据通道，失败则丢弃
                let smp_data = match frame_data.check_frame_content() {
                    Ok(data) => {
                        info!("[UDP-Listen]-C 数据校验成功，开始传输数据");
                        data
                    },
                    Err(_) => {
                        warn!("数据校验失败，尝试丢弃该数据包");
                        continue
                    }
                };

                info!("[UDP-Listen]收到数据包: {}byte, from:{}", len_t, cur_packet.addr);
                match tx_channel.try_send(smp_data) {
                    Ok(_) => {
                        continue
                    }, // 不做操作，继续接收
                    Err(mpsc::error::TrySendError::Full(packet)) => { // 通道队列满
                        warn!("[UDP-Listen]通道队列数据已满，尝试丢弃");
                        continue
                    },
                    
                    Err(mpsc::error::TrySendError::Closed(_)) => { // 通道关闭，接收端退出
                        info!("[UDP-Listen]通道关闭，监听任务尝试退出");
                        break
                    }
                }
            },

            Err(err) => {
                error!("[UDP-Listen]无法接收到数据包");
                return Err(anyhow!(err))
            }
        }// match
    }// loop
    info!("[UDP-Listen]结束监听任务");
    Ok(())
}




