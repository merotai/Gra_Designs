


pub mod handler;
pub mod model;

use actix_web::{
    App,
    HttpServer,
    web,
};
use actix_cors::{
    Cors
};

use gd_common::{
    data_base::{
        db_mongo::{
            MongoDBClient
        },

    },
    func::algor_web_share_state::SharedState,
};


use crate::handler::{
    sensor_handler::{
        get_sensors,
        get_sensor_by_id,
        get_alarms
    },
    log_handler::{
        get_logs_from_db
    }
};

pub async fn run_server_task (shared_state: SharedState) -> std::io::Result<()> {

    // 在服务器部署中需要改换为0.0.0.0:(指定端口)
    // 此处用作本地测试进行演示
    println!("[server_task] 任务启动，地址端口为 127.0.0.1:6202");

    HttpServer::new(
        move || {
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header();
            App::new()
                .wrap(cors)
                .app_data(web::Data::new(shared_state.clone()))
                .service(
                    web::scope("/api")
                        .route("/sensors",      web::get().to(get_sensors))
                        .route("/sensor",       web::post().to(get_sensor_by_id))
                        .route("/alarms",       web::post().to(get_alarms))
                )
                .service(
                    web::scope("/log")
                        .route("/alert_log",web::post().to(get_logs_from_db))

                )
        }
    )
        .bind(("127.0.0.1", 6202))?
        .run()
        .await



}
