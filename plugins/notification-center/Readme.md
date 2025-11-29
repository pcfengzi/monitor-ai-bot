# 📢 notification-center 插件（企业级通知中心）

`notification-center` 是 **Monitor AI Bot** 中的「统一通知中心」插件，实现了文章里那种：

> 多业务 → 一个统一通知入口 → 多下游渠道（短信、邮件、Push、站内信）的智能路由 + 风控 + 追踪。

特点：

- ✅ **插件级微服务**：自己起 HTTP Server、自己连 DB、自己维护表，不修改系统级 `storage` / `bot-host` / `api-server` 代码
- ✅ **统一发送接口**：上游只需要调用一个 `send` API
- ✅ **风控 & 频控**：黑名单 + 频率限制 + 夜间免打扰，可扩展
- ✅ **多渠道路由 + 降级**：短信 / 邮件 / Push / 站内信，按场景策略优先级路由，渠道不可用自动降级
- ✅ **消息落库 & 生命周期追踪**：`waiting → processing → sent/delivered/failed/blocked` 全流程状态机
- ✅ **模板渲染**：支持 `{{var}}` 变量注入，后续可以扩展管理后台
- ✅ **指标上报**：通过 `plugin-api` 的 metric 能力，对发送量、成功率、耗时等做监控

---

## 1. 插件目录结构

```text
plugins/
  notification-center/
    ├── Cargo.toml
    └── src/
        ├── lib.rs          # 插件入口（meta + run_with_ctx）
        ├── router.rs       # 插件内部 HTTP API 路由 + 队列 worker
        ├── db.rs           # 插件自有 DB 封装（模板表 + history 表 + 用户偏好）
        ├── template.rs     # 模板渲染（{{var}} 替换）
        ├── risk_guard.rs   # 风控（黑名单 + 频控 + 夜间免打扰）
        ├── metrics_bridge.rs # 和 host 的日志 / metric 桥接
        ├── types.rs        # 公共类型（Channel / Status / 请求结构等）
        └── channel/        # 下游发送渠道
            ├── mod.rs      # 渠道优先级 + 降级策略
            ├── sms.rs      # 短信发送（当前为 mock，可接第三方 SDK）
            ├── email.rs    # 邮件发送（当前为 mock，可接 SMTP/服务商）
            ├── push.rs     # App Push 发送（当前为 mock）
            └── inbox.rs    # 站内信发送（当前为 mock）
```

---

## 2. 插件在整体架构中的位置

在 Monitor AI Bot 中，大概是这样：

```text
上游业务 / 其它插件 / 前端 / 探针 agent
          │
          │  POST /plugin-api/notification-center/send
          ▼
+----------------------------+
|   api-server (网关层)      |
| - 识别 /plugin-api 前缀     |
| - 路由到对应插件 HTTP 服务 |
+-------------▲--------------+
              │ HTTP 转发
              ▼
+----------------------------+
| notification-center 插件   |
| - run_with_ctx 启动 HTTP  |
| - /send /message /templates|
| - 风控 / 模板 / 路由       |
| - 自己维护通知相关表       |
+-------------▲--------------+
              │
              │ 共享 MONITOR_AI_DB_URL
              ▼
        SQLite / Postgres
   (notification_templates /
    notification_history /
    notification_user_pref)
```

> 关键点：
>
> - 插件使用与系统同一个 `MONITOR_AI_DB_URL`，但**只创建和操作自己的表**；
> - 不修改 `storage` crate、不改 `bot-host` / `api-server` 源码；
> - 通过 `run_with_ctx` + `plugin_api` 的能力与 host 解耦，是一个真正**可插拔的微服务插件**。

---

## 3. 安装与构建

在仓库根目录执行：

```bash
cargo build -p notification-center
```

确保在 `config.toml` 中配置了插件扫描规则（示例）：

```toml
[plugin]
mode = "dev"                 # dev | prod
dev_dir = "target/debug"
prod_dir = "plugins-bin"
name_pattern = "_monitor"    # 保证生成的 dll 名字里包含该关键字
default_interval = 5
```

`Cargo.toml` 示例：

```toml
[package]
name = "notification-center"
version = "0.2.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
plugin-api = { path = "../../plugin-api" }
# 其它依赖见 Cargo.toml
```

Windows 下构建完成后会生成：

```text
target/debug/notification_center.dll
```

只要文件名命中 `name_pattern`，`bot-host` 就会在启动时扫描并加载该插件，然后通过 `run_with_ctx` 启动通知中心服务。

---

## 4. 环境变量 & 数据库表

### 4.1 环境变量

```env
# 复用统一的 DB（SQLite / Postgres 等）
MONITOR_AI_DB_URL=sqlite://database/monitor_ai.db

# 插件内部 HTTP 监听端口（由 plugin_api_info 暴露给 api-server）
NC_PLUGIN_PORT=5601
```

### 4.2 插件自有表结构

#### 4.2.1 通知模板表 `notification_templates`

```sql
CREATE TABLE IF NOT EXISTS notification_templates (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    scene       TEXT NOT NULL,      -- 业务场景：login_verify / order_payed / birthday_care 等
    channel     TEXT NOT NULL,      -- 渠道：sms/email/push/inbox
    lang        TEXT NOT NULL,      -- 语言：zh-CN / en-US
    version     INTEGER NOT NULL,   -- 模板版本号
    content     TEXT NOT NULL,      -- 模板内容
    is_active   INTEGER NOT NULL,   -- 1=启用，0=停用
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
```

#### 4.2.2 发送历史表 `notification_history`

消息生命周期：**waiting → processing → sent/delivered/failed/blocked**

```sql
CREATE TABLE IF NOT EXISTS notification_history (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    msg_id        TEXT NOT NULL UNIQUE,  -- 外部追踪用
    user_id       TEXT NOT NULL,
    scene         TEXT NOT NULL,
    channel       TEXT NOT NULL,
    content       TEXT NOT NULL,
    status        TEXT NOT NULL,        -- waiting/processing/sent/delivered/failed/blocked
    trace_id      TEXT NOT NULL,        -- 全链路 TraceId
    error         TEXT,
    retries       INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL,
    waiting_at    TEXT,
    processing_at TEXT,
    sent_at       TEXT,
    delivered_at  TEXT,
    failed_at     TEXT
);
```

#### 4.2.3 用户偏好表 `notification_user_pref`（简单版本）

```sql
CREATE TABLE IF NOT EXISTS notification_user_pref (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    TEXT NOT NULL,
    channel    TEXT NOT NULL,   -- sms/email/push/inbox
    enabled    INTEGER NOT NULL, -- 1=开启，0=关闭
    updated_at TEXT NOT NULL
);
```

---

## 5. 插件导出的 ABI

插件向 host 暴露的函数：

```rust
#[unsafe(no_mangle)]
pub extern "C" fn meta() -> PluginMeta;

#[unsafe(no_mangle)]
pub extern "C" fn run_with_ctx(ctx: *mut PluginContext);
```

- `meta()`：提供插件名称、版本、类型（kind=`"notification"`）；
- `run_with_ctx()`：
  - 将 host 的 `log_fn` / `emit_metric_fn` 封装为 `HostBridge`；
  - 初始化 DB（只建自己表）；
  - 起 tokio runtime & axum HTTP server；
  - 拉起异步 worker 处理队列。

> `plugin_api_info()` 是否需要由你当前的框架决定，如果已有该约定，你可以继续在 `lib.rs` 中实现并返回插件 HTTP 端口/前缀，供 api-server 做反向代理。

---

## 6. HTTP API 说明（通过 api-server 网关访问）

所有接口统一从 `api-server` 暴露，形式为：

```text
http://{API_SERVER}/plugin-api/notification-center/...
```

本地开发默认是：

```text
http://127.0.0.1:3001/plugin-api/notification-center/...
```

### 6.1 发送通知：`POST /send`

**请求：**

```http
POST /plugin-api/notification-center/send
Content-Type: application/json
```

```json
{
  "user_id": "1001",
  "scene": "order_payed",
  "channel_hint": "sms",        // 可选：业务方希望优先用的渠道
  "vars": {
    "order_id": "88888",
    "amount": "199"
  }
}
```

**行为：**

1. 为本次请求生成 `msg_id` + `trace_id`；
2. 写入 `notification_history`，状态为 `waiting`；
3. 将消息丢进插件内部的异步队列（`mpsc::channel`）；
4. 立即返回“已入队”的结果。

**响应示例：**

```json
{
  "msg_id": "ntf_6e5a1e6d-0a73-4b17-ae2e-8bd66d0c1234",
  "trace_id": "f8b7f0ce-3f63-40d9-b9f5-4a5f9d5aabcd",
  "status": "queued"           // 或 blocked/queue_error/db_error
}
```

> 真正的发送（风控、路由、模板、渠道调用）由后台 worker 异步完成，不会阻塞业务调用方。

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
  "status": "delivered",
  "trace_id": "f8b7f0ce-3f63-40d9-b9f5-4a5f9d5aabcd",
  "error": null,
  "retries": 0,
  "created_at": "2025-11-26T14:50:31.081Z"
}
```

---

### 6.3 模板列表：`GET /templates`（可选）

用于前端展示所有模板，方便后续做“模板管理”页面。当前实现是基础版，你可以根据需要扩展增删改接口。

---

### 6.4 模板预览：`POST /template_render_preview`（可选）

输入 `scene + vars`，使用当前激活模板渲染一份示例文案，适合管理端“实时预览”。

---

### 6.5 简单统计：`GET /stats`（占位）

当前实现为占位版本，返回一些 mock 统计字段，后续可以基于 `notification_history` 做真实聚合：

- 每日发送量 / 成功量
- 各渠道成功率
- 各场景的发送情况等

---

## 7. 内部处理流程（对应图中的 1~6 步）

worker 里完整的处理链路是：

1. **消息接入**  
   从队列中取出 `InternalMessage`，状态由 `waiting` → `processing`。

2. **预处理与风控**  
   调用 `risk_guard::check_risk`：
   - 黑名单拦截
   - 频率限制（user + scene / 时间窗）
   - 夜间免打扰（如 22:00–8:00）（可按场景放宽）  
   不通过则写 `status=blocked`，并上报 `notification_blocked` 指标。

3. **用户与模板匹配**  
   - 根据 `scene` 和渠道优先级选出主渠道；
   - 从 `notification_templates` 里取出对应模板；
   - 调用 `template::render` 用 `vars` 渲染最终内容。

4. **消息拼装**  
   渲染后的内容 + 基础变量（user_id、scene 等）构成最终发送文案。

5. **调度与分发（渠道优先级 + 降级）**  
   `channel::send_with_fallback` 完成：
   - 根据“强验证码 / 强提醒 / 营销 / 用户关怀 / 系统通知”等场景生成优先级列表；
   - 查询用户偏好（某些渠道是否关闭）；
   - 检查当前渠道可用性（现在用模拟逻辑，未来可接入真实健康检查）；
   - 依次调用 `sms::send` / `email::send` / `push::send` / `inbox::send`；
   - 上报每个渠道的发送时延 metric（`notification_channel_latency_ms_xxx`）。

6. **回执与追踪**  
   - 根据发送结果更新 `notification_history`：`status` + 时间字段 + error；
   - 上报结果指标：
     - `notification_success`
     - `notification_failed`
     - `notification_blocked`
     - `notification_process_duration_ms`  
   - 未来可以在 Dashboard 中按 `msg_id` / `trace_id` 做全链路追踪。

---

## 8. 与可观测性 / AI 的结合

得益于你已有的 Monitor AI Bot 能力：

- 所有 `HostBridge::metric()` 上报的指标都会进入统一 `metrics` 表；
- 所有日志通过 `log_fn` 进入统一日志流；
- AI 插件（如 `ai-analyzer`）可以直接消费通知中心的指标 / 日志：
  - 检测某个渠道成功率骤降
  - 检测某个场景发送量异常
  - 结合成本数据做“最优渠道策略”推荐

未来可以做到：

- 在 Dashboard 上有一个 **「通知中心」大屏**：
  - 渠道成功率趋势图
  - 不同供应商对比
  - 场景发送量 & 点击率
- AI 帮你自动调优模板 & 发送时段 & 渠道策略。

---

## 9. 下一步建议

在当前插件基础上，推荐的演进方向：

1. **前端通知中心管理页**
   - 模板列表 / 编辑 / 预览
   - 消息历史查询（按 user / scene / 时间）
   - 渠道统计图表（成功率、发送量、失败原因分布）

2. **真实渠道接入**
   - 在 `channel/sms.rs` / `email.rs` 中接入真实供应商 SDK / HTTP API
   - 在 DB 中增加渠道配置表（ak/sk、签名、模板 ID 等）

3. **更细粒度的风控策略**
   - per-channel 限流
   - 不同场景不同频控规则
   - 风控命中记录写入专门表，方便审计

4. **与 workflow / 探针联动**
   - API 工作流插件在出现失败时，自动调用通知中心发送告警
   - agent 探针上报节点异常 → 触发通知中心多渠道告警

---

**总结**：  
`notification-center` 插件已经实现了图里的大部分企业级通知中心能力，而且是作为一个**完全可插拔的插件**存在于你的 Monitor AI Bot 平台中，后续无论是扩充渠道、上云、多租户，还是接入 AI 做智能路由和策略优化，都有足够的扩展空间。
