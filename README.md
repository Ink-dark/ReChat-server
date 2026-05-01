# ReChat-provider

> 消息集合平台 — 连接 QQ（NapCat/OneBot），实时推送，Web 管理

[![CI](https://github.com/Ink-dark/ReChat-server/actions/workflows/ci.yml/badge.svg)](https://github.com/Ink-dark/ReChat-server/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

**ReChat** 是一个基于 Rust + Actix-web 构建的消息集合平台，通过 OneBot v11 协议接入 NapCat/QQ，将所有消息汇聚到一个中心，并通过 WebSocket 实时推送到 Web 管理界面。

<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/Actix--Web-000000?style=flat&logo=rust&logoColor=white" alt="Actix-web" />
  <img src="https://img.shields.io/badge/SQLite-003B57?style=flat&logo=sqlite&logoColor=white" alt="SQLite" />
  <img src="https://img.shields.io/badge/OneBot_v11-12B7F5?style=flat&logo=tencentqq&logoColor=white" alt="OneBot v11" />
</p>

***

## ✨ 功能特性

### 核心能力

| 功能              |  状态 | 说明                                                        |
| --------------- | :-: | --------------------------------------------------------- |
| **多平台消息接入**     |  ✅  | OneBot v11 (NapCat/QQ)，可扩展至微信/Telegram/Discord            |
| **实时消息推送**      |  ✅  | 双 WebSocket 通道：平台入站 `/onebot/v11/ws` + 客户端出站 `/ws/client` |
| **消息广播中枢**      |  ✅  | `MessageBroadcaster` 按平台/会话订阅，精准推送                        |
| **消息生命周期管理**    |  ✅  | Pending → Sending → Sent → Failed → Canceled 全状态流转          |
| **消息类型识别**       |  ✅  | Text / Image / File / Video / Audio 五大消息类型                   |
| **Web 管理界面**    |  ✅  | Vanilla JS SPA：仪表盘 / 消息流 / 发送消息 / 平台状态                    |
| **暗色模式**        |  ✅  | 亮色/暗色/Auto 三模式，localStorage 持久化                           |
| **RESTful API** |  ✅  | 创建/查询/列表/更新状态/删除消息/统计概览 + 健康检查（[完整文档](docs/api/README.md)）                  |
| **CLI 工具**      |  ✅  | 命令行发送消息 / 查询状态                                            |
| **Adapter 架构**  |  ✅  | 可插拔的适配器系统，方便接入新平台                                         |
| **Plugin 架构**   |  ✅  | 消息处理插件系统（过滤/转换/加密）                                        |
| **消息发送调度器**     |  ✅  | 后台 tokio task 轮询 + 重试 + 并发控制 + 原子认领                         |
| **SQLite 持久化**  |  ✅  | 消息数据本地存储，零配置                                              |
| **结构化日志**       |  ✅  | tracing + 文件/双输出，文件创建失败自动降级                               |

***

## 🏗 架构

```
                    ┌──────── 入站 ────────┐    ┌──────── 出站 ────────┐
                    │                      │    │                      │
  NapCat ──WS──→ /onebot/v11/ws          HTTP API                   /ws/client ←──WS── Web UI
    │                    │                POST /api/messages              │
    ▼                    ▼                      │                        ▼
 OneBot 事件         协议解析 (protocol.rs)    创建消息              Client WS Handler
    │                    │                      │                        │
    ▼                    ▼                      ▼                        ▼
 MessageEvent         OneBotAdapter         Message (DB)          MessageBroadcaster
         │              .send_message()          │                  (广播中枢)
         │                    │                  │   ▲                     ▲
         ▼                    ▼                  │   │                     │
    OneBotAdapter ◀── Message (内部) ◀── MessageDispatcher ◀── 轮询 Pending
    .send_message()                       (调度器 + 重试)
         │
         ▼
  send_group_msg / send_private_msg ──WS──→ NapCat

  ┌─────────────────────────────────────────────────────────────────────┘
  │
  ▼
 保存 DB ──→ broadcast ──→ 所有订阅该平台的 Web UI 实时收到
```

### 模块结构

```
src/
├── adapters/                    # 平台适配器
│   └── onebot/                  # OneBot v11 (NapCat/QQ)
│       ├── protocol.rs          #   协议数据模型 (Action/Event/Segment)
│       ├── ws.rs                #   WS 端点 /onebot/v11/ws
│       └── adapter.rs           #   Adapter trait 实现
├── api/
│   ├── endpoints/
│   │   ├── messages.rs          # HTTP REST API
│   │   └── ws_client.rs         # 客户端 WS /ws/client
│   └── mod.rs                   # 路由注册
├── core/
│   ├── adapter.rs               # Adapter trait + AdapterManager
│   ├── broadcaster.rs           # 消息广播中枢 (MessageBroadcaster)
│   ├── config.rs                # 配置管理
│   ├── dispatcher.rs            # 消息发送调度器 (轮询 + 重试 + 原子认领)
│   ├── logging.rs               # 日志初始化
│   ├── message.rs               # 消息仓库 (SQLite)
│   └── plugin.rs                # Plugin trait + PluginManager
├── models/
│   └── message.rs               # Message 数据模型
├── web/
│   ├── mod.rs                   # include_str! 嵌入服务
│   └── templates/
│       ├── index.html           # SPA 入口
│       ├── app.css              # 番茄红主题 + 暗色模式
│       └── app.js               # Hash 路由 + WebSocket + API
├── cli/mod.rs                   # 命令行工具
├── services/queue.rs            # Redis 消息队列（待集成）
├── lib.rs
└── main.rs
```

***

## 🚀 快速开始

### 前提

- Rust 1.70+
- (可选) Redis — 消息队列（当前无需）

### 安装与运行

```bash
# 克隆
git clone https://github.com/Ink-dark/ReChat-server.git
cd ReChat-server

# 构建
cargo build --release

# 启动
cargo run --release

# 使用自定义配置
cargo run --release -- --config config.json
```

服务启动后：

| 地址                                 | 说明                  |
| ---------------------------------- | ------------------- |
| `http://localhost:8080`            | Web 管理界面            |
| `http://localhost:8080/api/health` | 健康检查                |
| `GET /ws/client`                   | WebSocket 客户端连接     |
| `GET /onebot/v11/ws`               | NapCat WebSocket 连接 |

### 连接 NapCat

在 NapCat 中配置反向 WebSocket：

```
WS 地址: ws://localhost:8080/onebot/v11/ws
消息格式: array
```

配置完成后打开 Web 界面，即可实时查看 QQ 消息。

***

## 📡 API

### HTTP API

| 方法     | 路径                   | 说明     |
| ------ | -------------------- | ------ |
| `POST` | `/api/messages`      | 创建消息   |
| `GET`  | `/api/messages`      | 消息列表（支持 `?offset=&limit=&status=`） |
| `GET`  | `/api/messages/{id}` | 查询单条消息 |
| `PATCH` | `/api/messages/{id}` | 更新消息状态 |
| `DELETE` | `/api/messages/{id}` | 删除消息   |
| `GET`  | `/api/stats`         | 统计概览   |
| `GET`  | `/api/health`        | 健康检查   |

### WebSocket 指令

**客户端 → 服务端** (`/ws/client`)：

| type           | 说明      |
| -------------- | ------- |
| `subscribe`    | 订阅平台/会话 |
| `unsubscribe`  | 取消订阅    |
| `send_message` | 向平台发送消息 |

**服务端 → 客户端**：

| type             | 说明     |
| ---------------- | ------ |
| `new_message`    | 新消息推送  |
| `adapter_status` | 平台连接状态 |

详细文档 → [docs/api/README.md](docs/api/README.md)

***

## ⌨️ 命令行工具

```bash
# 发送消息 (支持 text / image / file / video / audio)
cargo run -- send -t text -r user1 -c "Hello"

# 查询状态
cargo run -- status -i <message-id>
```

***

## 🎨 Web UI（借鉴于[Tomato-Novel-Downloader](https://github.com/zhongbai2333/Tomato-Novel-Downloader)）

| 页面   | Hash         | 功能                  |
| ---- | ------------ | ------------------- |
| 仪表盘  | `#dashboard` | 6 状态统计卡片 + 最近消息（调用 `/api/stats`）         |
| 消息流  | `#messages`  | WebSocket 实时推送 + 筛选 |
| 发送消息 | `#send`      | 表单 → 平台发送           |
| 平台状态 | `#platforms` | Adapter 连接状态        |

特性：

- 🌙 亮色/暗色/Auto 主题切换
- 🔴 实时消息推送 + 断线自动重连
- 📱 768px 响应式
- ⚡ 零前端依赖，全部内嵌 Rust 二进制

***

## ⚙️ 配置

```json
{
  "server": {
    "host": "127.0.0.1",
    "port": 8080,
    "workers": 4,
    "web_ui": true,
    "web_ui_port": 8081
  },
  "database": {
    "path": "./rechat.db",
    "max_connections": 5
  },
  "sender": {
    "max_retries": 3,
    "retry_interval": 5,
    "batch_size": 10
  },
  "adapters": [],
  "plugins": []
}
```

***

## 🧪 开发

```bash
# 运行所有检查
cargo check --features windows-gui
cargo clippy
cargo test

# 格式化代码
cargo fmt
```

### 待开发

| 优先级 | 功能                           |
| :-: | ---------------------------- |
|   | 更多平台适配 (微信/Telegram/Discord) |
|  🟡 | Redis 消息队列集成                 |
|  🟢 | Plugin 实现 (加密/格式化/翻译)        |
|  🟢 | Docker 部署                     |

***

## 📄 许可证

Apache 2.0 License, Version 2.0 — 详见 [LICENSE](LICENSE)

***

## 特别鸣谢
- [Tomato-Novel-Downloader](https://github.com/zhongbai2333/Tomato-Novel-Downloader) --- UI 设计与实现参考
- [Ink-dark](https://github.com/Ink-dark) --- 项目维护与贡献
- 某人的大脑和眼睛 --- 提供了构想与建议
- 比特火炬 --- 提供了项目短链接跳转服务和部分人力支持
