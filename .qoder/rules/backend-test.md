---
trigger: model_decision
description: 后台生成rust测试
---

测试模块分为以下四种：

1. 单元测试：纯逻辑测试，无数据库依赖
2. 数据库集成测试：每种主要服务有一个集成测试
   - 使用 #[sqlx::test(migrations = "./migrations")]
   - 测试真实数据库交互
3. API集成测试: 复杂业务流程的端到端测试
4. 测试数据自动管理，依赖sqlx::test事务自动清理
5. 返回值使用Result，`.await`后使用`?`传播错误，Result传播与assert混合。