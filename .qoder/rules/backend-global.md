---
trigger: glob
glob: backend/**/*.rs
---

使用Axum 0.8.4时，注意读取[Axum 0.8.4 框架使用指南](/docs/frameworks/axum-usage.md)学习新版使用方法。

使用现代Rust模块系统：
- 不要创建mod.rs文件，创建与目录同名的.rs文件来定义模块
- 例如：src/user.rs 而不是 src/user/mod.rs