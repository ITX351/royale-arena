# Royale Arena 环境配置说明

## 概述

本项目支持通过环境变量进行配置，以适应不同的部署环境（开发、生产等）。

## 部署架构

### 生产环境路径规划

- **根路径 `/`** - Discuz论坛网站（保持不变）
- **前端路径 `/royale-arena/`** - Vue前端应用（CSR，静态文件部署）
- **后端API路径 `/royale-arena/api/`** - Rust后端服务（通过Nginx转发到容器）

### Nginx转发配置

```nginx
# 前端静态文件
location /royale-arena/ {
    alias /path/to/frontend/dist/;
    try_files $uri $uri/ /royale-arena/index.html;
}

# 后端API转发
location /royale-arena/api/ {
    proxy_pass http://127.0.0.1:3000/royale-arena/api/;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
}
```

## 环境配置

### 开发环境

**后端配置** (`.env`):
```env
DATABASE_URL=mysql://root:password@localhost:3306/royale_arena
SERVER_PORT=3000
API_PREFIX=/royale-arena/api
JWT_SECRET=your-dev-jwt-secret
JWT_EXPIRATION_HOURS=24
BCRYPT_COST=12
RUST_LOG=debug
```

**前端配置**:
- 开发服务器: `http://localhost:5173` 或 `http://localhost:5174`
- API代理: `/royale-arena/api` -> `http://localhost:3000`

**访问地址**:
- 前端: `http://localhost:5173/`
- 后端API: `http://localhost:3000/royale-arena/api/`

### 生产环境

**后端配置** (生产环境 `.env`):
```env
DATABASE_URL=mysql://prod_user:secure_pass@prod_db:3306/royale_arena
SERVER_PORT=3000
API_PREFIX=/royale-arena/api
JWT_SECRET=production-256bit-secure-key
JWT_EXPIRATION_HOURS=24
BCRYPT_COST=14
RUST_LOG=info
```

**Nginx配置示例** (详细配置参见 `docs/nginx-production.conf`):
```nginx
server {
    listen 80;
    server_name your-domain.com;
    
    # 根路径 - Discuz论坛（保持不变）
    location / {
        # 您现有的Discuz配置
    }
    
    # 前端静态文件
    location /royale-arena/ {
        alias /path/to/frontend/dist/;
        try_files $uri $uri/ /royale-arena/index.html;
    }
    
    # 后端API代理
    location /royale-arena/api/ {
        proxy_pass http://127.0.0.1:3000/royale-arena/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## API端点路径

### 开发环境
- 健康检查: `http://localhost:3000/health`
- 游戏列表: `http://localhost:3000/royale-arena/api/games`
- 管理员登录: `http://localhost:3000/royale-arena/api/admin/login`

### 生产环境
- 健康检查: `http://your-domain.com/health`
- 游戏列表: `http://your-domain.com/royale-arena/api/games`
- 管理员登录: `http://your-domain.com/royale-arena/api/admin/login`

## 环境变量说明

| 变量名 | 说明 | 默认值 | 示例 |
|--------|------|--------|------|
| `DATABASE_URL` | MySQL数据库连接字符串 | 无 | `mysql://user:pass@host:3306/db` |
| `SERVER_PORT` | 后端服务器监听端口 | `3000` | `8080` |
| `API_PREFIX` | API路径前缀 | `/royale-arena/api` | `/royale-arena/api` |
| `JWT_SECRET` | JWT签名密钥 | 无 | `your-256bit-secret-key` |
| `JWT_EXPIRATION_HOURS` | JWT过期时间（小时） | `24` | `24` |
| `BCRYPT_COST` | 密码哈希成本 | `12` | `14` |
| `RUST_LOG` | 日志级别 | `info` | `debug`, `info`, `warn` |

## 部署步骤

### 1. 后端部署

```bash
# 1. 复制生产环境配置
cp .env.production .env

# 2. 修改配置文件中的数据库连接等信息
vi .env

# 3. 构建生产版本
cargo build --release

# 4. 启动服务
./target/release/royale-arena-backend
```

### 2. 前端部署

```bash
# 1. 构建生产版本
pnpm build

# 2. 部署到Web服务器
cp -r dist/* /var/www/royale-arena/frontend/
```

## 测试配置

启动服务后，可以通过以下方式测试配置是否正确：

```bash
# 测试健康检查
curl http://localhost:3000/health

# 测试API端点
curl http://localhost:3000/royale-arena/api/games
```

## 注意事项

1. **安全性**: 生产环境请使用强密钥和安全的数据库配置
2. **端口配置**: 确保防火墙已开放相应端口
3. **数据库**: 确保数据库用户具有适当的权限
4. **SSL**: 生产环境建议配置HTTPS
5. **日志**: 生产环境建议使用 `info` 或 `warn` 日志级别