## Royale Arena 后端部署说明

本文档说明如何基于项目仓库提供的 Dockerfile、环境变量示例和 NGINX 反向代理配置，将 Royale Arena 后端服务部署到生产环境。

### 架构概览

- Rust 后端服务在 Docker 容器中运行，监听 `3000` 端口。
- MySQL 数据库用于存储游戏状态等业务数据。
- NGINX 作为反向代理，将公网请求转发到后端容器。
- 游戏状态文件通过卷挂载持久化到宿主机。

### 环境准备

1. **复制环境变量模板**
   ```bash
   cp backend/.env.example backend/.env
   ```
   根据实际环境修改以下关键项：
   - `DATABASE_URL`：指向生产数据库，确保账号具备建库建表权限。
   - `SERVER_PORT`：保持为 `3000`，除非 Docker Compose 或 NGINX 端口映射有特殊需求。
   - `API_PREFIX`：部署在子路径时可设置为 `/royale-arena`，裸域部署则使用 `/`。
   - `JWT_SECRET`：填写长度 ≥ 32 字节的随机字符串。
   - `RUST_LOG`：根据需要调整日志等级。

  如果你的 MySQL 实例中尚未创建用于本项目的数据库，请在数据库服务器或通过命令行手动创建（示例）：

  ```sql
  CREATE DATABASE royale_arena;
  ```

  并确保 `.env` 中的 `DATABASE_URL` 指向该数据库（例如：`mysql://user:password@host:3306/royale_arena`）。

2. **准备挂载目录**
   ```bash
   mkdir -p /srv/royale/game_states
   sudo chown 1001:1001 /srv/royale/game_states
   ```
   后端容器内的运行用户 UID 为 `1001`，请确保宿主机目录权限匹配。

### 构建后端镜像

项目提供的 `backend/Dockerfile` 采用多阶段构建，可直接构建发布镜像：

```bash
docker build -t royale-arena-rust-backend:0.1.1 ./backend
```

> 如需加速构建，可先在本地执行 `cargo build --release` 生成依赖缓存。

### Docker Compose 配置示例

以下片段可写入生产的 `docker-compose.yml`：

```yaml
services:
  royale:
    image: royale-arena-rust-backend:0.1.1
    container_name: royale_arena
    environment:
      TZ: Asia/Shanghai
    volumes:
      - /srv/royale/game_states:/srv/app/game_states
    depends_on:
      - mysql
    networks:
      - discuz_net
    env_file:
      - ./backend/.env
```

> 可加入 `ports: - 3000:3000` 直接暴露端口。

### NGINX 反向代理配置

将以下配置加入站点配置文件（例如 `nginx.conf` 或 `/etc/nginx/conf.d/royale.conf`）：

```nginx
map $http_upgrade $connection_upgrade {
    default upgrade;
    ''      close;
}

server {
    listen 443 ssl;

    location ^~ /royale-arena/api/ {
        proxy_http_version 1.1;
        proxy_pass http://royale:3000/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
        proxy_connect_timeout 5s;
        proxy_read_timeout 3600s;
    }
}
```

说明：`proxy_pass` 指向 Docker Compose 中定义的服务名 `royale`（容器内部监听端口为 `3000`），因此在 `.env` 中的 `SERVER_PORT` 与容器端口应保持一致。

### 前端部署（静态资源）

前端由 Vite 构建为静态文件（`dist`），将构建产物放到 NGINX 的静态目录并配置 SPA 回退。子路径 `/royale-arena/` 的 NGINX 示例：

```nginx
location ^~ /royale-arena/ {
    alias /var/www/html/royale-arena/current/;
    index index.html;
    try_files $uri $uri/ /index.html;
    add_header Cache-Control "public, max-age=60";
}
```

构建步骤（在构建机/CI 中）：

```bash
cd frontend
pnpm install --frozen-lockfile
pnpm build

cd backend
cargo sqlx prepare
```

把 `dist` 放到 NGINX 的静态目录，或在 CI 中将其打包进 NGINX 镜像。若部署在子路径，请在 `vite.config.ts` 设置 `base: '/royale-arena/'` 并重建。

### 启动与运维

1. 启动容器：
   ```bash
   docker compose up -d royale
   ```
2. 检查日志：
  ```bash
  docker logs -f royale_arena
  ```
3. 版本发布：
   - `cargo sqlx prepare` 更新编译预备数据。
   - 更新代码并重新构建镜像（修改标签或覆盖旧版本）。
   - `docker compose pull && docker compose up -d royale` 以滚动后端更新。
   - `ln -sfn ./vX.X.X ./current` 更新软链接current完成前端更新。

### 常见问题

- **SQLx 运行时报错 “database does not exist”**：确认数据库初始化成功后重新启动后端容器。
- **WebSocket 连接失败**：检查 NGINX 配置的 `map` 块是否生效，以及 `proxy_set_header Upgrade` 是否正确传递。
- **静态资源跨域**：确保前端部署的域名与后端一致，或在 NGINX 中增加 CORS 头部。
