# ReChat-sender 部署文档

## 1. 环境要求

- Rust 1.70+（推荐使用最新稳定版本）
- Cargo（Rust 包管理工具）
- Windows 操作系统（目前仅支持Windows）

## 2. 构建步骤

### 2.1 克隆代码库

```bash
git clone <repository-url>
cd ReChat-sender
```

### 2.2 构建项目

```bash
# 开发环境构建
cargo build

# 生产环境构建（优化编译）
cargo build --release
```

### 2.3 使用构建脚本

项目根目录下提供了构建脚本，可以简化构建过程：

```bash
# 运行构建脚本
rustc scripts/build.rs && ./build
```

构建完成后，可执行文件将位于 `build/rechat-sender` 目录。

## 3. 运行服务

### 3.1 直接运行

```bash
# 开发环境
cargo run

# 生产环境
./target/release/rechat-sender
```

### 3.2 作为服务运行

在 Windows 系统中，可以使用 `sc` 命令将 ReChat-sender 注册为系统服务：

```powershell
# 以管理员身份运行 PowerShell
sc create ReChatSender binPath= "C:\path\to\rechat-sender.exe" start= auto
sc start ReChatSender
```

## 4. 配置

### 4.1 配置文件

ReChat-sender 使用默认配置，也可以通过创建 `config.json` 文件来自定义配置：

```json
{
  "server": {
    "host": "127.0.0.1",
    "port": 8080,
    "workers": 1
  },
  "redis": {
    "url": "redis://localhost:6379",
    "queue_name": "rechat_messages"
  },
  "database": {
    "path": "./rechat.db"
  },
  "sender": {
    "max_retries": 3,
    "retry_interval": 5,
    "batch_size": 10
  }
}
```

### 4.2 环境变量

也可以通过环境变量来覆盖配置：

- `RECHAT_SERVER_HOST`：服务器主机
- `RECHAT_SERVER_PORT`：服务器端口
- `RECHAT_DATABASE_PATH`：数据库文件路径

## 5. API 接口

### 5.1 发送消息

```bash
POST /api/messages
Content-Type: application/json

{
  "message_type": "Text",
  "content": "Hello, world!",
  "recipient": "user1"
}
```

### 5.2 获取消息状态

```bash
GET /api/messages/{id}
```

### 5.3 健康检查

```bash
GET /api/health
```

## 6. Web 界面

ReChat-sender 提供了基于 Web 的图形化界面：

- 首页：`http://localhost:8080/`
- 发送消息：`http://localhost:8080/send`
- 查看状态：`http://localhost:8080/status`

## 7. 命令行界面

ReChat-sender 也提供了命令行界面：

### 7.1 发送消息

```bash
rechat-sender send --type text --recipient user1 --content "Hello, world!"
```

### 7.2 查看消息状态

```bash
rechat-sender status --id <message-id>
```

## 8. 故障排除

### 8.1 常见问题

1. **端口被占用**：修改配置文件中的 `server.port` 字段，使用不同的端口。

2. **数据库文件权限**：确保应用程序有足够的权限读写数据库文件。

3. **Redis 连接失败**：确保 Redis 服务正在运行，并且配置文件中的 Redis URL 正确。

### 8.2 日志

ReChat-sender 的日志输出到标准输出，可通过重定向来保存日志：

```bash
./rechat-sender > rechat.log 2>&1
```

## 9. 升级

### 9.1 更新代码

```bash
git pull
cargo build --release
```

### 9.2 重启服务

如果以服务方式运行，需要重启服务：

```powershell
sc stop ReChatSender
sc start ReChatSender
```

## 10. 安全性

### 10.1 注意事项

- 目前 ReChat-sender 没有实现身份验证，建议在内部网络中使用。
- 生产环境中建议配置防火墙，限制访问端口。
- 定期备份数据库文件，防止数据丢失。

### 10.2 未来改进

- 添加身份验证和授权机制
- 实现 HTTPS 支持
- 添加数据加密功能
