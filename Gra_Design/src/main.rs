
/// --------------私有依赖
use gd_connect::{
    run_mqtt_listen_task,
    run_mqtt_handle_task,
};
use gd_algorithm::{
    AlgorParams,
    run_algor_analyse_task,
    run_sms_send_task,
};

use gd_common::{
    IMUData,
    SMSType,
};

/// ---------------公共依赖
use anyhow::{
    Result,
    anyhow,
};
use tokio::{sync::mpsc, task, time};
use rumqttc::Publish;
use std::{
    time::{
        Duration,
        Instant,
    }
};


#[tokio::main]
async fn main() ->Result<()> {
    // 四个任务，三个数据通道
    let (listen_tx, listen_rx) = mpsc::channel::<Publish>(20);
    let (analyse_tx,  analyse_rx) = mpsc::channel::<IMUData>(20);
    let (sms_tx,  sms_rx) = mpsc::channel::<SMSType>(20);

    println!("[main]-开始启动所有任务");

    let mut tasks = task::JoinSet::new();
    tasks.spawn(run_mqtt_listen_task(listen_tx));


    tasks.spawn(run_mqtt_handle_task(listen_rx, analyse_tx));


    tasks.spawn(run_algor_analyse_task(analyse_rx, sms_tx));


    tasks.spawn(run_sms_send_task(sms_rx));



    let start = Instant::now();
    let mut check_interval = time::interval(Duration::from_secs(300));// 5min
    loop {
        tokio::select! {
            Some(result) = tasks.join_next() => {
                match result {
                    Ok(Ok(())) => {
                        println!("[main]-任务正常完成 运行时间 {:?}", start.elapsed());
                    },
                    Ok(Err(e)) => {
                        println!("[main]-任务内部出错 {} 运行时间 {:?}",e,start.elapsed());
                    },
                    Err(e) => {
                        eprintln!("[main]-任务异常出错 {} 运行时间 {:?}",e,start.elapsed());
                    }
                    
                }
                if tasks.is_empty() {
                    println!("[main]-所有任务全部完成");
                    break;
                }
            },
            _ = check_interval.tick() => {
                let running = tasks.len();
                println!("[main]-[状态报告]- 运行时间: {:?}, 运行的任务: {}/4", start.elapsed(),running);
            }
        }
    }

    Ok(())
}
