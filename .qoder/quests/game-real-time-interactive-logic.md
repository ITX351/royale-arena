# 游戏实时交互逻辑设计

## 1. 概述

本文档描述了 Royale Arena 游戏中，处于"进行中"状态的游戏前端导演页面的实时交互逻辑设计。该设计基于 WebSocket 连接，实现实时操作、交互和消息接收功能。页面将保持 WebSocket 连接，动态更新界面内容。

设计将包含以下主要功能：
- WebSocket 连接管理
- 全局游戏状态展示与修改
- 实时日志消息展示与筛选
- 消息分类显示与动画效果

## 2. 架构设计

### 2.1 前端技术栈
- **框架**: Vue 3 (Composition API)
- **语言**: TypeScript
- **UI库**: Element Plus
- **状态管理**: Pinia
- **构建工具**: Vite
- **WebSocket**: 浏览器原生 WebSocket API

### 2.2 后端技术栈
- **语言**: Rust
- **Web框架**: Axum
- **WebSocket库**: Tokio + Axum WebSocket 支持
- **连接管理**: 自定义 GameConnectionManager
- **消息广播**: 自定义 MessageBroadcaster

### 2.3 WebSocket 消息协议

#### 2.3.1 认证消息
客户端连接时发送认证消息:
```json
{
  "user_type": "director",
  "password": "director_password"
}
```

服务端响应认证结果:
```json
{
  "type": "system_message",
  "data": {
    "message": "WebSocket connection established successfully"
  }
}
```

#### 2.3.2 初始状态消息
认证成功后，服务端发送初始状态消息:

导演视角:
```json
{
  "global_state": {
    "game_phase": "day",
    "weather": 1.0,
    "night_start_time": null,
    "night_end_time": null,
    "next_night_destroyed_places": []
  },
  "game_data": {
    "players": {
      "player_id_1": {
        "id": "player_id_1",
        "name": "玩家1",
        "location": "地点A",
        "life": 100,
        "strength": 100,
        // ... 其他玩家属性
      }
    },
    "places": {
      "地点A": {
        "name": "地点A",
        "players": ["player_id_1"],
        "items": [],
        "is_destroyed": false
      }
    }
  },
  "action_result": null
}
```

#### 2.3.3 实时更新消息
当游戏状态发生变化时，服务端广播更新消息，格式与初始状态消息相同，但包含 action_result 字段:

``json
{
  "global_state": {
    // 全局状态信息
  },
  "game_data": {
    // 游戏数据
  },
  "action_result": {
    "data": {
      // 动作结果数据
    },
    "log_message": "导演调整玩家生命值",
    "message_type": "system_notice",
    "timestamp": "2023-11-15T10:00:00Z"
  }
}
```

## 3. 前端组件设计

### 3.1 整体布局
```
+-------------------------------------------------------------+
|                    全局游戏状态区域                         |
+-------------------------------------------------------------+
|                                                             |
|  [天气设置] [夜晚开始时间] [夜晚结束时间] [缩圈地点设置]      |
|  [空投设置] [地点状态调整] [玩家状态调整] [广播消息]         |
|                                                             |
+-------------------------------------------------------------+
|                    实时日志消息区域                         |
+-------------------------------------------------------------+
|  [筛选面板: 日期筛选 | 演员筛选 | 关键词筛选]                |
+-------------------------------------------------------------+
|                                                             |
|  [日志消息列表]                                              |
|  - [消息1]                                                  |
|  - [消息2]                                                  |
|  - ...                                                     |
|  - [消息N]                                                  |
|                                                             |
|  [展开/折叠按钮]                                            |
|                                                             |
+-------------------------------------------------------------+
```

### 3.2 全局游戏状态组件

#### 3.2.1 组件功能
- 展示当前游戏的全局状态信息
- 提供交互控件以修改游戏状态
- 实时更新显示最新的全局状态

#### 3.2.2 状态展示项
- 当前游戏阶段（白天/夜晚）
- 天气条件值
- 夜晚开始时间
- 夜晚结束时间
- 下一夜晚缩圈地点集合

#### 3.2.3 交互控件
- 天气调节滑块
- 夜晚时间设置面板
- 缩圈地点选择器
- 空投设置面板（独立Vue组件，详见3.2.4）
- 地点状态切换开关
- 玩家状态调整表单
- 广播消息输入框（详见3.2.5）

### 3.2.4 空投设置面板（独立Vue组件）

#### 3.2.4.1 组件功能
- 允许输入若干个空投物品（物品允许重复输入相同的名称）
- 允许一键随机生成空投结果
- 生成后允许预览
- 预览后可以接受或拒绝

#### 3.2.4.2 组件设计
1. **物品输入区域**
   - 使用 `el-input` 组件输入物品名称
   - 允许添加多个物品（可重复）
   - 提供添加/删除物品按钮

2. **随机生成按钮**
   - 使用 `el-button` 组件
   - 点击后根据游戏规则随机生成空投物品

3. **预览区域**
   - 显示生成的空投物品列表
   - 使用 `el-table` 展示物品信息

4. **操作按钮**
   - 接受按钮：确认空投并发送到后端
   - 拒绝按钮：取消本次空投操作

#### 3.2.4.3 组件目录结构
- 组件文件路径：`frontend/src/views/director/components/AirdropPanel.vue`
- 组件将被 `InGameManagement.vue` 组件引用

#### 3.2.4.4 后端交互
- 通过 WebSocket 发送空投请求到后端
- 后端处理空投逻辑并在指定地点添加物品
- 后端返回操作结果和日志消息

### 3.2.5 广播消息输入框

#### 3.2.5.1 组件功能
- 向所有玩家发送消息
- 向特定玩家发送消息

#### 3.2.5.2 组件设计
1. **消息输入区域**
   - 使用 `el-input` 组件输入消息内容
   - 多行文本输入支持

2. **目标选择**
   - 广播到所有玩家选项
   - 选择特定玩家选项
   - 使用 `el-select` 和 `el-option` 组件实现玩家选择

3. **发送按钮**
   - 使用 `el-button` 组件
   - 触发消息发送逻辑

#### 3.2.5.3 后端交互
- 通过 WebSocket 发送广播消息请求到后端
- 后端根据目标类型处理消息广播
- 向所有玩家或特定玩家发送消息
- 后端返回操作结果和日志消息

### 3.3 实时日志消息组件

#### 3.3.1 组件功能
- 展示所有游戏日志消息
- 支持消息筛选功能
- 消息分类显示不同颜色
- 新消息动画效果
- 消息列表折叠/展开

#### 3.3.2 消息显示规范
- 按时间倒序排列（最新的在最上方）
- 系统消息使用蓝色标识
- 用户消息使用绿色标识
- 新消息添加时有渐变动画效果

#### 3.3.3 筛选功能
- 按日期筛选（开始日期、结束日期）
- 按相关演员名称筛选
- 按关键词筛选（模糊匹配消息内容）

#### 3.3.4 显示控制
- 默认只显示前20条消息
- 底部设置"显示全部"链接
- 全部展示后允许再折叠回来

## 4. WebSocket 连接管理

### 4.1 连接建立流程
1. 页面加载时，从前端路由参数获取 game_id
2. 从页面状态或API获取导演密码
3. 创建 WebSocket 连接到后端 `/ws/{game_id}` 端点
4. 发送认证消息
5. 等待认证结果和初始状态消息

### 4.2 消息处理流程
1. 接收服务端消息
2. 解析消息类型
3. 更新相应状态
4. 触发界面更新

### 4.3 连接维护
- 监听连接断开事件
- 实现重连机制
- 连接断开时提示用户

## 5. 状态管理设计

### 5.1 全局状态结构
``typescript
interface GameState {
  gamePhase: 'day' | 'night';
  weather: number;
  nightStartTime: string | null;
  nightEndTime: string | null;
  nextNightDestroyedPlaces: string[];
  players: Record<string, Player>;
  places: Record<string, Place>;
}

interface Player {
  id: string;
  name: string;
  location: string;
  life: number;
  strength: number;
  // 其他玩家属性
}

interface Place {
  name: string;
  players: string[];
  items: Item[];
  isDestroyed: boolean;
}

interface LogMessage {
  data: any;
  logMessage: string;
  messageType: 'system_notice' | 'user_directed';
  timestamp: string;
}
```

### 5.2 状态更新机制
1. 通过 WebSocket 接收状态更新
2. 使用 Pinia 管理全局状态
3. 组件通过响应式状态自动更新

## 6. UI/UX 设计规范

### 6.1 Element Plus 组件使用
- 使用 `el-card` 组织页面区域
- 使用 `el-form` 和 `el-form-item` 构建设置表单
- 使用 `el-table` 展示玩家和地点信息
- 使用 `el-input`, `el-select`, `el-date-picker` 等基础控件
- 使用 `el-button` 触发操作
- 使用 `el-alert` 显示系统提示

### 6.2 消息展示样式
- 系统消息背景色: #ecf5ff (蓝色)
- 用户消息背景色: #f0f9ff (绿色)
- 新消息动画: 渐变出现效果

### 6.3 响应式设计
- 适配不同屏幕尺寸
- 移动端优化布局

## 7. 其他

### 7.1 预计实现功能
1. WebSocket 连接管理
2. 全局游戏状态展示与修改
3. 实时日志消息展示与筛选
4. 消息分类显示与动画效果
5. 空投设置面板（独立Vue组件）
6. 广播消息输入框

### 7.2 后端API限制
1. 当前后端API已支持单个物品的空投功能（通过 `handle_drop` 方法），但不支持一次性投放多个物品
2. 需要后端扩展API以支持批量空投功能，因此空投物品的接口在前端实现中暂时留空待后续完善。
3. 广播消息功能已完整支持（通过 `handle_broadcast` 和 `handle_director_message_to_player` 方法）

### 7.3 前端实现建议
1. 空投设置面板需要在前端实现批量物品管理逻辑
2. 对于批量空投，等待后端扩展批量空投接口
3. 所有消息显示均使用简体中文，符合项目规范
