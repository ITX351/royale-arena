# 行动结果消息系统重构设计文档

## 1. 概述

本设计旨在重构现有的行动结果消息系统，以支持在同一行动中向不同接收端发送差异化消息。重构将引入新的 `ActionResults` 类型，并增强 `ActionResult` 的广播控制能力。

## 2. 设计变更

### 2.1 新增 ActionResults 类型

引入新的 `ActionResults` 结构，用于包装多个 `ActionResult` 对象：

```rust
pub struct ActionResults {
    pub results: Vec<ActionResult>,
}
```

同时为 `ActionResult` 实现便捷转换方法：

```rust
impl ActionResult {
    pub fn as_results(self) -> ActionResults {
        ActionResults {
            results: vec![self],
        }
    }
}
```

### 2.2 增强 ActionResult 广播控制

修改 `ActionResult` 的工厂方法，为 `new_system_message` 和 `new_user_message` 添加 `broadcast_to_director` 参数：

```rust
impl ActionResult {
    pub fn new_system_message(data: serde_json::Value, broadcast_players: Vec<String>, log_message: String, broadcast_to_director: bool) -> Self {
        ActionResult::new(data, broadcast_players, log_message, MessageType::SystemNotice, broadcast_to_director)
    }
    
    pub fn new_user_message(data: serde_json::Value, broadcast_players: Vec<String>, log_message: String, broadcast_to_director: bool) -> Self {
        ActionResult::new(data, broadcast_players, log_message, MessageType::UserDirected, broadcast_to_director)
    }
}
```

### 2.3 差异化消息处理

#### 2.3.1 攻击行动消息差异化

在 `handle_attack_action` 中，将返回两个不同的消息：
1. 向攻击者和导演发送完整消息
2. 向被攻击者发送隐去攻击者身份的差分消息（不向导演广播）

#### 2.3.2 传音行动消息差异化

在 `handle_deliver_action` 中，将返回两个不同的消息：
1. 向发送者和导演发送完整消息
2. 向接收者发送格式调整后的差分消息（不向导演广播）

#### 2.3.3 导演行动消息处理

所有导演行动处理方法都将更新为新的返回类型，并保持原有的广播逻辑。

## 3. 接口变更

所有原本返回 `Result<ActionResult, String>` 的函数将改为返回 `Result<ActionResults, String>`。

## 4. 实现计划

1. 修改 `ActionResult` 结构和工厂方法
2. 添加 `ActionResults` 结构和相关转换方法
3. 更新所有使用 `ActionResult` 的函数签名
4. 实现攻击行动的差异化消息处理
5. 实现传音行动的差异化消息处理
6. 更新导演行动处理方法
7. 更新玩家行动调度器
8. 更新导演行动调度器
9. 更新所有调用点以适应新的返回类型

以下为用户的原输入：
```
重构返回消息系统以支持同一次行动产生不同的消息差分下发给不同的接收端ws。我们扩充如下定义：
1、新建一个类ActionResults，它只包括ActionResult的简单vector。ActionResult支持一个as_results或类似的函数以方便简单地转换成一个只包括一个元素的ActionResults对象。所有原来是`Result<ActionResult, String>`的函数均改为返回`Result<ActionResults, String>`，并且绝大多数函数都改为调用新的简单转换函数以生成新格式的返回值。
2、在ActionResult的`new_system_message`和`new_user_message`函数中，加入与`new_info_message`相同的`broadcast_to_director`参数。原有对`new_system_message`和`new_user_message`的调用均将此项设为真。
3、`handle_attack_action`发生时，向导演和原攻击发起者发送完整的原消息，向被攻击者发送一个隐去攻击者的差分消息（格式形似`你被攻击了……`），这个消息需要设置成不对导演广播，以确保只有被攻击者收到这条消息，包括导演在内的其他人都不会收到这条消息（被攻击者将不再收到原有的包括攻击来源的消息）。将这条新消息与原消息打包进ActionResults。
4、`handle_deliver_action`发生时，也设计一个简单的差分以对发起者和接收者发送不同的消息。发起者和导演看到的是目前代码中展示的消息内容，对接收方不隐去发送者的名字，但是简单修改格式（`你收到了来自……的消息`），并确保这个消息不会对导演展示。
所有的修改均应是破坏性的，不需要考虑任何向后兼容。完成最小实现，拒绝任何过度设计，不编写任何测试，所有输出使用简体中文并显式指定所有输出使用简体中文。设计方案中避免长篇输出代码。
```
