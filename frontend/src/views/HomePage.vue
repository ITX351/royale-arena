<template>
  <div class="home-page">
    <!-- 顶部导航 -->
    <header class="header">
      <div class="container">
        <div class="header-content">
          <div class="logo-section">
            <h1 class="logo">雾雨小镇大逃杀</h1>
          </div>
          
          <div class="search-section">
            <el-input
              v-model="gameStore.searchQuery"
              placeholder="搜索游戏..."
              :prefix-icon="Search"
              class="search-input"
              clearable
            />
            <el-select 
              v-model="gameStore.statusFilter" 
              placeholder="状态筛选" 
              class="status-filter"
            >
              <el-option label="全部" value="all" />
              <el-option label="活动中" value="active" />
              <el-option label="等待中" value="waiting" />
              <el-option label="进行中" value="running" />
              <el-option label="已结束" value="ended" />
              <el-option label="已隐藏" value="hidden" />
            </el-select>
          </div>
          
          <div class="admin-section">
            <el-button 
              @click="refreshGames"
              :loading="gameStore.loading"
              :icon="Refresh"
              circle
              title="刷新游戏列表"
            />
            <el-button 
              type="primary" 
              @click="goToAdminLogin"
              :icon="User"
            >
              管理员登录
            </el-button>
          </div>
        </div>
      </div>
    </header>

    <!-- 主要内容 -->
    <main class="main-content">
      <div class="container">
        <!-- 错误提示 -->
        <el-alert
          v-if="gameStore.error"
          :title="gameStore.error"
          type="error"
          :closable="true"
          @close="gameStore.clearError"
          class="error-alert"
        />
        
        <!-- 游戏列表 -->
        <div class="games-grid" v-loading="gameStore.loading">
          <GameCard
            v-for="game in paginatedGames"
            :key="game.id"
            :game="game"
            @view-detail="handleViewGameDetail"
          />
        </div>
        
        <!-- 空状态 -->
        <div v-if="!gameStore.loading && gameStore.filteredGames.length === 0" class="empty-state">
          <el-empty 
            :description="getEmptyStateText()"
            :image-size="200"
          >
            <el-button type="primary" @click="resetFilters">
              重置筛选
            </el-button>
          </el-empty>
        </div>
        
        <!-- 分页 -->
        <div class="pagination-wrapper" v-if="gameStore.filteredGames.length > pageSize">
          <el-pagination
            v-model:current-page="currentPage"
            :page-size="pageSize"
            :total="gameStore.filteredGames.length"
            layout="prev, pager, next, jumper, total"
            @current-change="handlePageChange"
          />
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { Search, User, Refresh } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import GameCard from '@/components/GameCard.vue'
import { useGameStore } from '@/stores/game'
import type { GameListItem } from '@/types/game'

const router = useRouter()
const gameStore = useGameStore()

// 响应式数据
const currentPage = ref(1)
const pageSize = ref(12)

// 计算属性
const paginatedGames = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return gameStore.filteredGames.slice(start, end)
})

// 方法
const handleViewGameDetail = (game: GameListItem) => {
  router.push(`/game/${game.id}`)
}

const handlePageChange = (page: number) => {
  currentPage.value = page
  // 滚动到页面顶部
  window.scrollTo({ top: 0, behavior: 'smooth' })
}

const goToAdminLogin = () => {
  router.push('/admin/login')
}

const resetFilters = () => {
  gameStore.setSearchQuery('')
  gameStore.setStatusFilter('all')
  currentPage.value = 1
}

const getEmptyStateText = () => {
  if (gameStore.statusFilter === 'hidden') {
    return '没有找到已隐藏的游戏'
  }
  if (gameStore.searchQuery.trim()) {
    return `没有找到与 "${gameStore.searchQuery}" 相关的游戏`
  }
  return '暂无游戏，请联系管理员创建游戏'
}

// 手动刷新功能
const refreshGames = async () => {
  await gameStore.refreshGames()
  ElMessage.success('刷新成功')
}

// 监听筛选条件变化，重置页码
watch([() => gameStore.searchQuery, () => gameStore.statusFilter], () => {
  currentPage.value = 1
})

// 生命周期
onMounted(async () => {
  await gameStore.loadGames()
})
</script>

<style scoped>
.home-page {
  min-height: 100vh;
  background-color: #f5f7fa;
}

.header {
  background: white;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  position: sticky;
  top: 0;
  z-index: 100;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 64px;
  gap: 16px;
}

.logo-section .logo {
  margin: 0;
  font-size: 24px;
  font-weight: bold;
  color: #409eff;
}

.search-section {
  display: flex;
  gap: 12px;
  flex: 1;
  max-width: 400px;
}

.search-input {
  flex: 1;
}

.status-filter {
  width: 120px;
}

.admin-section {
  display: flex;
  gap: 8px;
  align-items: center;
}

.main-content {
  padding: 24px 0;
}

.error-alert {
  margin-bottom: 16px;
}

.games-grid {
  display: grid;
  gap: 20px;
  margin-bottom: 24px;
}

.empty-state {
  margin: 48px 0;
}

/* 响应式设计 */
@media (max-width: 767px) {
  .header-content {
    flex-direction: column;
    height: auto;
    padding: 12px 0;
    gap: 12px;
  }
  
  .search-section {
    order: 2;
    width: 100%;
    max-width: none;
  }
  
  .admin-section {
    order: 3;
    align-self: flex-end;
  }
  
  .games-grid {
    grid-template-columns: 1fr;
    gap: 12px;
  }
}

@media (min-width: 768px) and (max-width: 1023px) {
  .games-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
  }
}

@media (min-width: 1024px) {
  .games-grid {
    grid-template-columns: repeat(3, 1fr);
    gap: 20px;
  }
}
</style>