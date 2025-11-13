# Royale Arena

Royale Arena 是一个基于网页的大逃杀游戏，玩家在限定时间内执行行动，包括移动、搜索、攻击和使用道具等。

## 技术架构

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust
- **部署**: Nginx

## 游戏规则参考

本项目基于**雾雨小镇狼人村**大逃杀规则进行开发：
- [『黑色幸存者』——规则帖](https://www.mistytown.cn/forum.php?mod=viewthread&tid=12353&fromuid=9472)

## 开发

```bash
# 前端开发
cd frontend
pnpm install
pnpm dev

# 后端开发
cd backend
cargo run

# 发布前预准备编译数据
cargo sqlx prepare
```

## 部署

详细步骤参考 `DEPLOYMENT.md`，涵盖环境变量、镜像构建、Docker Compose 与 NGINX 配置。
- 部署前端：在 `frontend` 目录运行 `pnpm build`，将生成的 `dist` 部署到 NGINX 静态目录。
- 复制 `backend/.env.example` 为 `.env` 并配置数据库、JWT、API 前缀等关键项。
- 通过 `docker build -t royale-arena-rust-backend:0.4.2 ./backend` 构建后端镜像，使用 Compose 挂载 `/srv/royale/game_states` 目录并注入 `.env`。
- 在 NGINX 中创建 `location ^~ /royale-arena/api/`，将请求代理至 `royale:3000` 并保留 WebSocket 头。
- 首次部署按顺序启动数据库与后端容器，使用 `docker logs -f royale_arena` 观察运行状态。

## 访问使用

目前该项目已在[雾雨小镇](https://www.mistytown.cn/)服务器部署。

访问以下链接以使用：[https://www.mistytown.cn/royale-arena/](https://www.mistytown.cn/royale-arena/)

## 相关项目

本项目是对旧版导演工具的升级和重构：
- [旧版工程](https://github.com/lydrainbowcat/directors/) - 随机身份分发器

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

作者：ITX351, lydrainbowcat