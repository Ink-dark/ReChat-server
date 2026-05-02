# ReChat-sender 部署文档

## 1. 项目特点

ReChat-sender 是一个纯 Rust 编写的消息聚合服务，具有以下优势：

- **零外部依赖**：所有依赖都静态编译进二进制文件，无需安装任何运行时库
- **开箱即用**：SQLite 数据库使用 bundled 模式，无需安装数据库服务
- **跨平台支持**：支持 Windows、Linux、macOS
- **高性能**：基于 Tokio 异步运行时和 Actix-web 框架

## 2. 环境要求

仅需安装 Rust 工具链：

- Rust 1.70+ (推荐使用最新稳定版)
- Cargo (随 Rust 一起安装)

### 安装 Rust

**Linux/macOS**：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Windows**：
下载并运行 [rustup-init.exe](https://win.rustup.rs/)

## 3. 构建项目

### 3.1 克隆代码库

```bash
git clone <repository-url>
cd ReChat-sender
```

### 3.2 标准构建（推荐）

构建完全无外部依赖的二进制文件：

```bash
# 开发环境构建
cargo build

# 生产环境构建（优化编译）
cargo build --release
```

构建成功后，二进制文件位于：
- Linux/macOS: `./target/release/rechat-sender`
- Windows: `.\target\release\rechat-sender.exe`

### 3.3 可选特性

如果需要 Redis 支持，可以启用 `redis-support` 特性：

```bash
cargo build --release --features redis-support
```

## 4. 运行服务

### 4.1 直接运行

```bash
# Linux/macOS
./target/release/rechat-sender

# Windows
.\target\release\rechat-sender.exe
```

### 4.2 使用配置文件

可以通过 JSON 配置文件自定义配置：

```bash
./target/release/rechat-sender --config config.json
```

`config.json` 示例：

```json
{
  "server": {
    "host": "0.0.0.0",
    "port": 8080,
    "workers": 4,
    "web_ui": true,
    "web_ui_port": 8081
  },
  "database": {
    "path": "./rechat.db",
    "max_connections": 5,
    "connection_timeout": 3
  },
  "sender": {
    "max_retries": 3,
    "retry_interval": 5,
    "batch_size": 10,
    "concurrency": 5
  },
  "adapters": [],
  "plugins": []
}
```

## 5. 跨平台构建

### 5.1 构建 Windows 版本（在 Linux/macOS 上）

```bash
# 安装目标平台
rustup target add x86_64-pc-windows-gnu

# 安装 MinGW-w64 工具链（Ubuntu）
sudo apt install mingw-w64

# 构建
cargo build --release --target x86_64-pc-windows-gnu
```

### 5.2 构建 Linux 版本（在 Windows 上）

使用 WSL (Windows Subsystem for Linux) 是最简单的方式。

### 5.3 构建 macOS 版本

在 macOS 上直接构建即可：

```bash
# Intel 芯片
cargo build --release --target x86_64-apple-darwin

# Apple Silicon (M1/M2)
cargo build --release --target aarch64-apple-darwin

# 通用二进制 (Universal 2)
rustup target add x86_64-apple-darwin aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
lipo -create -output ./target/release/rechat-sender-universal \
  ./target/x86_64-apple-darwin/release/rechat-sender \
  ./target/aarch64-apple-darwin/release/rechat-sender
```

## 6. 作为服务运行

### 6.1 Linux (systemd)

创建 `/etc/systemd/system/rechat-sender.service`：

```ini
[Unit]
Description=ReChat Message Sender Service
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/rechat-sender
ExecStart=/opt/rechat-sender/rechat-sender
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

启用并启动服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable rechat-sender
sudo systemctl start rechat-sender
sudo systemctl status rechat-sender
```

### 6.2 Windows (系统服务)

使用 NSSM (Non-Sucking Service Manager) 将其注册为 Windows 服务：

```powershell
# 下载 NSSM
# 以管理员身份运行 PowerShell

nssm install ReChatSender
# 在弹出的对话框中设置：
# Path: C:\path\to\rechat-sender.exe
# Startup directory: C:\path\to\

nssm start ReChatSender
```

或者使用 Windows 的 `sc` 命令：

```powershell
sc create ReChatSender binPath= "C:\path\to\rechat-sender.exe" start= auto
sc start ReChatSender
```

### 6.3 macOS (launchd)

创建 `~/Library/LaunchAgents/com.rechat.sender.plist`：

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.rechat.sender</string>
    <key>ProgramArguments</key>
    <array>
        <string>/opt/rechat-sender/rechat-sender</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>WorkingDirectory</key>
    <string>/opt/rechat-sender</string>
</dict>
</plist>
```

加载服务：

```bash
launchctl load ~/Library/LaunchAgents/com.rechat.sender.plist
```

## 7. API 接口

### 7.1 健康检查

```bash
GET /api/health
```

### 7.2 发送消息

```bash
POST /api/messages
Content-Type: application/json

{
  "message_type": "Text",
  "content": "Hello, world!",
  "recipient": "user1"
}
```

### 7.3 获取消息列表

```bash
GET /api/messages?offset=0&limit=10
```

### 7.4 获取单条消息

```bash
GET /api/messages/{id}
```

## 8. Web 界面

服务启动后，可以通过浏览器访问 Web 管理界面：

- 首页: `http://localhost:8080`
- 使用访问令牌登录（启动时会在控制台显示）

## 9. 日志和故障排除

### 9.1 查看日志

日志输出到标准输出，可以重定向到文件：

```bash
./rechat-sender > rechat.log 2>&1
```

使用 systemd 时：

```bash
sudo journalctl -u rechat-sender -f
```

### 9.2 常见问题

1. **端口被占用**
   
   修改配置文件中的 `server.port` 字段，或使用环境变量：
   ```bash
   export RECHAT_SERVER_PORT=9090
   ./rechat-sender
   ```

2. **数据库文件权限**
   
   确保程序有读写数据库文件的权限，数据库文件位置由 `database.path` 配置。

3. **访问令牌**
   
   每次启动服务都会生成新的访问令牌，检查控制台输出获取。

## 10. 安全建议

1. **绑定到特定 IP**
   
   生产环境中建议绑定到 `127.0.0.1` 或内网 IP，避免暴露到公网。

2. **防火墙配置**
   
   使用防火墙限制访问端口。

3. **数据备份**
   
   定期备份 `rechat.db` 数据库文件。

## 11. 升级

```bash
# 拉取最新代码
git pull

# 重新构建
cargo build --release

# 停止旧服务
sudo systemctl stop rechat-sender

# 替换二进制文件
cp target/release/rechat-sender /opt/rechat-sender/

# 启动新服务
sudo systemctl start rechat-sender
```
