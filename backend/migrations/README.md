# Royale Arena 数据库迁移指南

本文档说明了如何使用 `sqlx` 工具和迁移文件来管理 Royale Arena 项目的数据库。

## 先决条件

1.  **安装 `sqlx-cli`**: 确保你已经安装了 `sqlx` 命令行工具。
    ```bash
    cargo install --features postgres,mysql sqlx-cli
    ```
    (确保包含了 `mysql` 特性)
2.  **配置环境变量**: 项目根目录提供了一个 `.env.example` 文件。请将其复制为 `.env` 并根据你的本地环境修改其中的值，特别是 `DATABASE_URL`。

## 迁移文件

迁移文件位于 `backend/migrations/` 目录下。每个迁移包含两个文件：
*   `{timestamp}_{name}.up.sql`: 定义如何应用此迁移（例如，创建表、插入数据）。
*   `{timestamp}_{name}.down.sql`: 定义如何回滚此迁移（例如，删除表、删除数据）。

当前迁移：
1.  `..._init_schema.up.sql`: 创建数据库表结构。
2.  `..._seed_test_data.up.sql`: 填充初始测试数据。

## 工作流

### 1. 创建新的迁移

当你需要修改数据库结构（如添加新表、修改列）或添加新的初始数据时，应创建一个新的迁移。

在 `backend/` 目录下运行：

```bash
sqlx migrate add -r 名称描述
```

*   `-r` 标志表示这是一个可重复运行的脚本（适用于 `CREATE IF NOT EXISTS` 或 `INSERT IGNORE`）。
*   例如：`sqlx migrate add -r add_player_status_column`

这会生成两个新的文件，你需要在 `.up.sql` 中编写应用变更的 SQL，在 `.down.sql` 中编写回滚的 SQL。

### 2. 应用迁移

要将所有未应用的迁移应用到 `.env` 文件中 `DATABASE_URL` 指定的数据库，请运行：

```bash
sqlx migrate run
```

这会按时间戳顺序执行所有尚未运行的 `.up.sql` 脚本。请确保在运行此命令前，`.env` 文件已正确配置，并且数据库服务正在运行。

### 3. 在测试中使用迁移

Rust 测试中可以使用 `sqlx` 提供的宏来自动管理测试数据库。

```rust
use sqlx::MySqlPool;

#[sqlx::test(migrations = "../migrations")]
async fn test_something(pool: MySqlPool) {
    // 测试代码
    // 在此函数运行前，会自动创建一个临时测试数据库，
    // 并应用 migrations 目录下的所有迁移。
    // pool 参数已连接到这个干净的测试数据库。
    // 测试结束后，临时数据库会被自动销毁。
}
```

### 4. （可选）手动建立数据库

虽然 `sqlx migrate` 通常假设数据库已存在，但如果你想手动创建初始数据库，可以使用以下 SQL 命令（在 MySQL 客户端中执行）：

```sql
-- 1. 创建数据库 (如果尚不存在)
CREATE DATABASE IF NOT EXISTS royale_arena
    DEFAULT CHARACTER SET utf8mb4
    DEFAULT COLLATE utf8mb4_unicode_ci;

-- 2. 创建用户并授权 (如果需要)
-- CREATE USER 'royale_user'@'localhost' IDENTIFIED BY 'your_strong_password';
-- GRANT ALL PRIVILEGES ON royale_arena.* TO 'royale_user'@'localhost';
-- FLUSH PRIVILEGES;
```

之后，将 `.env.example` 复制为 `.env`，并确保 `DATABASE_URL` 设置正确，例如 `DATABASE_URL=mysql://royale_user:your_strong_password@localhost:3306/royale_arena`，然后使用 `sqlx migrate run` 来应用迁移。

## 注意事项

*   迁移文件一旦被应用（即记录在数据库的 `_sqlx_migrations` 表中），就不应再修改其内容，以保证迁移历史的一致性。如果需要更改，应创建新的迁移。
*   `.down.sql` 脚本主要用于开发和测试环境，生产环境回滚需谨慎操作。
*   确保 `.env` 文件中的 `DATABASE_URL` 指向正确的数据库实例和名称。