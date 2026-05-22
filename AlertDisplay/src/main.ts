import { createApp } from 'vue'
import {createPinia} from "pinia";
import {monitor_routers} from "@/router/index.ts"

import './style.css'


import monitorAPP from "./monitorAPP.vue";


const App = createApp(monitorAPP);
App.use(createPinia());
App.use(monitor_routers);
App.mount('#app');
