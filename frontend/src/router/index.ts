import { createRouter, createWebHistory } from 'vue-router'
import { useAdminStore } from '@/stores/admin'

// 懒加载组件
const HomePage = () => import('@/views/HomePage.vue')
const GameDetailPage = () => import('@/views/GameDetailPage.vue')
const PlayerInterface = () => import('@/views/PlayerInterface.vue')
const DirectorMain = () => import('@/views/director/DirectorMain.vue')
const ActorMain = () => import('@/views/actor/ActorMain.vue')
const AdminLoginPage = () => import('@/views/admin/AdminLoginPage.vue')
const AdminLayout = () => import('@/views/admin/AdminLayout.vue')
const AdminGamesPage = () => import('@/views/admin/AdminGamesPage.vue')
const AdminRulesPage = () => import('@/views/admin/AdminRulesPage.vue')
const AdminUsersPage = () => import('@/views/admin/AdminUsersPage.vue')

const routes = [
  // 公共路由
  {
    path: '/',
    name: 'Home',
    component: HomePage,
    meta: {
      title: '雾雨小镇大逃杀 - 首页'
    }
  },
  {
    path: '/game/:id',
    name: 'GameDetail',
    component: GameDetailPage,
    meta: {
      title: '游戏详情'
    }
  },
  {
    path: '/game/:id/player',
    name: 'PlayerInterface',
    component: PlayerInterface,
    meta: {
      title: '玩家界面'
    }
  },
  {
    path: '/game/:id/director',
    name: 'DirectorMain',
    component: DirectorMain,
    meta: {
      title: '导演控制台'
    }
  },
  {
    path: '/game/:id/director/:password',
    name: 'DirectorMainWithPassword',
    component: DirectorMain,
    meta: {
      title: '导演控制台'
    }
  },
  {
    path: '/game/:id/actor',
    name: 'ActorMain',
    component: ActorMain,
    meta: {
      title: '演员界面'
    }
  },
  {
    path: '/game/:id/actor/:password',
    name: 'ActorMainWithPassword',
    component: ActorMain,
    meta: {
      title: '演员界面'
    }
  },
  
  // 管理员登录路由
  {
    path: '/admin/login',
    name: 'AdminLogin',
    component: AdminLoginPage,
    meta: {
      title: '管理员登录'
    }
  },
  
  // 管理员后台路由（需要认证）
  {
    path: '/admin',
    component: AdminLayout,
    meta: {
      requiresAuth: true,
      title: '管理后台'
    },
    redirect: '/admin/games',
    children: [
      {
        path: 'games',
        name: 'AdminGames',
        component: AdminGamesPage,
        meta: {
          title: '游戏管理'
        }
      },
      {
        path: 'rules',
        name: 'AdminRules',
        component: AdminRulesPage,
        meta: {
          title: '规则模版管理'
        }
      },
      {
        path: 'admins',
        name: 'AdminUsers',
        component: AdminUsersPage,
        meta: {
          requiresSuperAdmin: true,
          title: '管理员管理'
        }
      }
    ]
  },
  
  // 404 重定向
  {
    path: '/:pathMatch(.*)*',
    redirect: '/'
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// 路由守卫
router.beforeEach(async (to, _from, next) => {
  const adminStore = useAdminStore()
  
  // 只在需要认证的路由上才初始化认证状态，避免干扰正常的路由跳转
  if (to.meta.requiresAuth && !adminStore.isLoggedIn) {
    try {
      await adminStore.initAuth()
    } catch (err) {
      console.warn('初始化认证状态失败:', err)
    }
  }
  
  // 设置页面标题
  if (to.meta.title) {
    document.title = to.meta.title as string
  }
  
  // 检查是否需要管理员认证
  if (to.meta.requiresAuth) {
    if (!adminStore.isLoggedIn) {
      next('/admin/login')
      return
    }
    
    // 检查是否需要超级管理员权限
    if (to.meta.requiresSuperAdmin && !adminStore.isSuperAdmin) {
      next('/admin/games')
      return
    }
  }
  
  // 如果已登录管理员访问登录页，重定向到后台
  if (to.name === 'AdminLogin' && adminStore.isLoggedIn) {
    next('/admin/games')
    return
  }
  
  next()
})

export default router