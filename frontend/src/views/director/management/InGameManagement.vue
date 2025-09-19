<template>
  <div class="in-game-management">
    <el-card class="management-card">
      <template #header>
        <div class="card-header">
          <h3>游戏中管理</h3>
        </div>
      </template>
      
      <div class="management-content">
        <el-alert
          title="游戏中管理功能"
          type="info"
          show-icon
          :closable="false"
          class="management-info"
        >
          <template #default>
            <p>游戏正在进行中，您可以在此管理游戏进程。</p>
            <p>目前支持的操作包括暂停游戏和结束游戏。</p>
          </template>
        </el-alert>
        
        <div class="management-actions">
          <el-button type="warning" size="large" @click="$emit('request-pause')">
            暂停游戏
          </el-button>
          <el-button type="danger" size="large" @click="$emit('request-end')">
            结束游戏
          </el-button>
        </div>
        
        <div class="management-note">
          <el-alert
            title="注意"
            type="warning"
            show-icon
            :closable="false"
          >
            <template #default>
              <p>游戏状态变更需要在导演控制台主界面进行操作。</p>
              <p>请使用页面顶部的状态控制按钮来管理游戏。</p>
            </template>
          </el-alert>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
// Emits
defineEmits<{
  (e: 'request-pause'): void
  (e: 'request-end'): void
}>()
</script>

<style scoped>
.management-card {
  margin-bottom: 24px;
}

.card-header h3 {
  margin: 0;
  color: #303133;
}

.management-content {
  min-height: 300px;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.management-info {
  margin-bottom: 20px;
}

.management-actions {
  display: flex;
  justify-content: center;
  gap: 24px;
  flex-wrap: wrap;
}

.management-note {
  margin-top: auto;
}

@media (max-width: 768px) {
  .management-actions {
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }
}
</style>