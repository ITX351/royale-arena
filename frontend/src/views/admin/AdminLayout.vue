<template>
  <div class="admin-layout">
    <!-- 移动端顶部导航 -->
    <div class="mobile-header" v-if="isMobile">
      <div class="mobile-header-content">
        <el-button @click="toggleSidebar" :icon="Menu" circle />
        <h2 class="mobile-title">{{ currentPageTitle }}</h2>
        <div class="mobile-header-right">
          <span class="mobile-username">{{ adminStore.userInfo?.username }}</span>
          <el-button @click="goToHome" :icon="House" circle size="small" />
          <el-button @click="handleLogout" :icon="SwitchButton" circle size="small" type="danger" />
        </div>
      </div>
    </div>

    <!-- 侧边栏 -->
    <aside class="sidebar" :class="{ 'sidebar-collapsed': sidebarCollapsed, 'sidebar-mobile': isMobile }">
      <div class="sidebar-header">
        <h1 class="sidebar-title" v-if="!sidebarCollapsed || isMobile">
          <el-icon><Monitor /></el-icon>
          管理后台
        </h1>
        <el-button 
          v-if="!isMobile"
          @click="toggleSidebar" 
          :icon="sidebarCollapsed ? Expand : Fold" 
          circle 
          size="small"
          class="sidebar-toggle"
        />
      </div>

      <nav class="sidebar-nav">
        <el-menu
          :default-active="$route.path"
          class="sidebar-menu"
          :collapse="sidebarCollapsed && !isMobile"
          router
        >
          <el-menu-item index="/admin/games">
            <el-icon><Setting /></el-icon>
            <span>游戏管理</span>
          </el-menu-item>
          
          <el-menu-item index="/admin/rules">
            <el-icon><DocumentChecked /></el-icon>
            <span>规则模版</span>
          </el-menu-item>
          
          <el-menu-item 
            index="/admin/admins" 
            v-if="adminStore.isSuperAdmin"
          >
            <el-icon><UserFilled /></el-icon>
            <span>管理员管理</span>
          </el-menu-item>
        </el-menu>
      </nav>

      <!-- 用户信息（桌面端） -->
      <div class="sidebar-footer" v-if="!isMobile">
        <div class="user-info" v-if="!sidebarCollapsed">
          <div class="user-avatar">
            <el-icon><User /></el-icon>
          </div>
          <div class="user-details">
            <div class="user-name">{{ adminStore.userInfo?.username }}</div>
            <div class="user-role">
              {{ adminStore.isSuperAdmin ? '超级管理员' : '管理员' }}
            </div>
          </div>
        </div>
        
        <div class="sidebar-actions">
          <el-button 
            @click="goToHome"
            :icon="House" 
            circle 
            size="small"
            title="返回首页"
          />
          <el-button 
            @click="handleLogout"
            :icon="SwitchButton" 
            circle 
            size="small"
            type="danger"
            title="退出登录"
          />
        </div>
      </div>
    </aside>

    <!-- 主要内容区域 -->
    <main class="main-content" :class="{ 'main-content-expanded': sidebarCollapsed && !isMobile, 'main-content-mobile': isMobile }">
      <!-- 桌面端顶部栏 -->
      <header class="content-header" v-if="!isMobile">
        <div class="content-header-left">
          <h2 class="page-title">{{ currentPageTitle }}</h2>
        </div>
        <div class="content-header-right">
          <div class="user-info">
            <span class="username">{{ adminStore.userInfo?.username }}</span>
            <el-tag 
              :type="adminStore.isSuperAdmin ? 'danger' : 'primary'" 
              size="small" 
              effect="dark"
            >
              {{ adminStore.isSuperAdmin ? '超级管理员' : '管理员' }}
            </el-tag>
          </div>
          <el-button @click="goToHome" :icon="House" text>
            返回首页
          </el-button>
          <el-button @click="handleLogout" :icon="SwitchButton" text type="danger">
            退出登录
          </el-button>
        </div>
      </header>

      <!-- 页面内容 -->
      <div class="page-content">
        <router-view />
      </div>
    </main>

    <!-- 移动端侧边栏遮罩 -->
    <div 
      v-if="isMobile && !sidebarCollapsed" 
      class="sidebar-overlay"
      @click="closeSidebar"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import { 
  Menu, 
  User, 
  UserFilled, 
  SwitchButton, 
  Monitor, 
  Expand, 
  Fold,
  DocumentChecked,
  House
} from '@element-plus/icons-vue'
import { useAdminStore } from '@/stores/admin'

const router = useRouter()
const route = useRoute()
const adminStore = useAdminStore()

// 响应式状态
const sidebarCollapsed = ref(false)
const isMobile = ref(false)

// 计算属性
const currentPageTitle = computed(() => {
  const titles: Record<string, string> = {
    '/admin/games': '游戏管理',
    '/admin/rules': '规则模版管理',
    '/admin/admins': '管理员管理'
  }
  return titles[route.path] || '管理后台'
})

// 响应式检测
const checkMobile = () => {
  isMobile.value = window.innerWidth < 768
  if (isMobile.value) {
    sidebarCollapsed.value = true
  }
}

// 侧边栏控制
const toggleSidebar = () => {
  sidebarCollapsed.value = !sidebarCollapsed.value
}

const closeSidebar = () => {
  if (isMobile.value) {
    sidebarCollapsed.value = true
  }
}

// 下拉菜单处理（移动端使用） - 已移除，直接使用按钮

const goToHome = () => {
  router.push('/')
}

const handleLogout = () => {
  adminStore.logout()
  ElMessage.success('已退出登录')
  router.push('/admin/login')
}

// 生命周期
onMounted(() => {
  checkMobile()
  window.addEventListener('resize', checkMobile)
})

onUnmounted(() => {
  window.removeEventListener('resize', checkMobile)
})
</script>

<style scoped>
.admin-layout {
  display: flex;
  height: 100vh;
  background-color: #f5f7fa;
  width: 100vw;
  position: fixed;
  top: 0;
  left: 0;
}

/* 移动端顶部导航 */
.mobile-header {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 60px;
  background: white;
  border-bottom: 1px solid #ebeef5;
  z-index: 1001;
}

.mobile-header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  padding: 0 16px;
}

.mobile-header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.mobile-username {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
  margin-right: 8px;
}

.mobile-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

/* 侧边栏 */
.sidebar {
  width: 250px;
  min-width: 250px;
  max-width: 250px;
  background: white;
  border-right: 1px solid #ebeef5;
  display: flex;
  flex-direction: column;
  transition: all 0.3s ease;
  position: relative;
  z-index: 1000;
  height: 100vh;
}

.sidebar-collapsed {
  width: 64px;
  min-width: 64px;
  max-width: 64px;
}

.sidebar-mobile {
  position: fixed;
  top: 0;
  left: 0;
  height: 100vh;
  z-index: 1002;
  transform: translateX(-100%);
}

.sidebar-mobile:not(.sidebar-collapsed) {
  transform: translateX(0);
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  border-bottom: 1px solid #ebeef5;
  height: 64px;
  box-sizing: border-box;
}

.sidebar-title {
  margin: 0;
  font-size: 18px;
  font-weight: bold;
  color: #409eff;
  display: flex;
  align-items: center;
  gap: 8px;
}

.sidebar-toggle {
  flex-shrink: 0;
}

.sidebar-nav {
  flex: 1;
  padding: 16px 0;
  overflow: hidden;
}

.sidebar-menu {
  border: none;
}

.sidebar-footer {
  border-top: 1px solid #ebeef5;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.sidebar-collapsed .sidebar-footer {
  align-items: center;
  padding: 16px 8px;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.sidebar-actions {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  transition: all 0.3s ease;
}

.sidebar-collapsed .sidebar-actions {
  flex-direction: column;
  gap: 18px;
  align-items: center;
  justify-content: center;
  width: 100%;
}

.sidebar-collapsed .sidebar-actions .el-button {
  margin: 0 !important;
  align-self: center;
}

.user-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: #409eff;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
}

.user-details {
  flex: 1;
  min-width: 0;
}

.user-name {
  font-weight: 600;
  color: #303133;
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.user-role {
  font-size: 12px;
  color: #909399;
}

/* 主要内容区域 */
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  width: calc(100vw - 250px);
  transition: all 0.3s ease;
  height: 100vh;
}

.main-content-expanded {
  width: calc(100vw - 64px);
}

.main-content-mobile {
  padding-top: 60px;
}

.content-header {
  background: white;
  border-bottom: 1px solid #ebeef5;
  padding: 0 24px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.content-header-left {
  flex: 1;
}

.page-title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: #303133;
}

.content-header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.user-dropdown {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  padding: 8px 12px;
  border-radius: 4px;
  transition: background-color 0.3s;
}

.user-dropdown:hover {
  background-color: #f5f7fa;
}

.username {
  font-weight: 500;
  color: #303133;
}

.dropdown-icon {
  font-size: 12px;
  color: #909399;
}

.page-content {
  flex: 1;
  padding: 24px;
  overflow-y: visible;
}

/* 移动端遮罩 */
.sidebar-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 999;
}

/* 响应式设计 */
@media (max-width: 767px) {
  .page-content {
    padding: 16px;
  }
}
</style>