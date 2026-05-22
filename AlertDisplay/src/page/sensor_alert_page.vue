<script setup lang="ts">

import {onMounted, computed, onUnmounted} from "vue";
import {useMonitorStorage} from "@/store/monitorStorage"


const monitorStore = useMonitorStorage();

let timer: ReturnType<typeof setInterval> | null = null


const startRefresh = () => {
  timer = setInterval(() => {
    console.log("定时5s刷新数据");
    monitorStore.refresh_sensor_alarm();
  },5000);// 5秒
};

const stopRefresh = () => {
  if (timer) {
    clearInterval(timer);
    timer = null;
    console.log("已停止定时刷新");
  }
}
const barChartWidth = computed(() => {
  const count = monitorStore.sensors.length || 1;
  return count * (monitorStore.barWidth + monitorStore.barGap) + monitorStore.barGap;
});

const alarmCounts = computed(() => {
  const counts: Record<number, number> = {};
  // 先置零
  for (const sensor of monitorStore.sensors) {
    counts[sensor.imu_number] = 0;
  }
  // 再执行
  for (const alarm of monitorStore.alarms) {
    counts[alarm.imu_number] = (counts[alarm.imu_number] ?? 0) + 1;
  }
  return counts
});


const maxAlarmCount = computed(() => {
  const values = Object.values(alarmCounts.value);
  return values.length ? Math.max(...values,1) : 1;
});

const statusClass = (state: string) => {
  if (state === "online") return "state-online";
  if (state === "alarm") return "state-alarm";
  if (state === "warning") return "state-warning";
  return "state-offline";
};





onMounted(() => {
  startRefresh()
})

onUnmounted(() => {
  stopRefresh()
})

</script>


<template>

  <div class="dashboard">

    <!--标题部分-->
    <header class="dashboard-header">
      <div>
        <p class = "eyebrow">传感器监控</p>
        <h1>检测内容总览</h1>
        <p class = "subtitle">可以实时获取传感器状态以及告警内容</p>
      </div>
    </header>

    <p v-if = "monitorStore.errorMessage" class = "error-banner">{{monitorStore.errorMessage}}</p>

    <section class="grid">
      <!--传感器状态展示-->
      <article class="card stat-card">
        <div class="card-head">
          <h2>传感器状态</h2>
          <p>实时状态与校准信息</p>
        </div>
        <div class="sensor-list">
          <div v-for="sensor in monitorStore.sensors" :key="sensor.imu_number" class="sensor-item">
            <div>
              <h3>IMU {{ sensor.imu_number }}</h3>
              <p class="muted">{{ sensor.last_update }}</p>
            </div>
            <div class="sensor-meta">
              <span :class="['status-pill', statusClass(sensor.tilt_state)]">
                {{ sensor.tilt_state }}
              </span>
              <span class="meta-item">Pitch: {{ sensor.pitch.toFixed(2) }}</span>
              <span class="meta-item">Roll: {{ sensor.roll.toFixed(2) }}</span>
              <span class="meta-item">
                校准: {{ sensor.calibration_complete ? "完成" : "未完成" }}
              </span>
              <span class="meta-item">采样: {{ sensor.sample_count }}</span>
            </div>
          </div>
          <p v-if="!monitorStore.sensors.length" class="empty">暂无数据</p>
        </div>
      </article>

    </section>
    <!---->
    <section class="grid">
      <!--报警信息展示-->
      <article class="card alarm-card">
        <div class="card-head">
          <h2>最新报警</h2>
          <p>按时间倒序展示</p>
          <input v-model.number="monitorStore.alarmLimit" type="number" min="1" max="50" />
        </div>
        <ul class="alarm-list">

          <li v-for="(alarm, index) in monitorStore.alarms" :key="`${alarm.timestamp}-${index}`">
            <div>
              <strong>传感器 {{ alarm.imu_number }}号  </strong>
              <span class="muted">{{ alarm.timestamp }}</span>
            </div>
            <p>{{ alarm.alarm_type }} - {{ alarm.detail }}</p>
          </li>
          <li v-if="!monitorStore.alarms.length" class="empty">暂无报警信息</li>

        </ul>
      </article>
      <!--告警数量展示-->
      <article class="card chart-card">
        <div class="card-head">
          <h2>报警分布</h2>
          <p>按传感器编号统计近期报警警数量</p>
        </div>
        <div class="chart-wrap">
          <svg
              :width="barChartWidth"
              :height="monitorStore.barChartHeight"
              :viewBox="`0 0 ${barChartWidth} ${monitorStore.barChartHeight}`"
              role="img"
              aria-label="报警数量柱状图"
          >
            <g v-for="(sensor, index) in monitorStore.sensors" :key="sensor.imu_number">
              <rect
                  :x="monitorStore.barGap + index * (monitorStore.barWidth + monitorStore.barGap)"
                  :y="
                  monitorStore.barChartHeight -
                  (alarmCounts[sensor.imu_number] / maxAlarmCount) *
                    (monitorStore.barChartHeight - 30)
                "
                  :height="
                  (alarmCounts[sensor.imu_number] / maxAlarmCount) *
                  (monitorStore.barChartHeight - 30)
                "
                  :width="monitorStore.barWidth"
                  rx="6"
                  class="bar"
              />
              <text
                  :x="monitorStore.barGap + index * (monitorStore.barWidth + monitorStore.barGap) + monitorStore.barWidth / 2"
                  :y="monitorStore.barChartHeight - 8"
                  text-anchor="middle"
                  class="axis-label"
              >
                {{ sensor.imu_number }}
              </text>
            </g>
          </svg>
          <p v-if="!monitorStore.sensors.length" class="empty">暂无传感器数据</p>
        </div>
      </article>


    </section>


  </div>
</template>

<style scoped>
.dashboard {
  min-height: 100vh;
  padding: 2.5rem 2.5rem 4rem;
  color: #101520;
  background:
      radial-gradient(circle at 12% 10%, rgba(107, 195, 255, 0.2), transparent 55%),
      radial-gradient(circle at 90% 12%, rgba(255, 197, 130, 0.25), transparent 45%),
      #f6f5f2;
  font-family: "IBM Plex Sans", "Segoe UI", "PingFang SC", sans-serif;
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  gap: 2rem;
  flex-wrap: wrap;
  margin-bottom: 2rem;
}

.eyebrow {
  text-transform: uppercase;
  letter-spacing: 0.14em;
  font-size: 0.7rem;
  color: #55706f;
}

h1 {
  font-size: clamp(2rem, 3vw, 2.8rem);
  margin: 0.2rem 0 0.6rem;
}

.subtitle {
  max-width: 40rem;
  color: #4d5b6b;
}

.toolbar {
  display: flex;
  gap: 1rem;
  align-items: center;
  flex-wrap: wrap;
}

.input-group {
  display: flex;
  gap: 0.6rem;
  align-items: center;
}

.input-group label {
  display: grid;
  gap: 0.3rem;
  font-size: 0.85rem;
  color: #4f5c6c;
}

button {
  border: none;
  padding: 0.7rem 1.1rem;
  border-radius: 999px;
  background: #e6ecf2;
  color: #253046;
  cursor: pointer;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 10px 20px rgba(23, 33, 54, 0.1);
}

button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

button.primary {
  background: #1e3d59;
  color: #fefefe;
}

input {
  border: 1px solid rgba(20, 33, 61, 0.2);
  border-radius: 10px;
  padding: 0.45rem 0.7rem;
  min-width: 90px;
}

.error-banner {
  padding: 0.8rem 1rem;
  border-radius: 12px;
  background: #ffe4e0;
  color: #a32020;
  margin-bottom: 1.5rem;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 1.5rem;
  margin-bottom: 1.5rem;
}

.card {
  background: #ffffff;
  border-radius: 24px;
  padding: 1.6rem;
  box-shadow: 0 20px 45px rgba(18, 31, 54, 0.08);
}

.card-head h2 {
  margin-bottom: 0.35rem;
}

.card-head p {
  color: #6c7a89;
  font-size: 0.92rem;
}

.chart-wrap {
  margin-top: 1.6rem;
  padding: 1.2rem;
  border-radius: 18px;
  background: #f0f4f8;
  overflow-x: auto;
}

.bar {
  fill: #4d8df5;
}

.axis-label {
  font-size: 0.7rem;
  fill: #5d6f87;
}

.sensor-list {
  margin-top: 1.2rem;
  display: grid;
  gap: 1rem;
}

.sensor-item {
  padding: 1rem;
  border-radius: 18px;
  background: #f8f7f4;
  display: grid;
  gap: 0.8rem;
}

.sensor-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.6rem 1rem;
  font-size: 0.85rem;
}

.status-pill {
  padding: 0.2rem 0.7rem;
  border-radius: 999px;
  font-weight: 600;
  text-transform: uppercase;
  font-size: 0.7rem;
  letter-spacing: 0.08em;
}

.state-online {
  background: rgba(86, 218, 152, 0.18);
  color: #1b7a50;
}

.state-warning {
  background: rgba(255, 209, 102, 0.28);
  color: #8a5b00;
}

.state-alarm {
  background: rgba(250, 107, 107, 0.2);
  color: #b11919;
}

.state-offline {
  background: rgba(97, 110, 139, 0.2);
  color: #3d4559;
}

.alarm-list {
  list-style: none;
  display: grid;
  gap: 0.8rem;
  margin-top: 1.2rem;
  padding: 0;
}

.alarm-list li {
  padding: 0.9rem 1rem;
  border-radius: 16px;
  background: #f7f1ea;
  display: grid;
  gap: 0.4rem;
}

.log-toolbar {
  display: flex;
  gap: 0.8rem;
  margin-top: 1.2rem;
  align-items: center;
  flex-wrap: wrap;
}

.log-toolbar label {
  display: grid;
  gap: 0.3rem;
  font-size: 0.85rem;
}

.log-table {
  margin-top: 1.2rem;
  display: grid;
  gap: 0.4rem;
}

.log-row {
  display: grid;
  grid-template-columns: 1.1fr 1fr 0.6fr 1.2fr;
  gap: 0.6rem;
  padding: 0.6rem 0.3rem;
  font-size: 0.9rem;
  border-bottom: 1px solid rgba(78, 92, 118, 0.12);
}

.log-head {
  font-weight: 700;
  color: #3c4b5d;
}

.muted {
  color: #7b8696;
  font-size: 0.8rem;
}

.empty {
  color: #7a8695;
  text-align: center;
  margin-top: 1rem;
}

@media (max-width: 900px) {
  .dashboard {
    padding: 2rem 1.5rem 3rem;
  }

  .log-row {
    grid-template-columns: 1fr 1fr;
  }
}
</style>






















