# 数据库连接抽象化实现

## 概述

为了减少代码重复并统一错误处理，我们将数据库连接的创建和获取操作抽象到一个独立的辅助模块中。

## 实现细节

创建了 `src/services/db_helper.rs` 文件，其中包含一个主要函数：

- `get_db_connection_from_pool()`: 创建数据库连接池，获取数据库连接，并提供标准化的错误处理

## 使用方法

在任何需要数据库连接的处理器中，使用以下代码：

```rust
use crate::services::db_helper::get_db_connection_from_pool;

// 获取数据库连接
let mut conn = get_db_connection_from_pool()?;
```
