

use gd_common::{
    func::algor_web_share_state::{
        SharedState,
        SensorState,
        AlarmState,
    }
};

use actix_web::{
    web,
    App,
    HttpResponse,
    middleware
};

use crate::model::{
    api_model::ApiResponse,
    request_model::{
        SensorRequest,
        AlertRequest,
    },
};


pub async fn get_sensors(data: web::Data<SharedState>) -> HttpResponse {

    let state = data.read().await;
    let mut sensors: Vec<&SensorState> = state.sensors.values().collect();
    sensors.sort_by_key(|s| {s.imu_number});
    let response = ApiResponse::success(200,"获取到传感器组信息",sensors);
    HttpResponse::Ok().json(response)
}



pub async fn get_sensor_by_id(data: web::Data<SharedState>, request: web::Json<SensorRequest>) -> HttpResponse {

    if request.req_type != 2 {
        let response = ApiResponse::<()>::error(400,"不是指定的操作类型，请重试");
        return HttpResponse::BadRequest().json(response);
    }

    let id = match request.info.parse::<usize>() {
        Ok(num) => num,
        Err(_) => {
            let response = ApiResponse::<()>::error(400,"请输入数字字符");
            return HttpResponse::BadRequest().json(response)
        },
    };

    let state = data.read().await;

    match state.sensors.get(&id) {
        Some(sensor) => {
            let response = ApiResponse::success(200, "找到指定传感器",sensor);
            HttpResponse::Ok().json(response)
        },
        None => {
            let response = ApiResponse::<()>::error(404, "未找到指定传感器");
            HttpResponse::NotFound().json(response)
        }
    }

}


pub async fn get_alarms(data: web::Data<SharedState>,request: web::Json<AlertRequest>) -> HttpResponse {
    // 1为指定数量或全部
    if request.req_type != 1 {
        let response = ApiResponse::<()>::error(400,"不是指定的操作类型，请重试");
        return HttpResponse::BadRequest().json(response);
    }

    let limit = if request.limit > 50 {
        50
    }else { 
        request.limit
    };

    let state = data.read().await;
    let alarms:Vec<&AlarmState> = state.alarms.iter().rev().take(limit).collect();
    
    let response = ApiResponse::success(200,"获取局部alarm数据成功",alarms);
    HttpResponse::Ok().json(response)

}

