

use serde::{
    Deserialize,
    Serialize,

};


///----------------------------------------------------------


#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct SensorRequest {
    pub req_type: usize,    // 0代表默认，1代表全选，2代表单个
    pub info: String,
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct AlertRequest {
    pub req_type: usize,    // 0代表默认，1代表全选,2代表单个
    pub limit: usize,
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct LogRequest {
    pub req_type: usize,
    pub page: usize,        // 页码
    pub page_size: usize,   //每页数量
}



















