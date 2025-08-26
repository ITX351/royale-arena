<template>
  <div class="admin-users-page">
    <div class="page-header">
      <div class="header-left">
        <h3 class="page-title">管理员管理</h3>
        <p class="page-subtitle">管理系统管理员账户（仅超级管理员可见）</p>
      </div>
      <div class="header-right">
        <el-button type="primary" :icon="Plus">
          添加管理员
        </el-button>
      </div>
    </div>

    <el-card>
      <el-empty 
        description="管理员管理功能正在开发中..."
        :image-size="200"
      >
        <el-button type="primary" @click="goToGames">
          返回游戏管理
        </el-button>
      </el-empty>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { Plus } from '@element-plus/icons-vue'

const router = useRouter()

const goToGames = () => {
  router.push('/admin/games')
}
</script>

<style scoped>
.admin-users-page {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
}

.header-left {
  flex: 1;
}

.page-title {
  margin: 0 0 4px 0;
  font-size: 24px;
  font-weight: 600;
  color: #303133;
}

.page-subtitle {
  margin: 0;
  color: #909399;
  font-size: 14px;
}

.header-right {
  flex-shrink: 0;
}

@media (max-width: 767px) {
  .page-header {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>