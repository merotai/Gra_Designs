use serde::{
    Serialize,
    Deserialize
};

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct LogResponse<T> {
    pub code : usize,
    pub msg : String,
    pub data: Vec<T> ,

    pub page: usize,
    pub page_size: usize,


}
