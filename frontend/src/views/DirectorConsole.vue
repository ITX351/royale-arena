<template>
  <div class="director-console">
    <div class="console-header">
      <h2>导演控制台</h2>
      <p>游戏ID: {{ $route.params.id }}</p>
    </div>

    <el-card>
      <el-empty 
        description="导演控制台功能正在开发中..."
        :image-size="200"
      >
        <el-button type="primary" @click="goBack">
          返回游戏详情
        </el-button>
      </el-empty>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()

const goBack = () => {
  router.push(`/game/${route.params.id}`)
}
</script>

<style scoped>
.director-console {
  min-height: 100vh;
  padding: 24px;
  background-color: #f5f7fa;
}

.console-header {
  text-align: center;
  margin-bottom: 24px;
}

.console-header h2 {
  margin: 0 0 8px 0;
  color: #303133;
}

.console-header p {
  margin: 0;
  color: #909399;
}
</style>