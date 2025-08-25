# Royale Arena API 接口规范

## 概述

本文档定义了 Royale Arena 游戏系统的 API 接口规范，包括 REST API 和 WebSocket 接口。REST API 主要用于管理员认证和游戏状态查询，WebSocket 接口用于处理玩家的实时操作指令和导演控制指令。

## 认证方式

### 管理员认证
- 系统支持多个管理员账户
- 管理员凭据存储在 MySQL 数据库中
- 管理员通过 API 登录验证获取访问令牌

需要认证的API请求必须在Header中包含认证令牌：
```http
Authorization: Bearer <token>
```

### 需要认证的接口
以下接口需要管理员认证：
- 创建游戏
- 修改游戏设置
- 删除游戏
- 获取敏感游戏信息

以下接口无需认证：
- `/api/games` - 获取游戏列表
- `/api/game/{game_id}` - 获取游戏基本信息（公开信息）
- `/api/game/{game_id}/rules` - 获取游戏规则

### 导演和玩家认证
- 导演使用游戏管理密码访问导演控制台
- 玩家使用分配的演员密码登录游戏
- 导演和玩家通过 URI 中的密码参数直接访问，无需 API 登录
- 为了简化用户体验并允许用户收藏链接，此方式不加安全防护

### 认证方式总结
- 管理员API需要Bearer Token认证：`Authorization: Bearer <token>`
- 游戏相关API（导演/玩家）通过URL参数认证：`?password=<password>`
- WebSocket连接通过连接参数认证