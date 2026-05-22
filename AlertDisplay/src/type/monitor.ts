

export interface ApiResponse<T> {
    code:   number;
    msg:    string;
    data?:  T;
}


export interface SensorState {
    imu_number:             number;
    pitch:                  number;
    roll:                   number;
    tilt_state:             string;
    calibration_complete:   boolean;
    last_update:            string;
    sample_count:           number;
}

export interface AlarmState {
    timestamp:  string;
    imu_number: number;
    alarm_type: string;
    detail:     string;
}


export interface LogResponse<T> {
    code:       number;
    msg:        string;
    data:       T[];
    page:       number;
    page_size:  number;
}

export interface AlarmInfo {
    sensor_num: number;
    alarm_type: string;
    level:      number;
    alarm_time?: unknown;
    infos:      Record<string, unknown>;
}


export interface Info {

}
