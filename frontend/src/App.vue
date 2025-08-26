<template>
  <div id="app">
    <router-view />
  </div>
</template>

<script setup lang="ts">
// 认证初始化在路由守卫中处理，这里不需要重复调用
</script>

<style>
#app {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Helvetica Neue', sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* 全局样式重置 */
* {
  box-sizing: border-box;
}

body {
  margin: 0;
  padding: 0;
  background-color: #f5f7fa;
}

/* Element Plus 样式调整 */
.el-button + .el-button {
  margin-left: 8px;
}

.el-card {
  border-radius: 8px;
}

.el-table {
  border-radius: 8px;
}

/* 响应式工具类 */
@media (max-width: 767px) {
  .hidden-mobile {
    display: none !important;
  }
}

@media (min-width: 768px) {
  .hidden-desktop {
    display: none !important;
  }
}
</style>
