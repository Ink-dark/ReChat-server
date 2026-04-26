# OneBot v11 Adapter 开发规划

> 目标：实现 NapCat 等基于 OneBot v11 协议的消息平台接入
> 日期：2026-04-26
> 状态：规划中

---

## 一、背景知识

### OneBot 协议要点
| 概念 | 说明 |
|------|------|
| **通信方向** | 反向 WebSocket — NapCat 作为客户端连接我们的服务端 |
| **连接地址** | NapCat 配置 `ws://<host>:<port>/onebot/v11/ws` |
| **消息格式** | JSON，支持 Array 和 String 两种消息格式 |
| **事件** | NapCat → ReChat：`post_type`: `message` / `notice` / `request` |
| **动作** | ReChat → NapCat：`{"action": "...", "params": {...}, "echo": "..."}` |
| **响应** | NapCat → ReChat：`{"status": "ok/failed", "retcode": 0, "data": {...}}` |

### OneBot 核心动作（API）
| 动作 | 用途 |
|------|------|
| `send_private_msg` | 发送私聊消息 |
| `send_group_msg` | 发送群聊消息 |
| `send_msg` | 统一发送（自动判断私聊/群聊） |
| `delete_msg` | 撤回消息 |
| `get_msg` | 获取消息详情 |
| `get_group_info` | 获取群信息 |
| `get_group_member_info` | 获取群成员信息 |

### OneBot 消息段格式
```json
[{"type": "text", "data": {"text": "Hello"}}, {"type": "image", "data": {"file": "http://..."}}]
```

### 事件类型
| 事件 | post_type | 说明 |
|------|-----------|------|
| 私聊消息 | `message` + `message_type: private` | 用户私聊机器人 |
| 群聊消息 | `message` + `message_type: group` | 群内消息 |
| 群通知 | `notice` | 加群/退群/禁言等 |
| 好友请求 | `request` | 好友添加请求 |

---

## 二、架构设计

### 2.1 整体数据流

```
                    ┌──────── 入站 ────────┐    ┌──────── 出站 ────────┐
                    │                      │    │                      │
  NapCat ──WS──→ /onebot/v11/ws          HTTP API                   /ws/client ←──WS── 前端
    │                    │                POST /api/messages              │
    ▼                    ▼                      │                        ▼
 事件 JSON          OneBot WS Handler         创建消息              Client WS Handler
    │                    │                      │                        │
    ▼                    ▼                      ▼                        ▼
 解析 → MessageEvent   OneBotAdapter         Message (DB)          MessageBroadcaster
          │              .receive_message()      │                  (广播中枢)
          │                    │                 │                        ▲
          ▼                    ▼                 │                        │
     OneBotAdapter ◀── Message (内部) ◀─────────┘                        │
     .send_message()                                                      │
          │                                                               │
          ▼                                                               │
     动作 JSON                                                            │
          │                                                               │
          ▼                                                               │
  NapCat ←──WS── (send_msg/send_group_msg)                                │
                                                                          │
  ┌───────────────────────────────────────────────────────────────────────┘
  │
  ▼
Plugin Pipeline → 保存 DB ──→ broadcast ──→ 所有订阅客户端
```

**两条 WebSocket 通道**：

| 通道 | 端点 | 方向 | 协议 | 发起方 |
|------|------|------|------|:---:|
| 平台通道 | `/onebot/v11/ws` | NapCat ← → ReChat | OneBot v11 | NapCat 连接我们 |
| 客户端通道 | `/ws/client` | 客户端 ← → ReChat | 自定义 JSON | 前端/客户端连接我们 |

**广播中枢 `MessageBroadcaster`** 是两条通道的交汇点：
- Adapter 收到消息 → `broadcaster.broadcast(msg)` → 所有订阅该平台的客户端实时收到
- HTTP API 创建消息 → 同样广播（用于多端同步）

### 2.2 客户端 WS 消息格式

```json
// ========== 服务端 → 客户端 ==========

// 新消息推送
{"type": "new_message", "data": {
    "id": "uuid",
    "platform": "qq",
    "conversation": "group_123456",
    "conversation_name": "技术交流群",
    "content": "Hello, ReChat!",
    "message_type": "Text",
    "sender": {"id": "789", "name": "张三", "role": "member"},
    "created_at": 1714000000
}}

// 消息状态变更
{"type": "message_status", "data": {
    "id": "uuid", "status": "Sent", "updated_at": 1714000001
}}

// 平台连接状态
{"type": "adapter_status", "data": {
    "platform": "qq", "status": "Connected"
}}

// ========== 客户端 → 服务端 ==========

// 订阅指定平台/会话
{"type": "subscribe", "platforms": ["qq"], "conversations": ["group_123"]}

// 取消订阅
{"type": "unsubscribe", "platforms": ["qq"]}

// 向平台发送消息
{"type": "send_message", "data": {
    "platform": "qq",
    "conversation": "group_123456",
    "content": "回复消息",
    "message_type": "Text"
}}
```

### 2.3 模块结构

```
src/
├── adapters/                    # 新增：适配器实现目录
│   ├── mod.rs                   # 模块声明 + 统一导出
│   └── onebot/                  # OneBot 适配器
│       ├── mod.rs               # 子模块声明
│       ├── protocol.rs          # 协议数据模型
│       ├── adapter.rs           # Adapter trait 实现
│       └── ws.rs                # WebSocket 端点处理器（平台通道）
├── api/
│   ├── endpoints/
│   │   ├── messages.rs          # 已有：HTTP API
│   │   └── ws_client.rs         # 新增：客户端 WS 端点
│   ├── mod.rs                   # 修改：注册 /ws/client 路由
│   └── ...
├── core/
│   ├── adapter.rs               # 修改：Adapter trait 增强
│   ├── broadcaster.rs           # 新增：消息广播中枢
│   ├── config.rs                # 已有 AdapterConfig
│   └── ...
├── lib.rs                       # 修改：新增 adapters 模块
└── main.rs                      # 修改：初始化 Broadcaster + OneBot adapter
```

### 2.4 广播中枢 `MessageBroadcaster`

```rust
// core/broadcaster.rs — 消息广播中枢

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use actix_web::web;
use tokio::sync::broadcast;

pub struct ClientSession {
    pub id: String,
    pub platforms: HashSet<String>,        // 订阅的平台 ["qq", "wechat"]
    pub conversations: HashSet<String>,    // 订阅的会话 ["group_123", "private_456"]
    pub sender: tokio::sync::mpsc::UnboundedSender<String>,
}

pub struct MessageBroadcaster {
    sessions: Arc<Mutex<HashMap<String, ClientSession>>>,
}

impl MessageBroadcaster {
    pub fn new() -> Self { ... }
    
    // 客户端连接时注册
    pub fn register(&self, session: ClientSession) { ... }
    
    // 客户端断开时注销
    pub fn unregister(&self, session_id: &str) { ... }
    
    // Adapter 收到消息后调用 —— 广播给所有订阅该平台的客户端
    pub fn broadcast_message(&self, msg: &BroadcastMessage) {
        for (_, session) in self.sessions.lock().unwrap().iter() {
            if session.platforms.contains(&msg.platform) {
                let json = serde_json::to_string(&msg).unwrap();
                let _ = session.sender.send(json);
            }
        }
    }
    
    // 平台状态变更广播
    pub fn broadcast_adapter_status(&self, platform: &str, status: &str) { ... }
}

struct BroadcastMessage {
    r#type: String,             // "new_message"
    data: BroadcastMessageData,
    platform: String,           // "qq" | "wechat" | ...
}

struct BroadcastMessageData {
    id: String,
    platform: String,
    conversation: String,
    conversation_name: Option<String>,
    content: String,
    message_type: String,
    sender: Option<SenderInfo>,
    created_at: u64,
}
```

### 2.5 连接场景示例

```
时间线：用户 A 在前端看到 QQ 群消息并回复

1. QQ 群有新消息
   → NapCat 推送事件到 /onebot/v11/ws
   
2. OneBot WS Handler 解析 → Message
   → save DB
   → broadcaster.broadcast_message(msg)
   
3. 前端 Web UI（已连接 /ws/client）
   → 收到 {"type": "new_message", ...}
   → 页面实时显示新消息
   
4. 用户在前端点击回复，输入内容
   → 前端 WS send: {"type": "send_message", "platform": "qq", ...}
   
5. Client WS Handler
   → 解析 → 创建 Message (DB)
   → OneBotAdapter.send_message(msg)
   → 构造 send_group_msg action → WS 发回 NapCat
   
6. NapCat 收到动作 → 发送到 QQ 群
```

### 2.3 核心数据结构

```rust
// protocol.rs — OneBot 协议层

// 动作请求 (ReChat → NapCat)
struct ActionRequest {
    action: String,
    params: serde_json::Value,
    echo: Option<String>,
}

// 动作响应 (NapCat → ReChat)
struct ActionResponse {
    status: String,       // "ok" | "failed"
    retcode: i64,
    data: serde_json::Value,
    echo: Option<String>,
}

// 事件 (NapCat → ReChat)
struct Event {
    time: i64,
    self_id: i64,
    post_type: String,    // "message" | "notice" | "request"
    // ... 根据 post_type 展开不同字段
}

// 消息段
struct MessageSegment {
    r#type: String,       // "text" | "image" | "at" | "reply" | ...
    data: HashMap<String, String>,
}

// 接收到的消息事件
struct MessageEvent {
    message_type: String,  // "private" | "group"
    sub_type: String,      // "friend" | "normal" | ...
    message_id: i64,
    user_id: i64,
    message: Vec<MessageSegment>,
    raw_message: String,
    // 私聊字段
    sender: Option<Sender>,
    // 群聊字段
    group_id: Option<i64>,
}
```

---

## 三、实施步骤

### Phase 0: 广播中枢 + 客户端 WS（输入/输出通道的基石）

这是整个消息平台的**核心基础设施**，必须在接入任何平台前就绪：

#### 0.1 广播中枢 (`core/broadcaster.rs`)
- [ ] `MessageBroadcaster` 结构体 + `ClientSession` 定义
- [ ] `register(session)` / `unregister(id)` — 客户端连接管理
- [ ] `broadcast_message(msg)` — 按平台/会话订阅广播
- [ ] `broadcast_adapter_status(platform, status)` — 平台状态变更通知
- [ ] 清理断开连接的客户端（惰性或定时）

#### 0.2 客户端 WS 端点 (`api/endpoints/ws_client.rs`)
- [ ] actix-web WebSocket handler，端点 `GET /ws/client`
- [ ] 握手阶段：解析客户端发来的 `subscribe` 消息
- [ ] 将连接注册到 `MessageBroadcaster`
- [ ] 接收客户端指令：`subscribe` / `unsubscribe` / `send_message`
- [ ] 将 `send_message` 转发给对应 Adapter
- [ ] 断线时自动 `unregister`

#### 0.3 集成
- [ ] `src/core/mod.rs` 添加 `pub mod broadcaster;`
- [ ] `src/api/mod.rs` 注册 `/ws/client`
- [ ] `src/main.rs` 初始化 `Arc<MessageBroadcaster>` 并注入 actix Data

### Phase 1: 协议层 (`protocol.rs`)
- [ ] 定义 `ActionRequest` / `ActionResponse` 结构体
- [ ] 定义 `Event` 枚举（区分 message/notice/request）
- [ ] 定义 `MessageSegment` — 消息段解析
- [ ] 定义 `MessageEvent` / `NoticeEvent` / `RequestEvent`
- [ ] 实现 `MessageSegment` → `Message` 转换（OneBot → 内部）
- [ ] 实现 `Message` → `MessageSegment[]` 转换（内部 → OneBot）
- [ ] 实现 JSON 解析/序列化（基于 serde）

### Phase 2: WebSocket 端点 (`ws.rs`)
- [ ] 实现 actix-web WebSocket handler
- [ ] 连接管理：新连接 → 注册到 adapter
- [ ] 消息接收：WS 帧 → JSON 解析 → 事件分发
- [ ] 消息发送：Adapter → JSON 序列化 → WS 帧
- [ ] 心跳维持
- [ ] 断线重连处理（由 NapCat 侧发起）

### Phase 3: Adapter 实现 (`adapter.rs`)
- [ ] `OneBotAdapter` struct：持有 WS sender、消息缓冲区、配置
- [ ] `Adapter::name()` → `"onebot"`
- [ ] `Adapter::start()` → 空操作（WS 由 actix 管理）
- [ ] `Adapter::stop()` → 断开 WS
- [ ] `Adapter::send_message()` → Message → OneBot action → WS send
- [ ] `Adapter::receive_message()` → 从缓冲区读取消息
- [ ] 事件 → Message 转换逻辑（QQ ID → recipient 映射等）

### Phase 4: 集成
- [ ] `src/lib.rs` 添加 `pub mod adapters;`
- [ ] `src/adapters/mod.rs` 声明 `pub mod onebot;`
- [ ] `src/api/mod.rs` 注册 `/onebot/v11/ws` 路由
- [ ] `src/main.rs` 创建 OneBotAdapter 实例并注册到 AdapterManager
- [ ] 从配置读取 OneBot 参数（token、端口等）

### Phase 5: 验证
- [ ] `cargo check` 无错误
- [ ] `cargo clippy` 无新增 warning
- [ ] 编写 OneBot adapter 单元测试
- [ ] 确保配置向后兼容（无 OneBot 配置时正常启动）

---

## 四、风险与注意事项

| 风险 | 缓解 |
|------|------|
| Adapter trait 是同步的，但 WS 是异步的 | 用 `Arc<Mutex<VecDeque>>` 做消息缓冲 |
| 多个 NapCat 实例连接 | 每个连接独立 Session，adapter 管理多个 Session |
| 消息段格式复杂（图片/语音等） | Phase 1 先支持 text + image，后续扩展 |
| OneBot 协议字段命名不规范（驼峰/下划线混用） | 用 serde `rename` 适配 |

---

## 五、预期成果

- NapCat 可通过配置连接 ReChat 服务端
- 支持接收私聊/群聊文本消息
- 支持通过 API 发送消息到 QQ
- 架构可扩展至其他 OneBot 实现（LLOneBot、go-cqhttp 等）
