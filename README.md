# 🚀 Monitor AI Bot（监控 AI 机器人）

Monitor AI Bot 是一个基于 **Rust 插件系统（动态库） + 多进程架构 + SQLite 存储 + 多终端客户端** 的轻量级 **监控 & 流程引擎平台**。

你可以通过扩展 Rust 插件（`cdylib`）来自定义各种能力，例如：

- 系统监控：CPU / 内存 / 磁盘 / 自定义业务指标
- **API 端到端流程测试**（登录 → 多接口链路 → 断言结果）
- 分布式探针（Agent）上报多台服务器状态
- AI 异常检测 & 智能告警
- 插件自带 HTTP API（像一个个微服务），通过统一网关对外暴露

并通过 **Web Dashboard + Web 客户端 + 桌面客户端 + 手机客户端** 进行实时可视化与操作。

---
## ✨ 总体架构

```text
                         ┌──────────────────────────┐
                         │  dashboard-frontend      │
                         │   管理端 Web（React）    │
                         └────────────▲─────────────┘
                                      │
                         ┌────────────┴─────────────┐
                         │      clients/web-client  │
                         │   业务/测试 Web 客户端     │
                         └────────────▲─────────────┘
                                      │
                         ┌────────────┴─────────────┐
                         │  clients/desktop-client  │
                         │   Tauri 桌面客户端       │
                         └────────────▲─────────────┘
                                      │
                         ┌────────────┴─────────────┐
                         │  clients/mobile-client   │
                         │   手机 App (Expo RN)     │
                         └────────────▲─────────────┘
                                      │
                       HTTP / REST    │   /metrics /logs /alerts
                                      │   /plugin-api/{plugin}/...
               ┌──────────────────────┴──────────────────────┐
               │                  api-server                 │
               │   - 查 SQLite：metrics / logs / alerts      │
               │   - 插件 API 网关：/plugin-api/{plugin}    │
               └──────────────────────▲──────────────────────┘
                                      │ 写 DB
           ┌──────────────────────────┴─────────────────────────┐
           │                     bot-host                       │
           │   - 扫描 & 加载插件（cdylib）                      │
           │   - 调用 run_with_ctx                              │
           │   - 通过 storage 写入 SQLite                       │
           │   - 读取 plugin_api_info 注册插件 API 映射         │
           └───────────────▲────────────────▲──────────────────┘
                           │ FFI            │ 注册 API 映射
                           │                │ (plugin_apis 表)
       ┌───────────────────┴────────────────┴────────────────────┐
       │                    Rust 插件系统                         │
       │   cpu-monitor / api-monitor / ai-analyzer / ...         │
       │   - 动态库：*.dll / *.so / *.dylib                      │
       │   - 可选自带 HTTP Server（经 api-server 网关对外暴露）  │
       └─────────────────────────────────────────────────────────┘

        （分布式探针）
      ┌────────────────────────────┐
      │   clients/agent-probe      │
      │   每台服务器/设备上的 Agent │
      │   定时采集 → POST /agent/...│
      └────────────────────────────┘
````

---

## 📂 项目结构

当前仓库的核心结构（简化示意）：

```text
monitor-ai-bot/
│
├── bot-host/                      # 主进程：插件调度 + 写 SQLite + 注册插件 API
│   └── src/main.rs
│
├── api-server/                    # API 服务：REST + 插件 API 网关
│   └── src/main.rs
│
├── core-types/                    # Metric / Log / Alert 等共享结构体
│   └── src/lib.rs
│
├── storage/                       # SQLite 封装（Db + 各种 CRUD）
│   └── src/lib.rs
│
├── plugin-api/                    # 插件 ABI 定义（C ABI）
│   └── src/lib.rs
│
├── workflow-core/                 # API 工作流引擎核心（步骤、变量、断言）
│   └── src/lib.rs
│
├── plugins/                       # 各种插件实现（cdylib）
│   ├── cpu-monitor/               # CPU 等系统指标监控
│   │   └── src/lib.rs
│   ├── api-monitor/               # API 流程工作流插件（调用 workflow-core）
│   │   └── src/lib.rs
│   ├── ai-analyzer/               # AI 异常检测插件
│   │   └── src/lib.rs
│   └── ...                        # 其它插件（如 workflow/AI/业务插件）
│
├── workflows/                     # 工作流配置文件（TOML）
│   └── api-monitor.toml
│
├── dashboard-frontend/            # 管理端 Web（React + Vite + ECharts）
│   └── src/App.tsx
│
├── clients/
│   ├── ui/                        # Web + Desktop 共用的通用 React 组件 & hooks
│   │   ├── components/
│   │   │   ├── MetricOverview.tsx
│   │   │   └── AlertList.tsx
│   │   └── hooks/
│   │       ├── useMetrics.ts
│   │       └── useAlerts.ts
│   │
│   ├── web-client/                # 业务/测试 Web 客户端（React + Vite）
│   │   └── src/App.tsx
│   │
│   ├── desktop-client/            # Tauri 桌面客户端（Rust + React + Vite）
│   │   ├── src-tauri/
│   │   └── src/App.tsx
│   │
│   ├── mobile-client/             # 手机 App（Expo + React Native + TS）
│   │   └── App.tsx
│   │
│   └── agent-probe/               # 分布式探针 Agent（Rust bin）
│       └── src/main.rs
│
├── plugins-bin/                   # 生产环境插件目录（拷贝编译好的 DLL/so）
│
├── database/                      # SQLite 数据文件目录（monitor_ai.db）
│
├── config.toml                    # Host 配置（插件扫描、调度等）
├── .env                           # 全局环境变量
├── Cargo.toml                     # Workspace 根配置
└── README.md
```

---

## ⚙️ 环境与配置

### 1. config.toml（Host 配置）

用于控制 `bot-host` 如何扫描插件、调度间隔等，例如：

```toml
[plugin]
# 运行模式：
# - "dev"  : 开发阶段，直接扫 target/debug
# - "prod" : 发布阶段，扫 plugins-bin 目录
mode = "dev"

# 开发模式下插件动态库所在目录（相对项目根路径）
dev_dir = "target/debug"

# 生产模式下插件动态库所在目录
prod_dir = "plugins-bin"

# 要加载的插件文件名需要包含的关键字（防止乱加载）
name_pattern = "_monitor"

# 默认调度周期（秒）
default_interval = 5
```
```

### 2. .env 示例（根目录）

```env
# 插件运行模式（优先于 config.toml）
MONITOR_AI_PLUGIN_MODE=dev

# 数据库 URL（bot-host / api-server 共用）
MONITOR_AI_DB_URL=sqlite://database/monitor_ai.db

# 工作流配置路径
API_MONITOR_CONFIG=workflows/api-monitor.toml

# API / AI 引擎等
API_SERVER_BASE=http://127.0.0.1:3001
AI_ENGINE_BASE=http://127.0.0.1:8000

# API 流程测试账号
USER=test_user
PASS=secret
EXPECTED_USER_ID=123

# AI 后端选择：python | openai | deepseek
AI_BACKEND=python

# （可选 OpenAI / DeepSeek 消费）
OPENAI_API_KEY=sk-xxx
DEEPSEEK_API_KEY=ds-xxx
```

### 3. Agent 端 .env 示例（clients/agent-probe 部署机器）

```env
AGENT_ID=server-001
MONITOR_AI_API_BASE=http://中心机IP:3001
AGENT_INTERVAL_SECS=5
```

## 🚀 快速启动（开发环境）

假设你在项目根目录 `monitor-ai-bot/`。

### 1. 启动 bot-host

```bash
cargo run -p bot-host
```


效果：

* 自动扫描插件（`target/debug/*_monitor.dll` / `.so`）
* 周期性调用插件的 `run_with_ctx`
* 通过 `storage::Db` 写 `metrics` / `logs` 表到 SQLite
* 发现有实现 `plugin_api_info` 的插件时，自动把其 API 映射写入 `plugin_apis` 表

---

### 2. 启动 api-server

```bash
cargo run -p api-server
```

默认监听 `http://127.0.0.1:3001`，提供：

* `GET /metrics`
* `GET /logs`
* `GET /alerts`（若已实现）
* `POST /agent/metrics`（Agent 上报）
* **`ANY /plugin-api/{plugin}/*rest` 插件 API 网关**

插件 API 网关会：

* 从 DB `plugin_apis` 表读取 `plugin -> base_url`
* 将请求转发到插件本地 HTTP Server（例如 `http://127.0.0.1:5501/...`）
* 前端 / Agent 只用固定路径：`/plugin-api/{plugin}/xxx`，无感知插件实际端口

---

### 3. 启动管理端 Dashboard（dashboard-frontend）

```bash
cd dashboard-frontend
npm install
npm run dev
# 浏览器访问 http://127.0.0.1:5173
```

用于：

* 核心指标面板
* 更丰富图表（ECharts）
* 管理插件 / 工作流 / 告警（持续增强中）

---

### 4. 启动 Web 客户端（clients/web-client）

```bash
cd clients/web-client
npm install
npm run dev
# 浏览器访问 http://127.0.0.1:5173（或 Vite 默认端口）
```

特点：

* 面向业务 / 测试同事的简化版视图
* 使用 `clients/ui` 提供的通用组件：

  * `MetricOverview`：关键指标卡片
  * `AlertList`：告警列表
* 默认调用：`GET /metrics` / `GET /alerts`

---

### 5. 启动桌面客户端（Tauri，clients/desktop-client）

```bash
cd clients/desktop-client
npm install
npm run tauri dev
```

特点：

* 使用和 Web 客户端基本一致的 UI（共用 `clients/ui`）
* 可以额外集成：

  * 系统托盘
  * 桌面通知
  * 本地配置缓存
  * 后续本地插件/Agent 管理等

---

### 6. 启动移动端 App（clients/mobile-client）

```bash
cd clients/mobile-client
npm install
npm start
# 使用 Expo Go 或 模拟器 打开
```

功能：

* 查看关键 CPU/API 流程指标
* 查看近期告警
* 下拉刷新（简单移动端场景）

注意：

* API 地址应使用中心机在局域网中的 IP，比如：`EXPO_PUBLIC_API_BASE=http://192.168.1.10:3001`

---

### 7. 启动 Agent 探针（clients/agent-probe）

在某台被监控服务器上：

```bash
cd clients/agent-probe
cargo run --release
```

Agent 会：

* 使用 `sysinfo` 等库采集本机 CPU / 内存 / host 名称等
* 定时 POST 到中心 `api-server` 的 `/agent/metrics`
* 在 DB 中写入 metrics：

  * `plugin = "agent-probe"`
  * `name = "agent_cpu_usage"` / `"agent_memory_used"` / ...
  * `labels` 中带 `agent_id` / `host`

---

## 🔌 插件系统（Plugin ABI + 插件 API 网关）

### 1. 基本 ABI：meta + run / run_with_ctx

在 `plugin-api` 中定义的核心结构体与函数类型（简化示意）：

```rust
#[repr(C)]
pub struct PluginMeta {
    pub name: *const c_char,
    pub version: *const c_char,
    pub kind: *const c_char,
}

#[repr(C)]
pub struct PluginContext {
    pub host_version: u32,
    pub log_fn: extern "C" fn(LogLevel, *const c_char),
    pub emit_metric_fn: extern "C" fn(MetricSample),
}

#[repr(C)]
pub struct MetricSample {
    pub name: *const c_char,
    pub value: f64,
    pub timestamp_ms: i64,
}

pub type PluginMetaFunc = unsafe extern "C" fn() -> PluginMeta;
pub type PluginRunFunc = unsafe extern "C" fn();
pub type PluginRunWithContextFunc = unsafe extern "C" fn(*mut PluginContext);
```

插件必须至少实现：

```rust
#[unsafe(no_mangle)]
pub extern "C" fn meta() -> PluginMeta { ... }

#[unsafe(no_mangle)]
pub extern "C" fn run_with_ctx(ctx: *mut PluginContext) { ... }
```

### 2. 插件 API 元信息：PluginApiInfo + plugin_api_info

为了让插件 **像微服务一样拥有自己的 HTTP API**，在 `plugin-api` 增加：

```rust
#[repr(C)]
pub struct PluginApiInfo {
    /// 插件内部 HTTP server 监听端口，例如 5501
    pub port: u16,
    /// 统一前缀，例如 "/" 或 "/api"
    pub prefix: *const c_char,
}

/// 插件可以（可选）导出：
pub type PluginApiInfoFunc = unsafe extern "C" fn() -> PluginApiInfo;
```

插件示例（如 `api-monitor`）：

```rust
use plugin_api::{PluginMeta, PluginContext, PluginApiInfo};
use std::os::raw::c_char;
use std::ffi::CString;

const NAME: &str = "api-monitor";
const VERSION: &str = "0.2.0";
const KIND: &str = "workflow";
const API_PORT: u16 = 5501;
const API_PREFIX: &str = "/";

fn c_string(s: &str) -> *const c_char {
    CString::new(s).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn meta() -> PluginMeta {
    PluginMeta {
        name: c_string(NAME),
        version: c_string(VERSION),
        kind: c_string(KIND),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_api_info() -> PluginApiInfo {
    PluginApiInfo {
        port: API_PORT,
        prefix: c_string(API_PREFIX),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn run_with_ctx(ctx: *mut PluginContext) {
    // 启动插件内 HTTP Server（只启动一次）
    // 例如 listen 127.0.0.1:5501，提供 /health /status
    // 然后执行自身监控/工作流逻辑，并通过 ctx.log_fn / emit_metric_fn 上报
}
```

### 3. host：自动发现并写入 plugin_apis

`bot-host` 在加载每个插件时：

1. 调用 `meta()` 获取 `plugin_name`
2. 尝试调用 `plugin_api_info()`：

   * 若存在：得到 `port` + `prefix` → 组装 `base_url = "http://127.0.0.1:{port}{prefix}"`
   * 调用 `Db::upsert_plugin_api(plugin_name, base_url)`
3. `api-server` 启动时通过 `Db::get_all_plugin_apis()` 加载所有映射，并放入内存 HashMap

### 4. api-server：统一插件 API 网关

新增固定路由（只改一次，后面再多插件都不用动）：

```text
ANY /plugin-api/:plugin/*rest
```

行为：

1. 从内存映射中拿到插件的 `base_url`（比如 `http://127.0.0.1:5501/`）
2. 拼接：`target = base_url + /{rest}`
3. 使用 `reqwest` 把原始 HTTP 请求（方法、头、body）转发给插件
4. 将插件返回的响应（状态码、头、body）原样返回给客户端

因此：

* 插件可以自己用 axum/hyper 实现任意复杂 API
* **前端、Agent、其它服务只用访问：**
  `http://api-server:3001/plugin-api/{plugin}/xxx`
* 无需感知插件绑定的端口、部署方式等

---

## 📊 数据模型（核心表）

### Metric（指标）

`core-types::Metric` 示例：

```rust
pub struct Metric {
    pub time: DateTime<Utc>,
    pub plugin: String,
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
}
```

典型记录：

* `plugin = "cpu-monitor", name = "cpu_usage", value = 37.5, labels = { "host": "server-001" }`
* `plugin = "api-monitor", name = "api_flow_success", value = 1.0, labels = { "workflow": "login_and_get_profile" }`
* `plugin = "agent-probe", name = "agent_cpu_usage", labels = { "agent_id": "server-002", "host": "dev-node-02" }`

### LogEvent（日志）

```rust
pub struct LogEvent {
    pub time: DateTime<Utc>,
    pub level: LogLevel,
    pub plugin: Option<String>,
    pub message: String,
    pub fields: HashMap<String, String>,
}
```

由插件通过 `log_fn` 上报，由 host 统一写入 SQLite。

### AlertEvent（告警）

```rust
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

pub struct AlertEvent {
    pub time: DateTime<Utc>,
    pub plugin: String,
    pub metric_name: String,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
}
```

由上层逻辑（插件或外部服务）通过 HTTP `POST /alerts` 写入；
前端通过 `GET /alerts` 展示。

### PluginApis（插件 API 映射）

SQLite 表 `plugin_apis`：

```sql
CREATE TABLE IF NOT EXISTS plugin_apis (
    plugin      TEXT PRIMARY KEY,
    base_url    TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
```

内容示例：

| plugin      | base_url                                               |
| ----------- | ------------------------------------------------------ |
| api-monitor | [http://127.0.0.1:5501/](http://127.0.0.1:5501/)       |
| ai-analyzer | [http://127.0.0.1:5502/api](http://127.0.0.1:5502/api) |

---

## 🧠 工作流引擎（workflow-core + api-monitor）

通过 `workflow-core` 定义结构化的 API 工作流：

* `Workflow`：一条流程（如“登录并获取用户信息”）
* `Step`：流程中的一个步骤（如“POST /login”）
* 支持：

  * 多步骤顺序执行
  * 从响应 JSON 中提取变量（`extract`）
  * 使用 `{{var_name}}` 注入到后续步骤的 URL / Header / Body
  * 基础断言（HTTP 状态码、JSON 值等）

`workflows/api-monitor.toml` 示例（简化）：

```toml
[[workflows]]
name = "login_and_get_profile"
enabled = true
base_url = "https://api.example.com"

  [[workflows.steps]]
  id = "login"
  method = "POST"
  path = "/auth/login"
  body = '{"username": "{{USER}}", "password": "{{PASS}}"}'

  [workflows.steps.headers]
  Content-Type = "application/json"

  [workflows.steps.extract]
  token = "data.token"

  [[workflows.steps.asserts]]
  status = 200

  [[workflows.steps]]
  id = "get_profile"
  method = "GET"
  path = "/user/profile"

  [workflows.steps.headers]
  Authorization = "Bearer {{token}}"

  [[workflows.steps.asserts]]
  status = 200
  json_path = "data.user.id"
  equals = "{{EXPECTED_USER_ID}}"
```

`api-monitor` 插件：

* 周期性读取这个 workflow 配置
* 执行 HTTP 请求链路，填充变量，执行断言
* 把结果作为 Metric 上报，例如：

  * `api_flow_success`（0/1）
  * `api_flow_duration_ms`
* 出问题时写 Log / 告警，为后续 AI 分析打基础

---

## 🧠 AI 分析插件（ai-analyzer）

`ai-analyzer` 插件的职责通常是：

1. 聚合某些 Metric，比如：

   * `cpu-monitor` 的 CPU 序列
   * `api-monitor` 的流程成功率 / 耗时等
2. 通过 HTTP 调用外部 AI 引擎：

   * 本地 Python `ai-engine`（FastAPI）
   * 或 OpenAI / DeepSeek 等模型
3. 输出结果：

   * 新的 Metric（如 `api_anomaly_score`）
   * 触发 Alert（通过 HTTP `POST /alerts`）

这样你可以把 AI 能力完全当作 **插件的一种实现方式**，而不需要改 host / api-server。

---

## ✅ 功能一览 & 未来规划

| 能力                                 | 状态      |
| ---------------------------------- | ------- |
| 动态加载插件（cdylib）                     | ✔ 已实现   |
| 插件上下文：日志 & 指标上报                    | ✔ 已实现   |
| 插件 API 网关（/plugin-api/{plugin})    | ✔ 已实现   |
| SQLite 持久化 metrics / logs / alerts | ✔ 已实现   |
| 多端客户端（Web / Desktop / Mobile）      | ✔ 已有骨架  |
| 分布式 Agent 探针（agent-probe）          | ✔ 已有示例  |
| API 工作流引擎（workflow-core）           | ✔ 已实现   |
| API 流程监控插件（api-monitor）            | ✔ 已实现   |
| AI 分析插件（ai-analyzer）               | ✔ 初版可用  |
| 仪表盘（ECharts 折线图 / 饼图）              | 🔜 持续增强 |
| 更复杂规则引擎 / 多租户 / 鉴权                 | 🔜 计划中  |

---

## 🧩 如何扩展？

你可以：

* **增加新的插件**

  * 监控 Redis / MySQL / Kafka / 业务指标
  * 插件内直接起 HTTP Server，通过 `plugin_api_info` 暴露 API（例如 `/health` / `/config`）
* **增加新的 Agent 采集项**

  * 在 `agent-probe` 中拓展更多系统信息、日志、业务数据
* **扩展前端**

  * 在 Dashboard / Web Client / Desktop/Mobile 中增加：

    * 多机器视图
    * 拖拽式工作流编排
    * 告警处理 / 确认 / 屏蔽
* **增强 AI 能力**

  * 对接更多模型提供商
  * 按业务场景（支付、订单、风控）定制异常规则

---

如果你在某个具体部分（例如：
**“给 api-monitor 加一个插件 API /status，并在前端展示”**，
或 **“Agent 新增磁盘使用率监控并画图”**）需要完整代码，我可以继续帮你从后端到前端一条链路写完。 😊

```

::contentReference[oaicite:0]{index=0}
```
