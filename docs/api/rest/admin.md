# 管理员接口

系统管理员相关的 API 接口，用于管理游戏实例和系统级配置。

## 接口列表

### 1. 管理员登录验证
```
POST /api/admin/login
```

**请求参数:**
```json
{
  "username": "string",  // 管理员用户名
  "password": "string"   // 管理员密码
}
```

**响应:**
```json
{
  "success": true,
  "token": "string",     // 认证令牌
  "expires_in": "integer" // 令牌过期时间（秒）
}
```


### 2. 创建游戏
```
POST /api/admin/games
```

**请求参数:**
```json
{
  "name": "string",
  "description": "string",
  "director_password": "string",
  "max_players": "integer"
}
```

**响应:**
```json
{
  "id": "string",
  "name": "string",
  "description": "string",
  "director_password": "string",
  "max_players": "integer"
}
```

### 3. 修改游戏设置
```
PUT /api/admin/game/{game_id}
```

**路径参数:**
- `game_id`: 游戏ID

**请求参数:**
```json
{
  "name": "string",           // 可选
  "description": "string",    // 可选
  "director_password": "string",  // 可选
  "max_players": "integer"    // 可选
}
```

**响应:**
```json
{
  "id": "string",
  "name": "string",
  "description": "string",
  "director_password": "string",
  "max_players": "integer"
}
```

### 4. 删除游戏
```
DELETE /api/admin/game/{game_id}
```

**路径参数:**
- `game_id`: 游戏ID

**响应:**
```json
{
  "success": true,
  "message": "Game deleted successfully"
}
```

### 5. 创建游戏规则模版
```
POST /api/admin/rule-templates
```

**请求参数:**
```json
{
  "template_name": "string",      // 模版名称
  "description": "string",        // 模版描述
  "is_active": "boolean",         // 模版是否激活，默认为true
  "rules_config": {}              // 完整的游戏规则配置
}
```

**响应:**
```json
{
  "id": "string",
  "template_name": "string",
  "description": "string",
  "is_active": "boolean",
  "rules_config": {},             // 完整的游戏规则配置
  "created_at": "ISO8601 datetime",
  "updated_at": "ISO8601 datetime"
}
```

### 6. 修改游戏规则模版
```
PUT /api/admin/rule-templates/{template_id}
```

**路径参数:**
- `template_id`: 模版ID

**请求参数:**
```json
{
  "template_name": "string",      // 可选
  "description": "string",        // 可选
  "is_active": "boolean",         // 可选
  "rules_config": {}              // 可选，完整的游戏规则配置
}
```

**响应:**
```json
{
  "id": "string",
  "template_name": "string",
  "description": "string",
  "is_active": "boolean",
  "rules_config": {},             // 完整的游戏规则配置
  "created_at": "ISO8601 datetime",
  "updated_at": "ISO8601 datetime"
}
```

### 7. 删除游戏规则模版
```
DELETE /api/admin/rule-templates/{template_id}
```

**路径参数:**
- `template_id`: 模版ID

**响应:**
```json
{
  "success": true,
  "message": "Rule template deleted successfully"
}
```

### 8. 获取管理员列表（仅超级管理员）
```
GET /api/admin/users
```

**查询参数:**
- `Authorization`: Bearer token (超级管理员认证令牌)

**响应:**
```json
{
  "users": [
    {
      "id": "string",
      "username": "string",
      "is_super_admin": "boolean"
    }
  ]
}
```

### 9. 创建管理员账户（仅超级管理员）
```
POST /api/admin/users
```

**查询参数:**
- `Authorization`: Bearer token (超级管理员认证令牌)

**请求参数:**
```json
{
  "username": "string",
  "password": "string",
  "is_super_admin": "boolean"  // 可选，默认为false
}
```

**响应:**
```json
{
  "success": true,
  "message": "Admin user created successfully",
  "user": {
    "id": "string",
    "username": "string",
    "is_super_admin": "boolean"
  }
}
```

### 10. 修改管理员账户（仅超级管理员）
```
PUT /api/admin/users/{user_id}
```

**路径参数:**
- `user_id`: 管理员用户ID

**查询参数:**
- `Authorization`: Bearer token (超级管理员认证令牌)

**请求参数:**
```json
{
  "username": "string",        // 可选
  "password": "string",        // 可选
  "is_super_admin": "boolean"  // 可选
}
```

**响应:**
```json
{
  "success": true,
  "message": "Admin user updated successfully",
  "user": {
    "id": "string",
    "username": "string",
    "is_super_admin": "boolean"
  }
}
```

### 11. 删除管理员账户（仅超级管理员）
```
DELETE /api/admin/users/{user_id}
```

**路径参数:**
- `user_id`: 管理员用户ID

**查询参数:**
- `Authorization`: Bearer token (超级管理员认证令牌)

**响应:**
```json
{
  "success": true,
  "message": "Admin user deleted successfully"
}
```