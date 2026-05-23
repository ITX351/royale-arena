# API端点参考

<cite>
**本文档中引用的文件**   
- [routes.rs](file://backend\src\routes.rs) - *更新了管理端路由*
- [handlers.rs](file://backend\src\admin\handlers.rs) - *新增管理员处理逻辑*
- [models.rs](file://backend\src\admin\models.rs) - *定义管理端数据模型*
- [service.rs](file://backend\src\admin\service.rs) - *实现管理员服务逻辑*
- [middleware.rs](file://backend\src\auth\middleware.rs) - *权限验证中间件*
- [auth.rs](file://backend\src\auth\service.rs) - *认证服务实现*
- [jwt.rs](file://backend\src\auth\jwt.rs) - *JWT令牌管理*
- [errors.rs](file://backend\src\errors.rs) - *错误处理定义*
- [director/handlers.rs](file://backend\src\director\handlers.rs) - *新增导演接口处理逻辑*
- [director/models.rs](file://backend\src\director\models.rs) - *定义导演接口数据模型*
- [director/errors.rs](file://backend\src\director\errors.rs) - *导演模块错误类型定义*
- [director/service.rs](file://backend\src\director\service.rs) - *实现导演服务逻辑*
</cite>

## 更新摘要
**已更新内容**
- 更新了简介部分，反映Rust后端实现
- 重写了项目结构描述，准确反映Rust代码组织
- 新增了管理端API详细文档
- 更新了架构概述，反映JWT认证机制
- 添加了详细的权限控制策略说明
- 新增了请求/响应示例和curl命令
- **新增导演模块API文档，包括批量添加、查询和删除演员接口**

**新增部分**
- 管理员认证API
- 管理员用户管理CRUD操作
- JWT认证机制详细说明
- 权限控制策略
- 错误响应码说明
- **导演模块API：批量添加演员、获取演员列表、批量删除演员**

**已弃用内容**
- 移除了基于Python Flask的旧架构描述
- 移除了Base64编码密钥的认证机制

## 目录
1. [简介](#简介)
2. [项目结构](#项目结构)
3. [核心组件](#核心组件)
4. [架构概述](#架构概述)
5. [管理端API](#管理端api)
6. [导演模块API](#导演模块api)
7. [认证与权限](#认证与权限)
8. [错误处理](#错误处理)
9. [使用示例](#使用示例)

## 简介
本API文档详细描述了Royale Arena后端系统的RESTful接口。该系统是一个基于Rust和Axum框架的高性能游戏后端，支持管理员认证、用户管理等核心功能。API采用JWT（JSON Web Token）进行用户身份验证，通过Bearer Token机制实现安全的API访问控制。系统实现了严格的权限分级，区分普通管理员和超级管理员角色，确保敏感操作的安全性。文档涵盖了所有API端点的HTTP方法、URL路径、请求参数、响应结构和状态码，并提供了详细的使用示例。

## 项目结构
Royale Arena项目采用Rust作为主要后端语言，基于Axum Web框架构建。后端代码位于`backend/src`目录，采用模块化设计。核心模块包括`admin`（管理员管理）、`auth`（认证授权）、`database`（数据库访问）和`errors`（错误处理）。系统使用SQLx进行异步数据库操作，通过JWT实现安全的认证机制。代码结构清晰，遵循Rust最佳实践，各模块职责分明。

```
mermaid
graph TB
subgraph "后端模块"
admin[admin模块\n管理员管理]
auth[auth模块\n认证授权]
database[database模块\n数据访问]
errors[errors模块\n错误处理]
end
admin --> auth
admin --> database
admin --> errors
auth --> database
auth --> errors
```

**图源**
- [routes.rs](file://backend\src\routes.rs)
- [lib.rs](file://backend\src\lib.rs)

**本节源**
- [src目录](file://backend\src)
- [Cargo.toml](file://backend\Cargo.toml)

## 核心组件
Royale Arena后端的核心组件包括JWT认证系统、管理员管理服务、数据库访问层和统一错误处理机制。JWT认证系统负责用户登录和令牌生成验证。管理员管理服务提供用户创建、读取、更新和删除（CRUD）操作。数据库访问层使用SQLx实现类型安全的异步数据库查询。错误处理机制通过自定义错误类型提供清晰的错误信息和适当的HTTP状态码。

**本节源**
- [auth模块](file://backend\src\auth)
- [admin模块](file://backend\src\admin)
- [database.rs](file://backend\src\database.rs)
- [errors.rs](file://backend\src\errors.rs)

## 架构概述
Royale Arena采用基于Rust和Axum的微服务架构，所有API端点通过路由注册。系统使用JWT进行用户认证，客户端在请求头中携带Bearer Token进行身份验证。权限控制通过中间件实现，超级管理员权限的API端点需要额外的权限验证。数据持久化使用关系型数据库，通过SQLx进行异步操作。错误处理采用分层设计，将底层数据库错误转换为用户友好的HTTP响应。

```
mermaid
graph TD
Client[客户端] --> |HTTP请求| Server[API服务器]
Server --> Auth[JWT认证]
Auth --> |验证| Token[Bearer Token]
Token --> |有效| Permission[权限检查]
Permission --> |超级管理员| AdminAPI[管理端API]
Permission --> |普通管理员| UserAPI[用户API]
AdminAPI --> DB[数据库]
UserAPI --> DB
DB --> Server
Server --> |JSON响应| Client
```

**图源**
- [routes.rs](file://backend\src\routes.rs)
- [middleware.rs](file://backend\src\auth\middleware.rs)
- [service.rs](file://backend\src\admin\service.rs)

## 管理端API
管理端API提供管理员用户的认证和管理功能，所有敏感操作都需要超级管理员权限。

### 管理员登录
管理员通过用户名和密码进行认证，成功后返回JWT令牌。

**端点**: `POST /api/admin/login`  
**认证**: 无需认证  
**权限**: 所有用户

**请求体**:
```json
{
  "username": "string",
  "password": "string"
}
```

**响应**:
```json
{
  "success": true,
  "token": "string",
  "expires_in": 86400
}
```

**状态码**:
- `200 OK`: 登录成功
- `401 Unauthorized`: 认证失败

**本节源**
- [handlers.rs](file://backend\src\admin\handlers.rs#L10-L20)
- [models.rs](file://backend\src\admin\models.rs#L45-L55)

### 获取管理员列表
获取所有管理员用户列表，仅超级管理员可访问。

**端点**: `GET /api/admin/users`  
**认证**: Bearer Token  
**权限**: 超级管理员

**响应**:
```json
{
  "users": [
    {
      "id": "string",
      "username": "string",
      "is_super_admin": false
    }
  ]
}
```

**状态码**:
- `200 OK`: 获取成功
- `401 Unauthorized`: 未认证
- `403 Forbidden`: 权限不足

**本节源**
- [handlers.rs](file://backend\src\admin\handlers.rs#L23-L32)
- [service.rs](file://backend\src\admin\service.rs#L15-L25)

### 创建管理员
创建新的管理员账户，仅超级管理员可访问。

**端点**: `POST /api/admin/users`  
**认证**: Bearer Token  
**权限**: 超级管理员

**请求体**:
```json
{
  "username": "string",
  "password": "string",
  "is_super_admin": false
}
```

**响应**:
```json
{
  "success": true,
  "message": "Admin user created successfully",
  "user": {
    "id": "string",
    "username": "string",
    "is_super_admin": false
  }
}
```

**状态码**:
- `200 OK`: 创建成功
- `400 Bad Request`: 请求数据无效
- `401 Unauthorized`: 未认证
- `403 Forbidden`: 权限不足
- `409 Conflict`: 用户已存在

**本节源**
- [handlers.rs](file://backend\src\admin\handlers.rs#L35-L47)
- [service.rs](file://backend\src\admin\service.rs#L27-L65)

### 更新管理员
更新管理员账户信息，仅超级管理员可访问。

**端点**: `PUT /api/admin/users/:user_id`  
**认证**: Bearer Token  
**权限**: 超级管理员

**路径参数**:
- `user_id`: 要更新的管理员ID

**请求体**:
```json
{
  "username": "string",
  "password": "string",
  "is_super_admin": false
}
```

**响应**:
```json
{
  "success": true,
  "message": "Admin user updated successfully",
  "user": {
    "id": "string",
    "username": "string",
    "is_super_admin": false
  }
}
```

**状态码**:
- `200 OK`: 更新成功
- `400 Bad Request`: 请求数据无效
- `401 Unauthorized`: 未认证
- `403 Forbidden`: 权限不足
- `404 Not Found`: 用户不存在

**本节源**
- [handlers.rs](file://backend\src\admin\handlers.rs#L50-L65)
- [service.rs](file://backend\src\admin\service.rs#L67-L125)

### 删除管理员
删除管理员账户，仅超级管理员可访问。

**端点**: `DELETE /api/admin/users/:user_id`  
**认证**: Bearer Token  
**权限**: 超级管理员

**路径参数**:
- `user_id`: 要删除的管理员ID

**响应**:
```json
{
  "success": true,
  "message": "Admin user deleted successfully"
}
```

**状态码**:
- `200 OK`: 删除成功
- `401 Unauthorized`: 未认证
- `403 Forbidden`: 权限不足
- `404 Not Found`: 用户不存在

**本节源**
- [handlers.rs](file://backend\src\admin\handlers.rs#L68-L80)
- [service.rs](file://backend\src\admin\service.rs#L127-L171)

## 导演模块API
导演模块API提供对游戏演员的管理功能，通过导演密码进行身份验证，无需JWT令牌。

### 批量添加演员
导演可以批量添加演员到指定游戏中。

**端点**: `POST /api/game/{game_id}/players?password={director_password}`  
**认证**: 导演密码（查询参数）  
**权限**: 游戏导演

**路径参数**:
- `game_id`: 目标游戏ID

**查询参数**:
- `password`: 导演密码

**请求体**:
```json
{
  "players": [
    {
      "player_name": "string",
      "password": "string",
      "team_id": 0
    }
  ]
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "success": [
      {
        "id": "string",
        "name": "string",
        "password": "string",
        "game_id": "string",
        "team_id": 0
      }
    ],
    "failed": [
      {
        "player_name": "string",
        "id": null,
        "reason": "string"
      }
    ]
  }
}
```

**状态码**:
- `200 OK`: 操作成功（可能部分成功）
- `400 Bad Request`: 请求数据无效
- `401 Unauthorized`: 导演密码错误
- `404 Not Found`: 游戏不存在
- `409 Conflict`: 游戏已开始，无法添加演员

**本节源**
- [handlers.rs](file://backend\src\director\handlers.rs#L20-L35)
- [models.rs](file://backend\src\director\models.rs#L10-L45)
- [service.rs](file://backend\src\director\service.rs#L100-L150)

### 获取演员列表
获取指定游戏中的所有演员列表。

**端点**: `GET /api/game/{game_id}/players?password={director_password}`  
**认证**: 导演密码（查询参数）  
**权限**: 游戏导演

**路径参数**:
- `game_id`: 目标游戏ID

**查询参数**:
- `password`: 导演密码

**响应**:
```json
{
  "success": true,
  "data": {
    "players": [
      {
        "id": "string",
        "name": "string",
        "password": "string",
        "game_id": "string",
        "team_id": 0
      }
    ]
  }
}
```

**状态码**:
- `200 OK`: 获取成功
- `401 Unauthorized`: 导演密码错误
- `404 Not Found`: 游戏不存在

**本节源**
- [handlers.rs](file://backend\src\director\handlers.rs#L37-L50)
- [models.rs](file://backend\src\director\models.rs#L50-L65)
- [service.rs](file://backend\src\director\service.rs#L152-L170)

### 批量删除演员
导演可以批量删除指定游戏中的演员。

**端点**: `DELETE /api/game/{game_id}/players?password={director_password}`  
**认证**: 导演密码（查询参数）  
**权限**: 游戏导演

**路径参数**:
- `game_id`: 目标游戏ID

**查询参数**:
- `password`: 导演密码

**请求体**:
```json
{
  "player_ids": ["string", "string"]
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "success": [
      {
        "id": "string",
        "name": "string",
        "message": "Player deleted successfully"
      }
    ],
    "failed": [
      {
        "player_name": null,
        "id": "string",
        "reason": "string"
      }
    ]
  }
}
```

**状态码**:
- `200 OK`: 操作成功（可能部分成功）
- `400 Bad Request`: 请求数据无效
- `401 Unauthorized`: 导演密码错误
- `404 Not Found`: 游戏或演员不存在
- `409 Conflict`: 游戏已开始，无法删除演员

**本节源**
- [handlers.rs](file://backend\src\director\handlers.rs#L52-L67)
- [models.rs](file://backend\src\director\models.rs#L70-L90)
- [service.rs](file://backend\src\director\service.rs#L172-L250)

## 认证与权限
系统采用JWT（JSON Web Token）进行用户认证和权限管理。

### 认证流程
1. 管理员通过`/api/admin/login`端点提交用户名和密码
2. 服务器验证凭据，成功后生成JWT令牌
3. 客户端在后续请求的`Authorization`头中携带`Bearer <token>`
4. 服务器通过中间件验证令牌的有效性

### 权限控制
系统实现两级权限控制：
- **普通管理员**: 可以登录系统
- **超级管理员**: 可以管理其他管理员账户

权限验证通过中间件实现：
- `jwt_auth_middleware`: 验证JWT令牌的有效性
- `super_admin_middleware`: 验证用户是否为超级管理员

```
mermaid
sequenceDiagram
participant Client as 客户端
participant Server as 服务器
participant Auth as 认证服务
Client->>Server : POST /api/admin/login
Server->>Auth : 验证用户名密码
Auth->>Server : 返回JWT令牌
Server->>Client : 返回令牌
Client->>Server : GET /api/admin/users<br/>Authorization : Bearer <token>
Server->>Server : 验证JWT令牌
Server->>Server : 检查超级管理员权限
Server->>Server : 执行业务逻辑
Server->>Client : 返回管理员列表
```

**图源**
- [auth.rs](file://backend\src\auth\service.rs)
- [middleware.rs](file://backend\src\auth\middleware.rs)
- [jwt.rs](file://backend\src\auth\jwt.rs)

**本节源**
- [auth模块](file://backend\src\auth)
- [models.rs](file://backend\src\admin\models.rs#L70-L90)

## 错误处理
系统提供统一的错误响应格式和适当的HTTP状态码。

### 错误响应格式
```json
{
  "success": false,
  "error": {
    "message": "错误信息",
    "details": "详细信息"
  }
}
```

### 状态码映射
| HTTP状态码 | 错误类型 | 描述 |
|-----------|---------|------|
| `400` | Bad Request | 请求数据无效 |
| `401` | Unauthorized | 认证失败 |
| `403` | Forbidden | 权限不足 |
| `404` | Not Found | 资源不存在 |
| `409` | Conflict | 资源冲突 |
| `500` | Internal Server Error | 服务器内部错误 |

### 具体错误
- **用户名或密码错误**: `401 Unauthorized`
- **认证令牌已过期**: `401 Unauthorized`
- **认证令牌无效**: `401 Unauthorized`
- **权限不足**: `403 Forbidden`
- **用户已存在**: `409 Conflict`
- **不能删除超级管理员**: `403 Forbidden`
- **导演密码错误**: `401 Unauthorized`
- **游戏不存在**: `404 Not Found`
- **游戏已开始，无法删除演员**: `409 Conflict`

**本节源**
- [errors.rs](file://backend\src\errors.rs)
- [middleware.rs](file://backend\src\auth\middleware.rs)
- [director/errors.rs](file://backend\src\director\errors.rs)

## 使用示例
### 管理员登录
```bash
curl -X POST http://localhost:8000/api/admin/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "superadmin",
    "password": "password123"
  }'
```

### 获取管理员列表
```bash
curl -X GET http://localhost:8000/api/admin/users \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### 创建管理员
```bash
curl -X POST http://localhost:8000/api/admin/users \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "username": "newadmin",
    "password": "password123",
    "is_super_admin": false
  }'
```

### 批量添加演员
```bash
curl -X POST http://localhost:8000/api/game/game123/players?password=director123 \
  -H "Content-Type: application/json" \
  -d '{
    "players": [
      {
        "player_name": "演员1",
        "password": "abc123",
        "team_id": 1
      },
      {
        "player_name": "演员2",
        "password": "def456",
        "team_id": 2
      }
    ]
  }'
```

### 获取演员列表
```bash
curl -X GET http://localhost:8000/api/game/game123/players?password=director123
```

### 批量删除演员
```bash
curl -X DELETE http://localhost:8000/api/game/game123/players?password=director123 \
  -H "Content-Type: application/json" \
  -d '{
    "player_ids": ["player1", "player2"]
  }'
```

### 字段验证规则
- **用户名**: 必填，长度3-50字符，唯一
- **密码**: 必填，长度6-100字符
- **is_super_admin**: 可选，布尔值，默认false
- **演员名称**: 必填，长度1-50字符，不能为空
- **演员密码**: 必填，长度6-8位，只能包含字母和数字
- **队伍ID**: 可选，整数，不能为负数，默认为0

**本节源**
- [models.rs](file://backend\src\admin\models.rs)
- [service.rs](file://backend\src\admin\service.rs)
- [handlers.rs](file://backend\src\admin\handlers.rs)
- [director/models.rs](file://backend\src\director\models.rs)
- [director/service.rs](file://backend\src\director\service.rs)
- [director/handlers.rs](file://backend\src\director\handlers.rs)