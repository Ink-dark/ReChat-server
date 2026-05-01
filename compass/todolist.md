# ReChat-provider 开发规划与任务清单

> 最后更新: 2026-05-01

---

## 一、已完成 (近期)

| 日期 | 任务 | 说明 |
|:---:|------|------|
| 04-30 | Phase 0-4: OneBot v11 适配器 | protocol / WS 端点 / Adapter trait 全部完成 |
| 04-30 | Web UI 重构 | Vanilla JS SPA，侧边栏布局，亮/暗/Auto 三模式主题 |
| 04-30 | 路由合并修复 | 多个 `web::scope("")` 冲突 → `configure()` |
| 04-30 | Token 认证系统 | 启动生成 UUID → 登录遮罩 → localStorage → API/WS 携带 |
| 04-30 | CI 格式修复 × 3 | import order / 行宽 / 链式调用换行 |
| 05-01 | 集成测试补全 | 39 个新测试: broadcaster(11) + protocol(21) + auth(7) |
| 05-01 | Phase B: API 补全 | list/stats/PATCH/DELETE 全部完成；status Canceled 拆分修复 |
| 05-01 | Phase C1-C2: 消息流 + Dispatcher | 图片/文件类型扩展、sender 补全、Canceled 跳过、Web 前端同步 |

---

## 二、下一步开发计划

### 🔴 Phase A: 消息发送调度核心 (P0)

> 当前 `/ws/client` 的 `send_message` 只调用 `send_to_adapter` 但不处理结果/重试。需要后台调度器。

| # | 任务 | 优先级 | 文件 | 说明 |
|:--|------|:---:|------|------|
| A1 | 创建 `core/dispatcher.rs` | 🔴 | 新建 | 后台 tokio task 轮询 Pending 消息 → 匹配 Adapter 发送 → 更新状态 |
| A2 | `SenderConfig` 接入调度器 | 🔴 | dispatcher.rs | max_retries / retry_interval / batch_size / concurrency |
| A3 | `MessageRepository::update_status()` | 🔴 | message.rs | 发送成功后更新为 Sent，失败后 retry_count++ |
| A4 | 调度器生命周期管理 | 🔴 | main.rs + dispatcher.rs | start / graceful shutdown |
| A5 | 集成测试 | 🔴 | tests/dispatcher.rs | 模拟发送成功/失败/超时/最大重试 |

### 🟡 Phase B: API 补全 ✅ 已完成

| # | 任务 | 优先级 | 说明 |
|:--|------|:---:|------|
| B1 | `GET /api/messages` 列表分页 | ✅ | 支持 `?limit=&offset=&status=` 筛选 |
| B2 | `PATCH /api/messages/{id}` 取消/重发 | ✅ | 支持 Pending/Sending/Sent/Failed/Canceled |
| B3 | `GET /api/stats` 统计概览 | ✅ | 5种状态计数 + total |
| B4 | `DELETE /api/messages/{id}` | ✅ | 删除消息记录 |

### 🟡 Phase C: 消息流完善 🔴 进行中

| # | 任务 | 优先级 | 说明 |
|:--|------|:---:|------|
| C1 | OneBot 图片/文件消息类型扩展 | ✅ | 根据消息段自动识别 Image/File/Text 类型 |
| C2 | 入站消息 sender 信息补全 | ✅ | conversation_name 从 nickname/群号自动生成 |
| C3 | 多平台支持框架就绪 | 🟢 | adapter 注册机制验证 (QQ 已有，可加 mock 测试微信) |

### 🟢 Phase D: 运维与部署

| # | 任务 | 优先级 | 说明 |
|:--|------|:---:|------|
| D1 | Dockerfile | 🟢 | 多阶段构建，静态链接 musl |
| D2 | docker-compose.yml | 🟢 | ReChat + (可选 Redis) |
| D3 | 配置验证 | 🟢 | `Config` 加载后验证 port 范围、path 可写性 |
| D4 | 环境变量覆盖配置 | 🟢 | `RE_CHAT_*` 前缀覆盖任意字段 |

### 🟢 Phase E: 架构优化

| # | 任务 | 优先级 | 说明 |
|:--|------|:---:|------|
| E1 | REPO `thread_local!` → `Data<T>` | 🟢 | 更符合 Actix 惯例 |
| E2 | SQLite WAL 模式 | 🟢 | `PRAGMA journal_mode=WAL` 提升并发读 |
| E3 | adapter_manager 列表 API | 🟢 | `GET /api/adapters` 返回 JSON |
| E4 | Web 仪表盘实时数据 | 🟢 | 统计卡片从 WS 获取 live 数据 |

---

## 三、优先级总结

| 优先级 | Phase | 概述 | 预计涉及文件数 |
|:---:|:---:|------|:---:|
| 🔴 P0 | A | **消息发送调度器** — 这是让 OneBot 真正能"发消息到 QQ"的最后一块拼图 | 4 文件 |
| 🟡 P1 | B | **API 补全** — 消息列表/状态变更，前端才能展示历史消息 | 3 文件 |
| 🟡 P1 | C | **消息流完善** — 图片/文件/多平台 | 3 文件 |
| 🟢 P2 | D | **运维部署** — Docker / 配置验证 | 3 文件 |
| 🟢 P3 | E | **架构优化** — REPO / WAL / API | 4 文件 |

---

## 四、关键里程碑

```
✅ 已完成 — 2026-05-01
├── OneBot v11 适配器 (入站/出站)
├── Web UI SPA (仪表盘/消息流/发送/平台)
├── Access Token 认证
├── 44 个集成测试 (broadcaster 11 + protocol 21 + auth 7 + repo 5)
└── CI 格式合规

⏳ Phase A: 发送调度器 → 目标 2026-05-02
├── [ ] dispatcher 后台任务
├── [ ] 状态流转 Pending→Sending→Sent/Failed
├── [ ] 重试逻辑 (max_retries + retry_interval)
└── [ ] 端到端验证: Web UI 发送 → NapCat → QQ 群

⏳ Phase B: API 补全 → 目标 2026-05-03
├── [ ] GET /api/messages 分页
├── [ ] PATCH /api/messages/{id}
└── [ ] GET /api/stats

⏳ Phase C-E: 完善 + 部署 → 目标 2026-05-05
├── [✅] 图片/文件类型识别
├── [✅] sender 信息补全
├── [✅] Dispatcher Canceled 跳过
├── [✅] Web 前端状态同步
├── [ ] Docker 部署
└── [ ] 架构优化
```
