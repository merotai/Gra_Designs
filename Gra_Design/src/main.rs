
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
    data_base::db_mongo,

    func::algor_web_share_state::{
        SharedState,
        DisplayState,
    }
};

use gd_server::{
    run_server_task
};

/// ---------------公共依赖
use anyhow::{
    Result,
    anyhow,
};
use tokio::{
    sync::{
        mpsc,
        RwLock
    },
    task,
    time,
};
use rumqttc::Publish;
use std::{
    time::{
        Duration,
        Instant,
    },
    sync::Arc
};

#[tokio::main]
async fn main() -> Result<()> {
    // 四个任务，三个数据通道
    let (listen_tx, listen_rx) = mpsc::channel::<Publish>(20);
    let (analyse_tx,  analyse_rx) = mpsc::channel::<IMUData>(20);
    let (sms_tx,  sms_rx) = mpsc::channel::<SMSType>(20);
    // 数据库接口
    let db_client = db_mongo::MongoDBClient::db_connect()
        .await
        .expect("Failed to connect to MongoDB");

    // 线程共享状态变量
    let shared_state: SharedState = Arc::new(RwLock::new(DisplayState::new(db_client.clone())));
    let algor_state = shared_state.clone();
    let web_state = shared_state.clone();

    println!("[main]-开始启动所有任务");
    let mut tasks = task::JoinSet::new();
    tasks.spawn(run_mqtt_listen_task(listen_tx));
    tasks.spawn(run_mqtt_handle_task(listen_rx, analyse_tx));
    tasks.spawn(run_algor_analyse_task(analyse_rx, sms_tx, algor_state));
    tasks.spawn(run_sms_send_task(sms_rx,db_client));

    // 为actix_web单独启动一个固定线程进行工作
    // 与tokio相关的4个任务彻底解耦，但是由于采用了Arc封装的参数web_state,因此可以在线程之间进行访问
    std::thread::spawn( move || {
       let result = actix_web::rt::System::new()
           .block_on(run_server_task(web_state));

        match result {
            Ok(()) => {
                println!("[main] web任务正常退出");
            },
            Err(err) => {
                eprintln!("[main] web任务出现异常: {}", err);
            }
        }
    });


    let start = Instant::now();
    let mut check_interval = time::interval(Duration::from_secs(300));// 5min
    // 对各个任务状态进行监控
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
                println!("[main]-[状态报告]- 运行时间: {:?}, 运行的任务: {}/4 , web服务也同时在线", start.elapsed(),running);
            }
        }
    }

    Ok(())
}

// 整个系统任务调度分为两个大部分，(listen,handle + algor + send), actix-web
// listen, handle, algor, send 这四个任务受到tokio运行时调度，但是listen受到外部因素，比如mqtt协议的client制约，才能完全关闭整个main函数
