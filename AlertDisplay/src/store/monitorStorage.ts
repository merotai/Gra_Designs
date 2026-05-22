


import {monitorAPI} from "@/api/monitorAPI.ts";


import {defineStore} from "pinia";

import type {AlarmState, AlarmInfo, SensorState} from "@/type/monitor.ts";


/// 该Storage是Option格式
export const useMonitorStorage = defineStore("MonitorState",{
    state: () => ({
        sensors: [] as SensorState[],
        alarms: [] as AlarmState[],
        logs: [] as AlarmInfo[],

        alarmLimit: 12,
        logPage: 1,
        logPageSize: 5,
        loading: false,
        errorMessage: null as string|null,

        barChartHeight: 160,
        barWidth: 28,
        barGap: 16,


    }),

    actions: {
        async loadSensor() {
            try {
                const response_l = await monitorAPI.getSensor();
                if (response_l.code === 200 && response_l.data) {
                    this.sensors = response_l.data
                } else {
                    this.errorMessage = "[加载传感器数据]错误的反馈码";
                    console.error("错误的反馈码");

                }
            } catch (_) {
                this.errorMessage = "[加载传感器数据]加载传感器数据失败";
                throw new Error("加载传感器数据失败");
            }

        },

        async loadAlarm() {

            try {
                const response = await monitorAPI.getAlarms(this.alarmLimit);
                if (response.code === 200 && response.data) {
                    this.alarms = response.data;
                } else {
                    this.errorMessage = "[加载报警信息]错误的反馈码";
                    console.error("错误的反馈码");
                }
            } catch (_) {
                this.errorMessage = "[加载报警信息]加载报警数据失败";
                throw new Error("加载报警数据失败");
            }

        },

        async loadLog() {
            try {
                const response = await monitorAPI.getLogs(this.logPage, this.logPageSize);
                if (response.code === 200) {
                    this.logs = response.data;

                } else {
                   this.errorMessage = "错误的反馈码";
                   console.error("错误的反馈码");
                }
            } catch (_) {
                this.errorMessage = "加载日志数据失败";
                throw new Error("加载日志数据失败");
            }
        },


        async refreshAll() {
            this.loading = true;
            this.errorMessage = null;
            try {
                await Promise.all([this.loadSensor(),this.loadAlarm(),this.loadLog()]);

            } catch (err) {
                this.errorMessage = err instanceof Error? err.message:"请求失败"
            } finally {
                this.loading = false;
            }
        },

        async refresh_sensor_alarm() {
            this.loading = true;
            try {
                await Promise.all([this.loadSensor(),this.loadAlarm()]);
            } catch (err) {
                this.errorMessage = err instanceof Error? err.message : "刷新失败"
            } finally {
                this.loading = false;
            }
        },



    }
})





























