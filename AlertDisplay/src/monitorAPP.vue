<script setup lang="ts">
import { RouterLink, RouterView } from "vue-router";
</script>

<template>

  <div class = "app-container">
    <!---->
    <header class = "sidebar">
      <nav>
        <RouterLink to="/profile">内容介绍</RouterLink>
        <RouterLink to="/realtime_check">实时状态查看</RouterLink>
        <RouterLink to="/log_check">日志查看</RouterLink>
      </nav>
    </header>
    <!---->
    <main class="content">
      <RouterView/>
    </main>

  </div>

</template>

<style scoped>
.app-container {
  position: relative;
}

.sidebar {
  position: fixed;
  left: 0;
  top: 0;
  width: 200px;
  height: 100vh;
  background: #e2f3d0;
  padding: 1rem;
  box-sizing: border-box;
  z-index: 100;
}
.content {
  position: fixed;
  left: 200px; /* 直接定位在侧边栏右侧 */
  top: 0;     /* 固定在顶部 */
  right: 0;   /* 右侧紧贴屏幕边缘 */
  bottom: 0;  /* 底部紧贴屏幕边缘 */
  padding: 1rem;
  overflow-y: auto; /* 如果内容超出，允许垂直滚动 */
}

nav {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

nav a {
  text-decoration: none;
  color: #333;
  padding: 0.5rem;
  border-radius: 4px;
}

nav a:hover {
  background: #6585ed;
}

nav a.router-link-active {
  font-weight: bold;
  color: #42b983;
  background: #e3e6df;
}
</style>