# ReChat-sender

ReChat-sender 是一个使用 Rust 和 Actix-web 构建的消息发送系统，支持文本、图片和文件消息，并提供了 RESTful API 和 Web 界面。

## 功能特性

- **多消息类型支持**：文本、图片和文件消息
- **完整的消息生命周期管理**：待处理、发送中、已发送、失败状态
- **RESTful API**：用于创建和查询消息
- **Web 界面**：提供直观的消息发送和状态查询界面
- **消息存储**：使用 SQLite 数据库存储消息
- **消息队列**：使用 Redis 作为消息队列
- **重试机制**：自动处理发送失败的消息
- **状态跟踪**：实时跟踪消息发送状态

## 技术栈

- **Rust**：高性能、安全的系统编程语言
- **Actix-web**：Rust 生态中的高性能 Web 框架
- **SQLite**：轻量级嵌入式数据库
- **Redis**：用于消息队列和缓存
- **Serde**：用于序列化和反序列化

## 目录结构

```
ReChat-sender/
├── .gitignore
├── Cargo.toml
├── DEPLOY.md
├── README.md
├── rechat.db
├── scripts/
│   └── build.rs
├── src/
│   ├── api/
│   │   ├── endpoints/
│   │   │   ├── messages.rs
│   │   │   └── mod.rs
│   │   └── mod.rs
│   ├── cli/
│   │   └── mod.rs
│   ├── core/
│   │   ├── config.rs
│   │   ├── message.rs
│   │   └── mod.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── models/
│   │   ├── message.rs
│   │   └── mod.rs
│   ├── services/
│   │   ├── mod.rs
│   │   └── queue.rs
│   └── web/
│       └── mod.rs
└── tests/
    └── sender.rs
```

## 快速开始

### 前置条件

- Rust 1.70+（使用 `rustup` 安装）
- SQLite 3
- Redis（用于消息队列）

### 安装和运行

1. **克隆仓库**

   ```bash
   git clone <repository-url>
   cd ReChat-sender
   ```

2. **安装依赖**

   ```bash
   cargo build
   ```

3. **启动服务**

   ```bash
   cargo run
   ```

   服务默认运行在 `http://127.0.0.1:8080`

4. **访问 Web 界面**

   打开浏览器访问 `http://127.0.0.1:8080`

## API 文档

### 端点

- **POST /api/messages**：创建新消息
- **GET /api/messages/{id}**：查询消息状态

### 请求示例

创建消息：

```bash
curl -X POST http://localhost:8080/api/messages \
  -H "Content-Type: application/json" \
  -d '{
    "message_type": "Text",
    "content": "Hello, ReChat!",
    "recipient": "user1"
  }'
```

查询消息状态：

```bash
curl http://localhost:8080/api/messages/{message-id}
```

## 命令行工具

ReChat-sender 提供了命令行工具，用于管理消息和服务。

### 测试命令

```bash
cargo test
```

### 版本发布

使用 `cargo release` 命令发布新版本：

```bash
# 查看当前版本
cargo release version

# 发布补丁版本 (v1.0.0 → v1.0.1)
cargo release patch

# 发布次版本 (v1.0.0 → v1.1.0)
cargo release minor

# 发布主版本 (v1.0.0 → v2.0.0)
cargo release major
```

## 配置

配置文件位于 `src/core/config.rs`，默认配置如下：

- **服务器**：127.0.0.1:8080
- **Redis**：redis://localhost:6379，队列名：rechat_messages
- **数据库**：./rechat.db
- **发送器**：最大重试次数 3，重试间隔 5 秒，批处理大小 10

## 部署

详细的部署指南请参考 [DEPLOY.md](file:///workspace/DEPLOY.md)

## 贡献

欢迎贡献代码、报告问题或提出新功能建议！

## 许可证

本项目采用 MIT 许可证。