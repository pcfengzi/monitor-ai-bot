下面是 **`plugins/notification-center/README.md`** 的完整内容，你可以直接新建 / 覆盖这个文件。

---

# 📢 notification-center 插件

`notification-center` 是 **Monitor AI Bot** 中的「统一通知中心」插件，实现了文章里那种：

> 多业务 → 一个统一通知入口 → 多下游渠道（短信、邮件、Push、站内信）的智能路由 + 风控 + 追踪。

特点：

* ✅ **插件级微服务**：自己起 HTTP Server、自己连 DB、自己维护表，不修改系统级 `storage` / `bot-host` / `api-server` 代码
* ✅ **统一发送接口**：上游只需要调用一个 `send` API
* ✅ **风控 & 频控**：简单黑名单 + 频率限制，可扩展
* ✅ **多渠道路由**：短信 / 邮件 / Push / 站内信，按场景策略优先级路由
* ✅ **消息落库 & 追踪**：所有发送记录写入 SQLite，可按 `msg_id` 查询
* ✅ **模板渲染**：支持变量注入，后续可扩展为 DB 模板管理

---

## 1. 插件目录结构

```text
plugins/
  notification-center/
    ├── Cargo.toml
    └── src/
        ├── lib.rs          # 插件入口（meta + run_with_ctx + plugin_api_info）
        ├── router.rs       # 插件内部 HTTP API 路由（axum）
        ├── db.rs           # 插件自有 DB 封装（模板表 + history 表）
        ├── template.rs     # 模板渲染（{{var}} 替换）
        ├── risk_guard.rs   # 风控（黑名单 + 简单频控）
        └── channel/        # 下游渠道
            ├── mod.rs      # send_by_best_channel 路由逻辑
            ├── sms.rs      # 短信模拟发送
            ├── email.rs    # 邮件模拟发送
            ├── push.rs     # Push 模拟发送
            └── inbox.rs    # 站内信模拟发送
```

---

## 2. 架构 & 数据流

**整体架构（在 Monitor AI Bot 里）：**

```text
 上游业务 / 其它插件 / 前端
          │
          │  POST /plugin-api/notification-center/send
          ▼
+----------------------------+
|   api-server (网关层)      |
| - 识别 plugin-api 前缀     |
| - 路由到对应插件 HTTP 服务 |
+-------------▲--------------+
              │ HTTP 转发
              ▼
+----------------------------+
| notification-center 插件   |
| - run_with_ctx 中启动 HTTP |
| - /send /message /templates |
| - 风控 / 模板 / 路由       |
| - DB: 自己管理两张表       |
+-------------▲--------------+
              │
              │ 写入 SQLite (同一个 MONITOR_AI_DB_URL)
              ▼
          monitor_ai.db
        (notification_templates
         notification_history)
```

> 注意：
>
> * 插件使用与系统相同的 `MONITOR_AI_DB_URL`，但**只创建 & 操作自己的表**；
> * 不修改 `storage` crate、不改 `bot-host` / `api-server`；
> * 完全满足「插件是一个独立微服务」的设计目标。

---

## 3. 安装与构建

在仓库根目录（已有 workspace）下，执行：

```bash
cargo build -p notification-center
```

在 **dev 模式** 下，只要你在 `config.toml` 里设置了：

```toml
[plugin]
mode = "dev"
dev_dir = "target/debug"
name_pattern = "_monitor" # 记得保证编译后的 dll 名字里包含 _monitor 或你配置的关键词
```

并且 `Cargo.toml` 中插件名符合，例如：

```toml
[package]
name = "notification-center"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]
```

那么编译后会生成（Windows 为例）：

```text
target/debug/notification_center.dll
```

只要文件名匹配 `name_pattern`，`bot-host` 扫描时就会加载该插件，并通过 `plugin_api_info()` 注册到 `api-server` 的 **插件 API 网关**。

---

## 4. 环境变量 & 数据库

插件使用以下环境变量（与全局保持一致）：

```env
# 通知中心插件复用统一的 DB URL（SQLite 为例）
MONITOR_AI_DB_URL=sqlite://database/monitor_ai.db
```

> 插件在 `run_with_ctx` 时会：
>
> 1. 连接上述 DB
> 2. 在里面创建两张自己的表（如不存在）：
>
>    * `notification_templates`
>    * `notification_history`

### 4.1 `notification_templates`（通知模板）

```sql
CREATE TABLE IF NOT EXISTS notification_templates (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    scene       TEXT NOT NULL,      -- 业务场景：order_payed / welcome 等
    channel     TEXT NOT NULL,      -- 渠道：sms/email/push/inbox
    lang        TEXT NOT NULL,      -- 语言：zh-CN / en-US
    version     INTEGER NOT NULL,   -- 版本号
    content     TEXT NOT NULL,      -- 模板内容
    is_active   INTEGER NOT NULL,   -- 1=启用，0=停用
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
```

### 4.2 `notification_history`（发送记录）

```sql
CREATE TABLE IF NOT EXISTS notification_history (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    msg_id       TEXT NOT NULL UNIQUE,
    user_id      TEXT NOT NULL,
    scene        TEXT NOT NULL,
    channel      TEXT NOT NULL,
    content      TEXT NOT NULL,
    status       TEXT NOT NULL,      -- queued/sent/delivered/failed
    trace_id     TEXT NOT NULL,
    error        TEXT,
    created_at   TEXT NOT NULL,
    sent_at      TEXT,
    delivered_at TEXT
);
```

所有的 `INSERT/SELECT` 都在插件 `db.rs` 里封装完成，并使用 `sqlx::query` 动态 SQL，不需要 `DATABASE_URL` 编译期检查。

---

## 5. 插件导出的 ABI

插件必须导出以下函数，供 host 使用：

```rust
#[unsafe(no_mangle)]
pub extern "C" fn meta() -> PluginMeta;

#[unsafe(no_mangle)]
pub extern "C" fn plugin_api_info() -> PluginApiInfo;

#[unsafe(no_mangle)]
pub extern "C" fn run_with_ctx(ctx: *mut PluginContext);
```

### 5.1 `meta()`

返回插件元信息：

* name: `"notification-center"`
* version: `"0.1.0"`
* kind: `"notification"`

### 5.2 `plugin_api_info()`

告诉 `api-server`：

* 插件内部 HTTP 服务监听端口（例如 `5601`）
* 路由前缀（当前为 `/`，通过 `plugin-api/notification-center/...` 统一转发）

### 5.3 `run_with_ctx()`

* 由 host 调用
* 内部启动 tokio runtime 和 axum HTTP server
* 负责：

  1. `init_db()`：初始化插件自有 DB schema
  2. `start_server(port)`：挂载 `/send` `/message` 等 API

---

## 6. 插件提供的 HTTP API（通过 api-server 网关访问）

所有接口统一通过网关访问：

```text
http://{API_SERVER}/plugin-api/notification-center/...
```

例如：本地调试默认是：

```text
http://127.0.0.1:3001/plugin-api/notification-center/send
```

### 6.1 发送通知：`POST /send`

**请求：**

```json
POST /plugin-api/notification-center/send
Content-Type: application/json

{
  "user_id": "1001",
  "scene": "order_payed",
  "channel_hint": "sms",         // 可选：业务侧期望首选渠道
  "vars": {
    "order_id": "88888",
    "amount": "199"
  }
}
```

**响应：**

```json
{
  "msg_id": "ntf_6e5a1e6d-0a73-4b17-ae2e-8bd66d0c1234",
  "trace_id": "f8b7f0ce-3f63-40d9-b9f5-4a5f9d5aabcd",
  "status": "queued",           // blocked / queued / ...
  "channel": "sms"              // 实际使用的渠道
}
```

行为说明：

1. 按 `user_id + scene` 做基础风控（黑名单、频控等，见 `risk_guard.rs`）
2. 根据 `scene` 和内部策略选择渠道（`channel/mod.rs` 中 `send_by_best_channel`）
3. 使用模板系统 `template.rs` 渲染内容，插入变量 `{{order_id}}` / `{{amount}}` 等
4. 调用具体渠道（比如 `sms::send`）模拟发送
5. 把整条发送记录写入 `notification_history` 表（包含 `msg_id` / `trace_id` 等）

---

### 6.2 查询单条消息：`GET /message/{msg_id}`

```http
GET /plugin-api/notification-center/message/ntf_6e5a1e6d-0a73-4b17-ae2e-8bd66d0c1234
```

**响应示例：**

```json
{
  "msg_id": "ntf_6e5a1e6d-0a73-4b17-ae2e-8bd66d0c1234",
  "user_id": "1001",
  "scene": "order_payed",
  "channel": "sms",
  "content": "您的订单 88888 已支付成功，金额 ¥199。",
  "status": "queued",
  "trace_id": "f8b7f0ce-3f63-40d9-b9f5-4a5f9d5aabcd",
  "error": null,
  "created_at": "2025-11-26T14:50:31.081Z",
  "sent_at": "2025-11-26T14:50:31.081Z",
  "delivered_at": null
}
```

> 适合前端 / 运维 / 调试使用，可以做“消息详情页”。

---

### 6.3 模板列表：`GET /templates`

```http
GET /plugin-api/notification-center/templates
```

**响应示例：**

```json
{
  "items": [
    {
      "id": 1,
      "scene": "order_payed",
      "channel": "sms",
      "lang": "zh-CN",
      "version": 1,
      "content": "您的订单 {{order_id}} 已支付成功，金额 ¥{{amount}}。",
      "is_active": true,
      "created_at": "2025-11-26T13:00:00Z",
      "updated_at": "2025-11-26T13:00:00Z"
    }
  ]
}
```

> 可用于 **前端模板管理页面**：查看/编辑当前所有模板。

---

### 6.4 模板预览：`POST /template_render_preview`

```json
POST /plugin-api/notification-center/template_render_preview
{
  "scene": "order_payed",
  "vars": {
    "order_id": "88888",
    "amount": "199"
  }
}
```

**响应：**

```json
{
  "preview": "您的订单 88888 已支付成功，金额 ¥199。"
}
```

> 前端在编辑模板或填变量时可以调用，用来做“预览效果”。

---

### 6.5 统计：`GET /stats`

目前是一个简单的占位接口：

```json
GET /plugin-api/notification-center/stats

{
  "total_sent": 123,
  "success_rate": 0.98,
  "top_channels": ["sms", "email"]
}
```

后续可以根据 `notification_history` 做真实聚合统计（按天、按渠道、按 scene 等）。

---

## 7. 扩展点（下一步可以做的事情）

基于当前企业级插件骨架，你可以比较容易地继续演进：

1. **模板管理后台：**

   * 在 `dashboard-frontend` 里新增一个 “通知中心” 页面
   * 接 `/templates` 接口展示模板列表
   * 追加一个管理接口（可以直接打到插件，而不是改系统 storage）

2. **多渠道供应商配置：**

   * 在插件的 DB 中新增 `notification_channels` 表
   * 不同 provider（阿里云、腾讯云、Twilio）配置在表里
   * `channel/sms.rs` 从表中读取配置并真正调用三方短信 API

3. **更高级风控：**

   * 对 user + scene 做每天限发次数
   * 夜间免打扰
   * 根据失败率自动降级渠道

4. **与 AI 插件联动：**

   * 通知历史作为数据源
   * AI 帮你分析最佳发送时段 / 最佳渠道 / 最佳文案（A/B）

---

## 8. 总结

`notification-center` 插件实现了一个**真正独立、可插拔的通知中心微服务**：

* 不侵入核心框架，不修改系统级代码；
* 统一通知入口，屏蔽多渠道细节；
* 支持风控、路由、模板、记录、追踪；
* 与 Monitor AI Bot 的整体架构天然契合，可与其它插件（API 工作流 / AI 分析 / 探针等）配合，构建完整的 **“监控 + 通知 + 自动化运维”** 能力。

如果你要把这块做成产品的一个独立模块，这个插件就是一个非常好的“初版内核”。
