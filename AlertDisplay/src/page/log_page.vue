<script setup lang="ts">

import {onMounted,ref} from "vue";
import {useMonitorStorage} from "@/store/monitorStorage";


const monitorStore = useMonitorStorage();



const formatAlarmTime = (value: unknown) => {
  if (!value) return "-";
  if (typeof value === "string") return value;
  if (typeof value === "object") {
    const maybe = value as {$data?: string | {$numberLong?: string} };
    if (typeof maybe.$data === "string") return maybe.$data;
    if (typeof maybe.$data === "object" && maybe.$data.$numberLong) {
      const timestamp = Number(maybe.$data.$numberLong);
      if (!Number.isNaN(timestamp)) {
        return new Date(timestamp).toISOString();
      }
    }
  }
  return String(value);
}


</script>

<template>
  <div class="dashboard">
    <header class="dashboard-header">
      <div>
        <p class = "eyebrow">日志展示</p>
        <h1>总览</h1>
        <p class = "subtitle">可以获取日志信息进行查询</p>
      </div>
    </header>

    <section class="grid">

      <article class="card log-card">
        <div class="card-head">
          <h2>告警日志</h2>
          <p>分页查询历史记录</p>
        </div>
        <div class="log-toolbar">
          <label>
            页码
            <input v-model.number="monitorStore.logPage" type="number" min="1" />
          </label>
          <label>
            每页
            <input v-model.number="monitorStore.logPageSize" type="number" min="1" max="20" />
          </label>
          <button @click="monitorStore.loadLog" :disabled="monitorStore.loading">查询</button>
        </div>
        <div class="log-table">
          <div class="log-row log-head">
            <span>传感器</span>
            <span>类型</span>
            <span>等级</span>
            <span>时间</span>
          </div>
          <div v-for="(log, index) in monitorStore.logs" :key="`log-${index}`" class="log-row">
            <span>IMU {{ log.sensor_num }}</span>
            <span>{{ log.alarm_type }}</span>
            <span>{{ log.level }}</span>
            <span>{{ formatAlarmTime(log.alarm_time) }}</span>
          </div>
          <p v-if="!monitorStore.logs.length" class="empty">暂无日志</p>
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