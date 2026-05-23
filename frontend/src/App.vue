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
</style>
