<template>
  <div class="admin-login-page">
    <div class="login-container">
      <div class="login-card">
        <!-- Logo和标题 -->
        <div class="login-header">
          <h1 class="login-title">
            <el-icon><UserFilled /></el-icon>
            管理员登录
          </h1>
          <p class="login-subtitle">Royale Arena 管理后台</p>
        </div>

        <!-- 登录表单 -->
        <el-form 
          ref="loginFormRef"
          :model="loginForm"
          :rules="loginRules"
          class="login-form"
          @submit.prevent="handleLogin"
        >
          <el-form-item prop="username">
            <el-input
              v-model="loginForm.username"
              placeholder="请输入用户名"
              size="large"
              :prefix-icon="User"
              clearable
            />
          </el-form-item>

          <el-form-item prop="password">
            <el-input
              v-model="loginForm.password"
              type="password"
              placeholder="请输入密码"
              size="large"
              :prefix-icon="Lock"
              show-password
              @keyup.enter="handleLogin"
            />
          </el-form-item>

          <el-form-item>
            <div class="form-options">
              <el-checkbox v-model="rememberLogin">
                记住登录状态
              </el-checkbox>
            </div>
          </el-form-item>

          <el-form-item>
            <el-button
              type="primary"
              size="large"
              :loading="adminStore.loading"
              @click="handleLogin"
              class="login-button"
            >
              <el-icon><Right /></el-icon>
              登录
            </el-button>
          </el-form-item>
        </el-form>

        <!-- 错误提示 -->
        <el-alert
          v-if="adminStore.error"
          :title="adminStore.error"
          type="error"
          :closable="true"
          @close="adminStore.clearError"
          class="error-alert"
          show-icon
        />

        <!-- 返回首页链接 -->
        <div class="back-to-home">
          <el-button @click="goToHome" text>
            <el-icon><ArrowLeft /></el-icon>
            返回首页
          </el-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { UserFilled, User, Lock, Right, ArrowLeft } from '@element-plus/icons-vue'
import { useAdminStore } from '@/stores/admin'
import type { LoginCredentials } from '@/types/admin'

const router = useRouter()
const adminStore = useAdminStore()

// 表单引用
const loginFormRef = ref<FormInstance>()

// 表单数据
const loginForm = reactive<LoginCredentials>({
  username: '',
  password: ''
})

// 其他状态
const rememberLogin = ref(true)

// 表单验证规则
const loginRules: FormRules = {
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' }
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' }
  ]
}

// 处理登录
const handleLogin = async () => {
  if (!loginFormRef.value) return

  // 表单验证
  const isValid = await loginFormRef.value.validate().catch(() => false)
  if (!isValid) return

  // 执行登录
  const result = await adminStore.login(loginForm)
  
  if (result.success) {
    ElMessage.success('登录成功')
    router.push('/admin/games')
  }
}

// 返回首页
const goToHome = () => {
  router.push('/')
}

// 认证初始化在路由守卫中处理，这里不需要重复调用
</script>

<style scoped>
.admin-login-page {
  /* 确保占满整个视口 */
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  /* 渐变背景全屏显示 */
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  /* 确保背景不会重复或缩放 */
  background-attachment: fixed;
  background-size: cover;
  background-repeat: no-repeat;
  /* Flexbox 居中布局 */
  display: flex;
  align-items: center;
  justify-content: center;
  /* 滚动处理 */
  overflow: auto;
  /* 防止内容超出时的问题 */
  padding: 20px;
  box-sizing: border-box;
}

.login-container {
  width: 100%;
  max-width: 400px;
  /* 确保容器自身也不会影响背景显示 */
  position: relative;
  z-index: 1;
}

.login-card {
  background: white;
  border-radius: 12px;
  padding: 40px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
}

.login-header {
  text-align: center;
  margin-bottom: 32px;
}

.login-title {
  margin: 0 0 8px 0;
  font-size: 24px;
  font-weight: bold;
  color: #303133;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.login-subtitle {
  margin: 0;
  color: #909399;
  font-size: 14px;
}

.login-form {
  margin-bottom: 16px;
}

/* Element Plus 输入框样式修复 */
.login-form :deep(.el-form-item) {
  margin-bottom: 24px;
}

/* 固定输入框宽度，避免 clearable 和 show-password 按钮导致的闪烁 */
.login-form :deep(.el-input) {
  width: 100%;
}

.login-form :deep(.el-input__wrapper) {
  width: 100%;
  /* 确保宿主元素宽度固定 */
  box-sizing: border-box;
  display: flex;
  align-items: center;
}

/* 输入框内部元素布局 */
.login-form :deep(.el-input__inner) {
  flex: 1;
  /* 为右侧按钮预留空间 */
  padding-right: 8px;
}

/* 右侧按钮区域固定宽度 */
.login-form :deep(.el-input__suffix) {
  /* 固定宽度，不管有没有按钮都占用相同空间 */
  width: 40px;
  display: flex;
  justify-content: center;
  align-items: center;
  flex-shrink: 0;
}

.login-form :deep(.el-input__suffix-inner) {
  display: flex;
  align-items: center;
}

/* 确保按钮区域在空状态时也保持相同宽度 */
.login-form :deep(.el-input__clear),
.login-form :deep(.el-input__password) {
  width: 16px;
  height: 16px;
}

.form-options {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.login-button {
  width: 100%;
}

.error-alert {
  margin-bottom: 16px;
}

.back-to-home {
  text-align: center;
  padding-top: 16px;
  border-top: 1px solid #ebeef5;
}

/* 响应式设计 */
@media (min-width: 768px) {
  .login-container {
    max-width: 600px;
  }
  
  .login-card {
    padding: 48px 60px;
  }
}

@media (min-width: 1024px) {
  .login-container {
    max-width: 720px;
  }
  
  .login-card {
    padding: 56px 80px;
  }
  
  .login-title {
    font-size: 28px;
  }
}

@media (min-width: 1440px) {
  .login-container {
    max-width: 800px;
  }
  
  .login-card {
    padding: 64px 100px;
  }
  
  .login-title {
    font-size: 32px;
  }
}

@media (max-width: 480px) {
  .admin-login-page {
    /* 移动端保持全屏显示 */
    padding: 16px;
    /* 确保内容不会超出屏幕 */
    min-height: 100vh;
  }
  
  .login-container {
    /* 移动端可以更宽一些 */
    max-width: none;
    width: 100%;
  }
  
  .login-card {
    padding: 24px;
    /* 确保卡片不会超出屏幕 */
    margin: 0;
    max-width: 100%;
  }
  
  .login-title {
    font-size: 20px;
  }
}
</style>