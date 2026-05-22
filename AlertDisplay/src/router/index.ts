

import {createRouter, createWebHistory} from "vue-router";

import sensor_alert_page from "@/page/sensor_alert_page.vue";

import log_page from "@/page/log_page.vue";
import  navigate_page from "@/page/navigate_page.vue";

export const monitor_routers = createRouter( {
    history:createWebHistory(),
    routes:[
        {
            path: "/",
            redirect: "/profile"
        },
        {
            path:"/realtime_check",
            name:"RealTime-monitor",
            component: sensor_alert_page
        },
        {
            path: "/log_check",
            name: "Logs",
            component: log_page

        },
        {
            path: "/profile",
            name: "introduce",
            component: navigate_page

        },
    ]
})





















