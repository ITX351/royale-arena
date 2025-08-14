# 管理员账户加密存储实现

## 概述

实现了管理员账户的加密存储和验证功能，使用 bcrypt 对密码进行加密处理。

## 功能组件

### 初始化脚本

- 创建了 `init_admin_users.rs` 二进制文件，用于初始化管理员账户
- 自动创建 `admin_users` 数据表
- 添加默认管理员账户并使用 bcrypt 加密密码

### 服务模块

实现了 `admin_service.rs` 模块，提供以下核心功能：

- `create_admin_user`: 创建新的管理员账户，自动对密码进行 bcrypt 加密
- `verify_admin_password`: 验证管理员登录密码
- `get_admin_user`: 根据用户名获取管理员信息

## 安全特性

- 密码使用 bcrypt 加密存储
- 数据库连接信息从环境变量加载
- 实现了完整的错误处理机制
