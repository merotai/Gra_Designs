use serde::{
    Serialize,
    Deserialize
};


use mongodb::{
    bson::{
        oid::ObjectId,
        DateTime
    }
};


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InFo {

    tilt_pitch_info: Option<f32>,
    tilt_roll_info: Option<f32>,
    tilt_hours_info: Option<f32>,

    microseism_rate_info: Option<f32>,
    microseism_threshold_info: Option<f32>,

    variance_current_info: Option<f32>,
    variance_baseline_info: Option<f32>,
    variance_ratio_info: Option<f32>,

    settlement_magnitude_info: Option<f32>,

    sensor_reason_info: Option<String>
}

impl InFo {

    pub fn new_tilt(pitch: f32, roll: f32, hours: f32) -> Self {
        InFo {
            tilt_pitch_info: Some(pitch),
            tilt_roll_info: Some(roll),
            tilt_hours_info: Some(hours),

            microseism_rate_info: None,
            microseism_threshold_info: None,

            variance_current_info: None,
            variance_baseline_info: None,
            variance_ratio_info: None,

            settlement_magnitude_info: None,

            sensor_reason_info: None,

        }
    }

    pub fn new_microseism(rate_min: f32, threshold: f32) -> Self {
        InFo {
            tilt_pitch_info: None,
            tilt_roll_info: None,
            tilt_hours_info: None,

            microseism_rate_info: Some(rate_min),
            microseism_threshold_info: Some(threshold),

            variance_current_info: None,
            variance_baseline_info: None,
            variance_ratio_info: None,

            settlement_magnitude_info: None,

            sensor_reason_info: None,
        }
    }

    pub fn new_variance(current: f32, baseline: f32, ratio: f32) -> Self {
        InFo {
            tilt_pitch_info: None,
            tilt_roll_info: None,
            tilt_hours_info: None,

            microseism_rate_info: None,
            microseism_threshold_info: None,

            variance_current_info: Some(current),
            variance_baseline_info: Some(baseline),
            variance_ratio_info: Some(ratio),

            settlement_magnitude_info: None,

            sensor_reason_info: None,
        }
    }

    pub fn new_settlement(magnitude: f32) -> Self {
        InFo {
            tilt_pitch_info: None,
            tilt_roll_info: None,
            tilt_hours_info: None,

            microseism_rate_info: None,
            microseism_threshold_info: None,

            variance_current_info: None,
            variance_baseline_info: None,
            variance_ratio_info: None,

            settlement_magnitude_info: Some(magnitude),

            sensor_reason_info: None,
        }
    }

    pub fn new_sensor_anomaly(reason: String) -> Self {
        InFo {
            tilt_pitch_info: None,
            tilt_roll_info: None,
            tilt_hours_info: None,

            microseism_rate_info: None,
            microseism_threshold_info: None,

            variance_current_info: None,
            variance_baseline_info: None,
            variance_ratio_info: None,

            settlement_magnitude_info: None,

            sensor_reason_info: Some(reason),
        }
    }

}




#[derive(Clone,Serialize,Deserialize,Debug)]
pub struct AlarmInfo {

    #[serde(rename = "_id",skip_serializing_if="Option::is_none")]
    pub alarm_id: Option<ObjectId>,     //

    pub sensor_num: usize,

    pub alarm_type: String,

    pub level: u16,                     // 当前只有 1, 2两种等级分别对应："报警","预警"

    #[serde(skip_serializing_if="Option::is_none")]
    pub alarm_time: Option<DateTime>,   // 发送警告的的时间

    pub infos: InFo,                  // 补充的信息

}

impl AlarmInfo {

    pub fn get_collection_name() -> &'static str {
        "AlarmInfo"
    }

    pub fn new() -> AlarmInfo {
        Self {
            alarm_id: None,
            alarm_type: "".to_string(),
            sensor_num: 0,
            alarm_time: None,
            level: 0,
            infos: InFo::new_tilt(0.0, 0.0, 0.0),
        }
    }
}





