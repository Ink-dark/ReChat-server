# ReChat-sender API 文档

> 版本: 0.1.0 | 更新: 2026-04-26

---

## 目录

- [1. 概述](#1-概述)
- [2. HTTP REST API](#2-http-rest-api)
  - [2.1 创建消息](#21-创建消息)
  - [2.2 查询消息](#22-查询消息)
  - [2.3 健康检查](#23-健康检查)
- [3. WebSocket API](#3-websocket-api)
  - [3.1 连接](#31-连接)
  - [3.2 客户端 → 服务端](#32-客户端--服务端)
  - [3.3 服务端 → 客户端](#33-服务端--客户端)
- [4. Web 页面](#4-web-页面)

---

## 1. 概述

### 端点总览

| 端点 | 方法 | 类型 | 说明 |
|------|:---:|:---:|------|
| `/api/messages` | POST | HTTP | 创建消息 |
| `/api/messages/{id}` | GET | HTTP | 查询单条消息 |
| `/api/health` | GET | HTTP | 健康检查 |
| `/ws/client` | GET | WebSocket | 客户端实时通道 |
| `/` | GET | HTML | 首页 |
| `/send` | GET | HTML | 发送消息页面 |
| `/status` | GET | HTML | 消息状态查询页面 |

### 数据模型

**Message** 对象：

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | String (UUID v4) | 消息唯一 ID |
| `message_type` | String | `"Text"` / `"Image"` / `"File"` |
| `content` | String | 消息内容 |
| `recipient` | String | 接收者标识（群号/用户 ID/会话 ID） |
| `status` | String | `"Pending"` / `"Sending"` / `"Sent"` / `"Failed"` |
| `created_at` | u64 | 创建时间 (Unix 秒) |
| `updated_at` | u64 | 更新时间 (Unix 秒) |
| `retry_count` | u32 | 重试次数 |

### 错误响应格式

所有 HTTP 错误均返回 JSON：

```json
{"error": "错误描述信息"}
```

| HTTP 状态码 | 含义 |
|:---:|------|
| 201 | 创建成功 |
| 200 | 查询成功 |
| 400 | 请求参数错误 |
| 404 | 消息不存在 |
| 500 | 服务端错误 |

---

## 2. HTTP REST API

### 2.1 创建消息

> `POST /api/messages`

创建一个新消息，写入数据库后返回消息详情。

**请求体** (JSON)：

| 字段 | 类型 | 必填 | 说明 |
|------|------|:---:|------|
| `message_type` | String | ✅ | `"Text"` / `"Image"` / `"File"` |
| `content` | String | ✅ | 消息内容 |
| `recipient` | String | ✅ | 接收者标识 |

**请求示例**：

```bash
curl -X POST http://localhost:8080/api/messages \
  -H "Content-Type: application/json" \
  -d '{
    "message_type": "Text",
    "content": "Hello, ReChat!",
    "recipient": "group_123456"
  }'
```

**成功响应** `201 Created`：

```json
{
  "id": "a1b2c3d4-...",
  "message_type": "Text",
  "content": "Hello, ReChat!",
  "recipient": "group_123456",
  "status": "Pending",
  "created_at": 1714000000,
  "updated_at": 1714000000,
  "retry_count": 0
}
```

**错误响应**：

| 状态码 | 条件 | 示例 |
|:---:|------|------|
| `400` | 无效的 message_type | `{"error": "Invalid message type"}` |
| `500` | 数据库未初始化 | `{"error": "Repository not initialized"}` |
| `500` | 数据库写入失败 | `{"error": "..."}` |

---

### 2.2 查询消息

> `GET /api/messages/{id}`

根据 ID 查询单条消息。

**路径参数**：

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | String | 消息 UUID |

**请求示例**：

```bash
curl http://localhost:8080/api/messages/a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

**成功响应** `200 OK`：

```json
{
  "id": "a1b2c3d4-...",
  "message_type": "Text",
  "content": "Hello, ReChat!",
  "recipient": "group_123456",
  "status": "Sent",
  "created_at": 1714000000,
  "updated_at": 1714000005,
  "retry_count": 0
}
```

**错误响应**：

| 状态码 | 条件 | 示例 |
|:---:|------|------|
| `404` | 消息不存在 | `{"error": "Message not found"}` |
| `500` | 数据库未初始化 | `{"error": "Repository not initialized"}` |

---

### 2.3 健康检查

> `GET /api/health`

检查服务是否正常运行。

**请求示例**：

```bash
curl http://localhost:8080/api/health
```

**成功响应** `200 OK`：

```json
{"status": "ok"}
```

---

## 3. WebSocket API

> `GET /ws/client`

WebSocket 连接用于客户端实时接收消息推送和发送指令。协议为自定义 JSON 格式。

### 3.1 连接

```
ws://<host>:<port>/ws/client
```

客户端通过 WebSocket 协议连接，服务端自动分配 session ID。连接建立后默认不订阅任何平台，需发送 `subscribe` 命令。

### 3.2 客户端 → 服务端

所有指令均为 JSON 对象，通过 `type` 字段区分。

---

#### 3.2.1 subscribe — 订阅平台/会话

订阅后，该平台的新消息会实时推送到此连接。

```json
{
  "type": "subscribe",
  "platforms": ["qq", "wechat"],
  "conversations": ["group_123", "private_456"]
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|:---:|------|
| `type` | String | ✅ | `"subscribe"` |
| `platforms` | String[] | ❌ | 要订阅的平台列表，如 `["qq"]` |
| `conversations` | String[] | ❌ | 要订阅的会话 ID 列表 |

---

#### 3.2.2 unsubscribe — 取消订阅

```json
{
  "type": "unsubscribe",
  "platforms": ["qq"],
  "conversations": ["group_123"]
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|:---:|------|
| `type` | String | ✅ | `"unsubscribe"` |
| `platforms` | String[] | ❌ | 要取消的平台 |
| `conversations` | String[] | ❌ | 要取消的会话 |

---

#### 3.2.3 send_message — 发送消息

通过指定平台的 Adapter 发送消息。流程：保存数据库 → 转发 Adapter → 广播给订阅客户端。

```json
{
  "type": "send_message",
  "data": {
    "platform": "qq",
    "conversation": "group_123456",
    "content": "Hello from ReChat!",
    "message_type": "Text"
  }
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|:---:|------|
| `type` | String | ✅ | `"send_message"` |
| `data.platform` | String | ✅ | 目标平台，如 `"qq"` |
| `data.conversation` | String | ✅ | 目标会话 ID（群号/用户 ID） |
| `data.content` | String | ✅ | 消息内容 |
| `data.message_type` | String | ❌ | `"Text"`（默认）/ `"Image"` / `"File"` |

**成功响应**：

```json
{
  "type": "ack",
  "status": "sent",
  "message_id": "a1b2c3d4-..."
}
```

**错误响应**：

```json
{"type": "error", "error": "Missing data field"}
```

---

### 3.3 服务端 → 客户端

服务端会主动推送以下事件。

#### 3.3.1 new_message — 新消息推送

当有 Adapter 接到新消息或客户端通过 `send_message` 发送成功时广播。

```json
{
  "type": "new_message",
  "data": {
    "id": "a1b2c3d4-...",
    "platform": "qq",
    "conversation": "group_123456",
    "conversation_name": "技术交流群",
    "content": "Hello, ReChat!",
    "message_type": "Text",
    "sender": {
      "id": "789",
      "name": "张三"
    },
    "created_at": 1714000000
  }
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `data.id` | String | 消息 UUID |
| `data.platform` | String | 来源平台 |
| `data.conversation` | String | 会话 ID |
| `data.conversation_name` | String? | 会话名称（可选，群名等） |
| `data.content` | String | 消息内容 |
| `data.message_type` | String | `Text` / `Image` / `File` |
| `data.sender` | Object? | 发送者信息（可选） |
| `data.sender.id` | String | 发送者 ID |
| `data.sender.name` | String | 发送者名称 |
| `data.created_at` | u64 | 创建时间 (Unix 秒) |

#### 3.3.2 adapter_status — 平台连接状态

当 Adapter 连接/断开时广播。

```json
{
  "type": "adapter_status",
  "data": {
    "platform": "qq",
    "status": "Connected"
  }
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `data.platform` | String | 平台名称 |
| `data.content` | String | 状态描述 |

#### 3.3.3 error — 错误

```json
{"type": "error", "error": "错误描述"}
```

---

## 4. Web 页面

| 路径 | 说明 |
|------|------|
| `GET /` | 首页 — 导航链接 |
| `GET /send` | 消息发送表单 — 选择消息类型、填写接收者和内容，提交到 `/api/messages` |
| `GET /status` | 消息状态查询 — 输入消息 ID，调用 `/api/messages/{id}` 显示结果 |

Web 页面为静态 HTML，内嵌 JavaScript 通过 `fetch` 调用 REST API。适合开发调试和简单管理。
