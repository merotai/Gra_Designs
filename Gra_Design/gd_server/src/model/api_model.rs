use serde::{
    Serialize,
    Deserialize,
};

#[derive(Debug,Serialize,Deserialize)]
pub struct ApiResponse<T> {
    pub code: u32,
    pub msg: String,

    #[serde(skip_serializing_if="Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(code_h: u32,msg_h: &str,data_h: T) -> Self {
        ApiResponse {
            code: code_h,
            msg : msg_h.to_string(),
            data: Some(data_h),
        }
    }

    pub fn error(code_h: u32, msg_h: &str) -> Self {
        ApiResponse {
            code: code_h,
            msg : msg_h.to_string(),
            data: None,
        }
    }
}






