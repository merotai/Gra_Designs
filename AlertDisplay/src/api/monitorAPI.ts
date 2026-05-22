import axios from "axios";

import type {
    AlarmState,
    AlarmInfo,
    ApiResponse,
    LogResponse,
    SensorState,
} from "@/type/monitor.ts";



const API_BASE = import.meta.env.VITE_API_BASE_URL ?? "http://127.0.0.1:6202"


export const monitorAPI = {


    async getSensor(): Promise<ApiResponse<SensorState[]>> {
        const response = await axios.get(`${API_BASE}/api/sensors`);
        return response.data;
    },

    async getSensorById(imu_number: number): Promise<ApiResponse<SensorState>> {
        const response = await axios.post(
            `${API_BASE}/api/sensor`,
            {req_type: 2, info: String(imu_number)}
        );
        return response.data;
    },

    async getAlarms(limit:number): Promise<ApiResponse<AlarmState[]>> {
        const response = await axios.post(
            `${API_BASE}/api/alarms`,
            {req_type: 1,limit}
        );
        return response.data;
    },

    async getLogs(page: number,page_size: number): Promise<LogResponse<AlarmInfo>> {
        const response = await axios.post(
            `${API_BASE}/log/alert_log`,
            {req_type: 0, page, page_size}
            );
        return response.data;
    },


};
















