


use serde::{Serialize,Deserialize};
use std::{
    collections::HashMap,
    sync::Arc
};
use tokio::sync::RwLock;
use chrono::{
    Utc
};

use crate::data_base::db_mongo::MongoDBClient;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorState {
    pub imu_number: usize,
    pub pitch: f32,
    pub roll: f32,
    pub tilt_state: String,
    pub calibration_complete: bool,
    pub last_update: String,
    pub sample_count: u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmState {
    pub timestamp: String,
    pub imu_number: usize,
    pub alarm_type: String,
    pub detail: String,
}


///
pub struct DisplayState {
    pub sensors: HashMap<usize,SensorState>,
    pub alarms : Vec<AlarmState>,
    pub db_client: Option<MongoDBClient>,

}

pub type SharedState = Arc<RwLock<DisplayState>>;

impl DisplayState {
    pub fn new(client:MongoDBClient) -> Self {
        let mut sensors = HashMap::new();
        for i in 1..3 {
            sensors.insert(i,SensorState{
                imu_number: i,
                pitch: 0.0,
                roll: 0.0,
                tilt_state: "offline".to_string(),
                calibration_complete:false,
                last_update: Utc::now()
                                .format("%%Y-%m-%d %H:%M:%S")
                                .to_string(),
                sample_count: 0,

            });

        }

        Self {
            sensors,
            alarms:Vec::new(),
            db_client: Some(client),
        }
    }

}