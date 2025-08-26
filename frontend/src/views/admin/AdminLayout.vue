<template>
  <div class="admin-layout">
    <!-- 移动端顶部导航 -->
    <div class="mobile-header" v-if="isMobile">
      <div class="mobile-header-content">
        <el-button @click="toggleSidebar" :icon="Menu" circle />
        <h2 class="mobile-title">{{ currentPageTitle }}</h2>
        <el-dropdown @command="handleCommand">
          <el-button :icon="User" circle />
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="profile">
                <el-icon><UserFilled /></el-icon>
                个人信息
              </el-dropdown-item>
              <el-dropdown-item command="logout" divided>
                <el-icon><SwitchButton /></el-icon>
                退出登录
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
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
            <el-icon><GamepadFilled /></el-icon>
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
        
        <el-dropdown @command="handleCommand" placement="top">
          <el-button 
            :icon="Setting" 
            circle 
            :class="{ 'collapsed-button': sidebarCollapsed }"
          />
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="home">
                <el-icon><HomeFilled /></el-icon>
                返回首页
              </el-dropdown-item>
              <el-dropdown-item command="logout" divided>
                <el-icon><SwitchButton /></el-icon>
                退出登录
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
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
          <el-button @click="goToHome" :icon="HomeFilled" text>
            返回首页
          </el-button>
          <el-dropdown @command="handleCommand">
            <div class="user-dropdown">
              <span class="username">{{ adminStore.userInfo?.username }}</span>
              <el-icon class="dropdown-icon"><ArrowDown /></el-icon>
            </div>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item disabled>
                  {{ adminStore.isSuperAdmin ? '超级管理员' : '管理员' }}
                </el-dropdown-item>
                <el-dropdown-item command="logout" divided>
                  <el-icon><SwitchButton /></el-icon>
                  退出登录
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
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
import { ElMessage, ElMessageBox } from 'element-plus'
import { 
  Menu, 
  User, 
  UserFilled, 
  SwitchButton, 
  Monitor, 
  Expand, 
  Fold,
  GamepadFilled,
  DocumentChecked,
  Setting,
  HomeFilled,
  ArrowDown
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

// 下拉菜单处理
const handleCommand = async (command: string) => {
  switch (command) {
    case 'home':
      goToHome()
      break
    case 'logout':
      await handleLogout()
      break
    case 'profile':
      // 可以添加个人信息页面
      ElMessage.info('个人信息功能待开发')
      break
  }
}

const goToHome = () => {
  router.push('/')
}

const handleLogout = async () => {
  try {
    await ElMessageBox.confirm(
      '确定要退出登录吗？',
      '确认退出',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    adminStore.logout()
    ElMessage.success('已退出登录')
    router.push('/admin/login')
  } catch {
    // 用户取消
  }
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

.mobile-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

/* 侧边栏 */
.sidebar {
  width: 250px;
  background: white;
  border-right: 1px solid #ebeef5;
  display: flex;
  flex-direction: column;
  transition: all 0.3s ease;
  position: relative;
  z-index: 1000;
}

.sidebar-collapsed {
  width: 64px;
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
}

.sidebar-menu {
  border: none;
}

.sidebar-footer {
  border-top: 1px solid #ebeef5;
  padding: 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
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

.collapsed-button {
  margin: 0 auto;
}

/* 主要内容区域 */
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  transition: all 0.3s ease;
}

.main-content-expanded {
  margin-left: -186px;
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
  overflow-y: auto;
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