




use tokio_stream::StreamExt;
use actix_web::{
    web,
    HttpResponse,
};
use mongodb::{
    Collection,
    options::{FindOptions},
    bson::doc,
};

use crate::model::{
    api_model::ApiResponse,
    request_model::{
        LogRequest,
    },
    response_model::LogResponse,

};

use gd_common::{
    data_base::{
        db_mongo,
        db_mongo_models::AlarmInfo
    }
};
use gd_common::{
    func::algor_web_share_state::SharedState
};
/// 获取到指定页码的数据，规定前端每页有 5组数据
pub async  fn get_logs_from_db(data: web::Data<SharedState>,request: web::Json<LogRequest>) -> HttpResponse {

    if request.req_type != 0 {
        let response = ApiResponse::<()>::error(400,"请求类型不匹配");
        return HttpResponse::InternalServerError().json(response)
    }
    let state = data.read().await;

    let db_client = match state.db_client.clone() {
        Some(db_client) => db_client,
        None => {
            let response = ApiResponse::<()>::error(400,"未找到数据库连接");
            return HttpResponse::BadRequest().json(response)
        },
    };

    let db = db_client.get_db();
    let alarm_info_collection: Collection<AlarmInfo> = db.collection(AlarmInfo::get_collection_name());

    let page_l =  request.page ;
    let page_size_l = request.page_size;
    let skip = (page_l - 1) * page_size_l;

    let find_option = FindOptions::builder()
        .skip(skip as u64)
        .limit(page_size_l as i64)
        .build();

    match alarm_info_collection
        .find(doc! {})
        .with_options(find_option)
        .await {
        Ok(mut cursor) => {
            let mut logs_list = Vec::new();
            while let Some(log) = cursor.try_next().await.unwrap_or(None) {
                logs_list.push(log);
            }
            let response = LogResponse {
                code: 200,
                msg: "成功获取到指定页码的数据".to_string(),
                data: logs_list,
                page: page_l,
                page_size: page_size_l,
            };
            HttpResponse::Ok().json(response)
        },
        Err(err) => {
            eprintln!("数据库查询错误: {}",err);
            let response = ApiResponse::<()>::error(400,"数据库查询错误");
            HttpResponse::InternalServerError().json(response)
        }
    }



}




















