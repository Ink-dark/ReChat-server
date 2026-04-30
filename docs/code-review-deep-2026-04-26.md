# ReChat-sender 深度代码审查 — 细枝末节与逻辑问题

> 日期: 2026-04-26 | 类型: 深度审查 | 范围: 全项目 .rs 文件

---

## 🔴 逻辑缺陷

### 1. broadcaster 未按 conversations 过滤 — 消息泄漏

**文件**: [broadcaster.rs:76-L77](file:///d:/ReChat-server/src/core/broadcaster.rs#L76-L77)

```rust
for (id, session) in sessions.iter() {
    if session.platforms.contains(platform)
        && let Ok(json) = serde_json::to_string(msg)
        && session.sender.send(json).is_err()
```

只检查 `platforms`，完全忽略 `conversations`。根据设计文档，客户端可订阅特定会话（如只订阅 `group_123`）。但当前实现：只要平台匹配，**所有会话的消息都会推送给该客户端**。

**后果**: 客户端 A 订阅 `qq` 平台下 `group_123`，也会收到 `group_456` 的消息。

`broadcast_adapter_status` 有同样问题。

**提议修复**:
```rust
if (session.platforms.is_empty() || session.platforms.contains(platform))
    && (session.conversations.is_empty() || ...contains conversation...)
```

---

### 2. `start_all()`/`initialize_all()` 遇首个失败即终止

**文件**: [adapter.rs:57-L62](file:///d:/ReChat-server/src/core/adapter.rs#L57-L62) / [plugin.rs:63-L68](file:///d:/ReChat-server/src/core/plugin.rs#L63-L68)

```rust
pub fn start_all(&self) -> Result<(), Box<dyn std::error::Error>> {
    for adapter in &self.adapters {
        adapter.start()?;  // ← ? here means return on first error
    }
    Ok(())
}
```

如果配置了 3 个 adapter，第 1 个启动失败，第 2、3 个**完全不会被尝试**。`stop_all`、`shutdown_all` 同理。

**提议修复**: 收集所有错误，至少打印日志，不要提前终止。

---

### 3. `get_plugin_info` 硬编码状态为 `Enabled`

**文件**: [plugin.rs:108](file:///d:/ReChat-server/src/core/plugin.rs#L108)

```rust
status: PluginStatus::Enabled,
```

无论 plugin 的实际状态如何，返回的都是 `Enabled`。如果 plugin `initialize()` 失败了，这里仍然显示 `Enabled`。

---

## 🟡 边界与鲁棒性

### 4. subscribe/unsubscribe 无确认 (ack)

**文件**: [ws_client.rs:128-L131](file:///d:/ReChat-server/src/api/endpoints/ws_client.rs#L128-L131)

客户端发送 `subscribe` → 服务端直接处理，不回复。客户端无从知道操作是否成功（session 还存活吗？订阅成功了吗？）。

send_message 有 ack，但 subscribe/unsubscribe 没有。

---

### 5. 无效 JSON 静默丢弃

**文件**: [ws_client.rs:92](file:///d:/ReChat-server/src/api/endpoints/ws_client.rs#L92)

```rust
if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&text) {
    handle_command(...).await;
}
```

客户端发送了格式错误的 JSON → 服务端收到后**什么都没做**，不报错、不回复。客户端会一直等直到超时。

---

### 6. HTTP API `create_message` 不触发广播

**文件**: [messages.rs:49-L69](file:///d:/ReChat-server/src/api/endpoints/messages.rs#L49-L69)

通过 `POST /api/messages` 创建的消息只保存到数据库，**不调用 `broadcaster.broadcast_message()`**。这意味着 WebSocket 客户端看不到 REST API 创建的消息 — 这可能是设计意图，但也可能是不一致的行为（send_message via WS 会广播，send_message via HTTP 不广播）。

---

### 7. `send_message` 中 `conversation` 未做非空校验

**文件**: [ws_client.rs:L148-L158](file:///d:/ReChat-server/src/api/endpoints/ws_client.rs#L148-L158)

`platform` 和 `content` 有空值校验，但 `conversation` (`recipient`) 没有。如果客户端不提供 conversation，会创建一个 `recipient = ""` 的消息。

---

## 🟢 代码气味

### 8. subscribe/unsubscribe 静默忽略不存在的 session

**文件**: [broadcaster.rs:97](file:///d:/ReChat-server/src/core/broadcaster.rs#L97)

```rust
if let Some(s) = sessions.get_mut(session_id) {
```

session 不存在时静默无操作，调用方无法感知。建议加 `tracing::warn!` 或返回 `bool`。

---

### 9. HTTP API 错误分支无 `tracing::error!`

**文件**: [messages.rs:63-L68](file:///d:/ReChat-server/src/api/endpoints/messages.rs#L63-L68)

数据库保存/读取失败时只返回 JSON 错误给客户端，不写日志。出问题时运维无法定位根因。

---

### 10. 发送 task 和接收 task 存在 double-unregister

**文件**: [ws_client.rs:L64-L69](file:///d:/ReChat-server/src/api/endpoints/ws_client.rs#L64-L69) + [L107-L114](file:///d:/ReChat-server/src/api/endpoints/ws_client.rs#L107-L114)

发送 task 出错时会 `unregister + close`；close 可能触发接收端收到 Close frame → 再次 `unregister`。两个 task 都可能 try-unregister 同一个 session。虽然 `HashMap::remove` 对不存在的 key 返回 None（无害），但是两次 debug 日志会让人困惑。

---

### 11. CLI 模块 `unwrap()` 风险低但仍在

**文件**: [cli/mod.rs:L57-L70](file:///d:/ReChat-server/src/cli/mod.rs#L57-L70)

`value_of("type").unwrap()` — 虽然 clap 的 `required(true)` 保证不为 None，但 `unwrap()` 仍是一种代码气味。同文件 L70、L80 同理。

---

### 12. 测试覆盖不足

**文件**: [tests/sender.rs](file:///d:/ReChat-server/tests/sender.rs)

仅 2 个测试，未覆盖:
- `get_pending_messages` 查询行为
- 重复 `save` 同一 ID 的覆盖行为 (`INSERT OR REPLACE`)
- 无效 message_type/status 从 DB 读取的错误处理
- Edge case: `id` 为不存在 UUID 时的 `get` 返回 `None`

---

## 审查结论

| 严重度 | 数量 | 关键项 |
|:---:|:---:|------|
| 🔴 逻辑缺陷 | 3 | broadcaster 未过滤 conversations、start_all 提前终止、plugin 状态硬编码 |
| 🟡 鲁棒性 | 4 | subscribe 无 ack、无效 JSON 静默丢弃、conversation 未校验、HTTP API 不广播 |
| 🟢 代码气味 | 5 | 静默忽略、日志不足、double-unregister、unwrap、测试覆盖 |
