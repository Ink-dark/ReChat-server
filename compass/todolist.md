# ReChat-sender 开发规划与任务清单

> 最后更新: 2026-04-26

---

## 一、已完成

| # | 任务 | 涉及文件 | 日期 |
|:--|------|---------|:---:|
| ✅ | 修复 REPO 未初始化时 `unwrap()` panic → 安全错误处理 | `src/api/endpoints/messages.rs`, `src/main.rs` | 04-26 |
| ✅ | 修复 `duration_since().unwrap()` → `unwrap_or_default()` + 日志 | `src/api/endpoints/messages.rs`, `src/core/message.rs` | 04-26 |
| ✅ | `Config::load()` 实际调用 — 新增 `--config` CLI 参数 | `src/main.rs` | 04-26 |
| ✅ | `workers` 从配置动态读取（不再硬编码为 1） | `src/main.rs` | 04-26 |
| ✅ | 清理无意义的 `--server` 参数 | `src/main.rs` | 04-26 |
| ✅ | 时间戳以 `i64` 数值存储（不再转字符串） | `src/core/message.rs` | 04-26 |
| ✅ | 移除 `cargo-tarpaulin` dev-dependency | `Cargo.toml` | 04-26 |
| ✅ | 无效枚举值错误类型从 `InvalidColumnType` → `InvalidQuery` | `src/core/message.rs` | 04-26 |
| ✅ | 数据库层时间戳合理性验证（零值/负值检查 + 日志） | `src/core/message.rs` | 04-26 |
| ✅ | API 层 `system_time_to_secs()` 安全转换 + 日志 | `src/api/endpoints/messages.rs` | 04-26 |
| ✅ | 合并远程 `adapter + plugin` 模块到 `main.rs` | `src/main.rs` | 04-26 |

---

## 二、待修复（代码质量）

| # | 任务 | 优先级 | 说明 |
|:--|------|:---:|------|
| 🔧 | `adapter.rs` — `AdapterStats::Default` 改用 `#[derive(Default)]` | 🟢 | Clippy suggestion，减少手写代码 |
| 🔧 | `adapter.rs` — 为 `AdapterManager` 实现 `Default` trait | 🟢 | Clippy suggestion，遵循 Rust 惯例 |
| 🔧 | `plugin.rs` — `PluginStats::Default` 改用 `#[derive(Default)]` | 🟢 | 同上 |
| 🔧 | `plugin.rs` — 为 `PluginManager` 实现 `Default` trait | 🟢 | 同上 |
| 🔧 | `MessageQueue` 未实例化，`services/queue.rs` 为死代码 | 🟡 | 需要 Redis 环境；需决定是集成还是移除此模块 |

---

## 三、待开发（功能特性）

### 🔴 当前进行中：OneBot v11 Adapter（NapCat/QQ 接入）

> 详细计划 → [compass/onebot_adapter_plan.md](compass/onebot_adapter_plan.md)

| Phase | 优先级 | 任务 | 状态 |
|:---:|:---:|------|:---:|
| P0 | 🔴 | 消息广播中枢 + 客户端 WS — `core/broadcaster.rs` + `api/endpoints/ws_client.rs` | ✅ |
| P1 | 🔴 | `src/adapters/onebot/protocol.rs` — OneBot 协议数据模型 | ⏳ |
| P2 | 🔴 | `src/adapters/onebot/ws.rs` — 平台 WS 端点 `/onebot/v11/ws` | ⏳ |
| P3 | 🔴 | `src/adapters/onebot/adapter.rs` — Adapter trait 实现 | ⏳ |
| P4 | 🔴 | 集成：lib.rs / api/mod.rs / main.rs 注册路由与初始化 | ⏳ |
| P5 | 🟡 | 测试 + 验证（cargo check / clippy / test） | ⏳ |

> ✅ **Phase 0 已完成** — 双 WS 通道基础设施就绪：`/ws/client` + 广播中枢。P1 开始接入 NapCat。

### 其他待开发

| # | 功能 | 优先级 | 涉及模块 | 说明 |
|:--|------|:---:|---------|------|
| 📦 | Redis 消息队列集成 | 🟡 | `services/queue.rs`, `main.rs` | 将 `MessageQueue` 实例化并接入消息发送流程 |
| 📦 | 更多 Adapter 实现 | 🟡 | `adapters/` | WebSocket、HTTP、Telegram、Discord 等 |
| 📦 | Plugin 实现（消息处理插件） | 🟡 | `core/plugin.rs` | 实现具体的 `Plugin` trait：加密插件、格式化插件等 |
| 📦 | 发送重试与调度逻辑 | 🟡 | `core/sender.rs` (新建) | 利用 `SenderConfig`(max_retries/retry_interval/batch_size) 实现后台发送调度 |
| 📦 | 消息状态流转 API | 🔴 | `api/endpoints/messages.rs` | `GET /api/messages` 列表查询、`PATCH` 取消/重发 |
| 📦 | Adapter 状态 API | 🟢 | `api/endpoints/adapters.rs` (新建) | 查看 adapter 状态、启停 adapter |
| 📦 | Plugin 管理 API | 🟢 | `api/endpoints/plugins.rs` (新建) | 查看/启用/禁用 plugin |
| 📦 | Web 界面完善 | 🟢 | `web/mod.rs`, `templates/` | 使用 Tera 模板重写 HTML 页面，添加实时状态 |
| 📦 | 配置验证 | 🟢 | `core/config.rs` | `Config` 加载后验证字段合法性（端口范围、路径等） |

---

## 四、待开发（测试与文档）

| # | 任务 | 优先级 | 说明 |
|:--|------|:---:|------|
| 📝 | API 端点集成测试 | 🟡 | 启动测试服务器，测试 `POST /api/messages`、`GET /api/messages/{id}` |
| 📝 | `MessageRepository` 完整测试 | 🟡 | 测试 `get_pending_messages`、重复 save 覆盖、无效 ID |
| 📝 | `MessageQueue` 测试（需要 Redis） | 🟢 | 测试 push/pop/len/is_empty |
| 📝 | 错误场景测试 | 🟡 | REPO 未初始化时 API 返回 500 而非 panic；无效 message type 返回 400 |

---

## 五、架构改进

| # | 建议 | 优先级 | 说明 |
|:--|------|:---:|------|
| 🏗 | REPO 从 `thread_local!` 改为 Actix `Data<T>` | 🟡 | 更符合 Actix 惯例，便于测试和注入依赖 |
| 🏗 | 提取发送器核心为独立 actor/service | 🟡 | 将消息发送、重试、状态管理封装为独立模块 |
| 🏗 | `rusqlite` 使用 WAL 模式 | 🟢 | 提升并发读取性能：`PRAGMA journal_mode=WAL;` |
| 🏗 | 配置支持环境变量覆盖 | 🟢 | `Config::load()` 后按 `RE_CHAT_*` 环境变量覆盖字段 |

---

## 六、依赖与发布

| # | 任务 | 优先级 | 说明 |
|:--|------|:---:|------|
| 📦 | 关注 `redis 0.24.0` / `net2` 未来兼容性 | 🟢 | Rust 未来版本将拒绝，需等待上游更新 |
| 📦 | Windows 服务化部署脚本 | 🟢 | `scripts/` 下添加安装/卸载 Windows Service 脚本 |
| 📦 | 编写 `DEPLOY.md` | 🟢 | 补充部署步骤、配置文件示例、CLI 使用说明 |

---

## 优先级图例

| 标记 | 含义 |
|:---:|------|
| 🔴 | 高优先级 — 阻塞性、影响核心功能 |
| 🟡 | 中优先级 — 重要但非阻塞 |
| 🟢 | 低优先级 — 改进优化，按需处理 |
| ✅ | 已完成 |
| 🔧 | 待修复 |
| 📦 | 待开发 |
| 📝 | 待测试 |
| 🏗 | 架构建议 |
