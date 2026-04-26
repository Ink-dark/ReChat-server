# ReChat-sender 代码修复计划

> 基于代码审查发现的 10 个潜在问题，按优先级修复
> 修复日期：2026-04-26

## 完成度统计

| 状态 | 数量 |
|------|:---:|
| ✅ 已修复 | 8 |
| ⏳ 暂缓 | 1 |
| ➖ 留待观察 | 1 |
| **总计** | **10** |

---

## 🔴 严重问题（优先修复）

### #1 ✅ REPO 未初始化时 unwrap() 导致服务崩溃
- **文件**: `src/api/endpoints/messages.rs`
- **问题**: `repo.borrow().as_ref().unwrap()` 在 REPO 为 None 时直接 panic，因 `workers(1)` 导致整个服务宕掉
- **修复方式**: 将 `unwrap()` 替换为 `match` + `map()`，在 REPO 未初始化时返回 `HttpResponse::InternalServerError`
- **状态**: ✅ 已修复 — `create_message` 和 `get_message` 均已改用 `borrow().as_ref().map(...)` 安全处理

### #2 ✅ duration_since() 无条件 unwrap() 潜在 Panic
- **文件**: `src/api/endpoints/messages.rs`, `src/core/message.rs`
- **问题**: 系统时间早于 UNIX_EPOCH 时 `duration_since()` 返回 Err，unwrap 会导致 panic
- **修复方式**: 使用 `unwrap_or_default()` 安全处理
- **状态**: ✅ 已修复 — 两处共 4 个 `unwrap()` 均替换为 `unwrap_or_default()`

### #3 ✅ Config::load() 从未调用，配置文件被忽略
- **文件**: `src/main.rs`
- **问题**: 始终使用 `Config::default()`，用户无法通过配置文件修改任何设置
- **修复方式**: 添加 `--config` CLI 参数，优先从配置文件加载，fallback 到默认值
- **状态**: ✅ 已修复 — 新增 `-c/--config <PATH>` 参数，加载失败时警告并使用默认配置

---

## 🟡 中等问题

### #4 ✅ 配置 workers: 4 与实际 workers(1) 矛盾
- **文件**: `src/main.rs`, `src/core/config.rs`
- **问题**: 配置中设置了 4 个 worker 但硬编码为 1 个，且 workers 字段完全未被使用
- **修复方式**: 使用 `config.server.workers` 替换硬编码的 `workers(1)`
- **状态**: ✅ 已修复 — `.workers(config.server.workers)`

### #5 ✅ --server CLI 参数 required(true) 但被 _matches 丢弃
- **文件**: `src/main.rs`
- **问题**: `--server` 是必填参数但结果赋给 `_matches`（前缀 `_` 表示忽略），完全没有使用
- **修复方式**: 移除无意义的 `--server` 参数，改为可选的 `--config` 参数
- **状态**: ✅ 已修复 — `--server` 已替换为 `-c/--config`

### #6 ✅ 时间戳以字符串存储到 INTEGER 列
- **文件**: `src/core/message.rs`
- **问题**: Schema 声明 `INTEGER` 但实际存储 `"1234567890".to_string()` 字符串，影响数值查询和排序
- **修复方式**: 直接将 `created_at_secs` (i64) 传给 SQL 参数，改用 `rusqlite::params!` 宏支持混合类型
- **状态**: ✅ 已修复 — 时间戳和 retry_count 均以数值类型存储

### #7 ⏳ MessageQueue(Redis) 已实现但从未被使用
- **文件**: `src/services/queue.rs`
- **问题**: 完整的 Redis 消息队列模块已实现但 main.rs 中未实例化，属于死代码
- **修复方式**: 暂不修改代码，需要 Redis 运行环境和消息发送的完整业务逻辑配合
- **状态**: ⏳ 暂缓 — 后续集成任务

---

## 🟢 轻微问题

### #8 ➖ get_pending_messages 中 limit: usize 类型不够明确
- **文件**: `src/core/message.rs`
- **问题**: `usize` 传给 SQL 查询参数，虽然能工作但不够明确
- **修复方式**: 保持原样，影响极小
- **状态**: ➖ 留待观察 — 当前无实际影响

### #9 ✅ cargo-tarpaulin 不应作为 dev-dependency
- **文件**: `Cargo.toml`
- **问题**: `cargo-tarpaulin` 是 CLI 工具，应通过 `cargo install` 全局安装，不应作为项目依赖
- **修复方式**: 从 `[dev-dependencies]` 中移除
- **状态**: ✅ 已修复 — 已从 Cargo.toml 中移除

### #10 ✅ 无效枚举值返回不恰当的 InvalidColumnType 错误
- **文件**: `src/core/message.rs`
- **问题**: 遇到未知的 message_type/status 值时返回 `InvalidColumnType` 错误，语义不准确
- **修复方式**: 改为 `rusqlite::Error::InvalidQuery`
- **状态**: ✅ 已修复 — `get()` 方法中两处均已更改

---

## 修复顺序（实际执行记录）

| 序号 | 修复项 | 状态 |
|:----:|--------|:---:|
| 1 | #9 — 清理 Cargo.toml | ✅ |
| 2 | #5 — 清理 main.rs CLI 参数 | ✅ |
| 3 | #3 + #4 — 修复配置加载 + workers 动态化 | ✅ |
| 4 | #6 — 修复时间戳存储方式 | ✅ |
| 5 | #2 — 修复 duration_since unwrap | ✅ |
| 6 | #1 — 修复 REPO unwrap panic | ✅ |
| 7 | #10 — 改进错误类型 | ✅ |
| 8 | #8 — limit 类型（留待观察） | ➖ |
| 9 | #7 — Redis 队列集成（暂缓） | ⏳ |

## 验证结果

```
cargo check    ✅ 0 errors, 0 warnings (项目代码)
cargo clippy   ✅ 0 warnings (项目代码)
cargo test     ✅ 2/2 passed
```
