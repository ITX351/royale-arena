# Royale Arena

Royale Arena 是一个基于网页的大逃杀游戏，玩家在限定时间内执行行动，包括移动、搜索、攻击和使用道具等。

## 技术架构

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust 语言开发
- **部署**: Nginx 服务器

## 核心特性

- 实时大逃杀游戏体验
- 玩家可在特定时间段内执行行动
- 导演可以控制游戏进程和规则
- 响应式网页界面，支持多设备访问

## 游戏规则参考

本项目基于**雾雨小镇狼人村**大逃杀规则进行开发：
- [『黑色幸存者』——规则帖](https://www.mistytown.cn/forum.php?mod=viewthread&tid=12353&fromuid=9472)

## 开发

```bash
# 前端开发
cd frontend
npm install
npm run dev

# 后端开发
cd backend
cargo run

# 初始化管理员帐户脚本（硬编码用户名及密码）
cargo run --bin init_admin_users
```

## 文档

- [游戏规则](docs/game-rules.md)
- [API规范](docs/api-spec.md)
- [项目总览](QWEN.md)
- [前端上下文](frontend/QWEN.md)
- [后端上下文](backend/QWEN.md)

## 相关项目

本项目是对旧版导演工具的升级和重构：
- [旧版工程](https://github.com/lydrainbowcat/directors/) - 随机身份分发器

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

作者：ITX351, lydrainbowcat