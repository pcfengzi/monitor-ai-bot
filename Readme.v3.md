# 🚀 Monitor AI Bot（监控 AI 机器人）

Monitor AI Bot 是一个基于 **Rust 插件系统（动态库） + 多进程架构 + SQLite 存储 + Web Dashboard** 的轻量级监控 & 流程引擎框架。

你可以通过扩展 Rust 插件动态库（DLL/so/dylib）来自定义各种能力，例如：

- 系统监控：CPU / 内存 / 磁盘
- **API 端到端流程测试**（登录 → 调用多个接口 → 校验返回）
- 业务数据健康检查
- AI 异常检测 & 告警
- 其它自定义采样 / 自动化任务

并通过 Web Dashboard 进行实时可视化。

---

## ✨ 架构总览

当前系统核心模块：

```text
                 ┌────────────────────────┐
                 │      dashboard         │
                 │ (React 前端可视化页面) │
                 └──────────▲────────────┘
                            │ HTTP API
                 ┌──────────┴────────────┐
                 │      api-server       │
                 │ 从 SQLite 读取 Metric │
                 │ / Log / Alert 并提供  │
                 │   RESTful 接口        │
                 └──────────▲────────────┘
                          写入 DB
                 ┌──────────┴────────────┐
                 │       bot-host        │
                 │   加载插件 → 执行     │
                 │   上报 Metric / Log   │
                 │   写入 SQLite         │
                 └──────────▲────────────┘
                          FFI 调用
       ┌───────────────────┴────────────────────┐
       │              Rust 插件系统             │
       │   cpu-monitor / api-monitor / ai-analyzer  │
       │        *.dll / *.so / *.dylib           │
       └─────────────────────────────────────────┘

                 （可选）
                 ┌────────────────────────┐
                 │      ai-engine         │
                 │ 独立 Python AI 服务    │
                 │ (FastAPI + 模型推理)   │
                 └────────────────────────┘
```

### 角色说明

* **bot-host**

  * 扫描插件目录，动态加载插件（`cdylib`）
  * 调用插件入口（`run_with_ctx`）
  * 接收插件上报的 Metric / Log，通过 `storage` 写入 SQLite
  * 定时调度，实现“周期性监控 & 流程执行”

* **api-server**

  * 作为独立进程，连接同一个 SQLite 实例
  * 提供统一 HTTP API：

    * `GET /logs`
    * `GET /metrics`
    * （可扩展）`GET/POST /alerts`
  * 未来可以扩展为权限控制、多租户等

* **dashboard-frontend**

  * React + TypeScript + Vite
  * 调用 api-server 提供的接口
  * 展示日志列表、监控指标、告警信息，后续可升级为图表

* **插件（plugins/*）**

  * `cpu-monitor`：系统指标采集插件示例
  * `api-monitor`：**API 工作流监控插件**（支持多步骤流程 + 变量传递）
  * `ai-analyzer`：AI 分析插件（从 `/metrics` 拉数据，调用 AI 接口，必要时 POST `/alerts`）

* **workflow-core**

  * 抽象“工作流”和“步骤”的核心结构
  * 支持：

    * 多步骤顺序执行
    * 从 JSON 响应中提取变量
    * 使用 `{{var}}` 在后续步骤中注入参数
    * 基础断言（HTTP 状态码 / JSON 字段值）

---

## 📂 项目结构

与你当前仓库保持一致的结构示例：

```text
monitor-ai-bot/
│
├── api-server/                  # 单独进程：提供 HTTP API
│   └── src/main.rs
│
├── bot-host/                    # 主程序：插件调度 + 写 SQLite
│   └── src/main.rs
│
├── core-types/                  # Metric / Log / Alert 等共享结构体
│   └── src/lib.rs
│
├── dashboard-frontend/          # 前端仪表盘（React + Vite + TS）
│   └── src/App.tsx
│
├── plugin-api/                  # 插件 ABI 接口定义（C ABI）
│   └── src/lib.rs
│
├── storage/                     # SQLite 封装（Host & API 共用）
│   └── src/lib.rs
│
├── workflow-core/               # 工作流描述与变量系统（Workflow + Step）
│   └── src/lib.rs
│
├── plugins/                     # 插件源码
│   ├── cpu-monitor/
│   │   └── src/lib.rs
│   ├── api-monitor/             # API 流程工作流插件
│   │   └── src/lib.rs
│   └── ai-analyzer/             # AI 分析插件
│       └── src/lib.rs
│
├── workflows/                   # 工作流配置（API 流程定义）
│   └── api-monitor.toml
│
├── plugins-bin/                 # 生产环境插件（DLL/so/dylib，可选）
│
├── ai-engine/                   # （可选）Python AI 引擎服务
│   └── main.py
│
├── .env                         # 环境变量（dev/prod、路径、AI 等）
├── config.toml                  # Host 配置（插件目录等）
├── Cargo.toml                   # Workspace
└── README.md
```

---

## ⚙️ 快速开始

### 1. 启动 bot-host（执行插件 → 写 SQLite）

```bash
cargo run -p bot-host
```

看到类似日志：

```text
=== 监控AI机器人 bot-host 启动 ===
已连接 SQLite 数据库: sqlite:monitor_ai.db
运行模式: dev, 插件目录: target/debug, ...
发现 3 个插件:
  - target/debug/cpu_monitor.dll
  - target/debug/api_monitor.dll
  - target/debug/ai_analyzer.dll
...
```

此时：

* 插件会被定期调用
* 日志 / 指标会写入 `monitor_ai.db`（SQLite 文件）

---

### 2. 启动 api-server（读取 SQLite → 提供 HTTP API）

```bash
cargo run -p api-server
```

默认监听：

```text
http://127.0.0.1:3001
```

API 示例：

* `GET http://127.0.0.1:3001/logs`
* `GET http://127.0.0.1:3001/metrics`
* （如果实现了 Alert 接口）`GET http://127.0.0.1:3001/alerts`

---

### 3. 启动前端 Dashboard

```bash
cd dashboard-frontend
npm install
npm run dev
```

浏览器访问：

```text
http://127.0.0.1:5173
```

就能看到从 `api-server` 获取的实时监控数据。

---

### 4. （可选）启动 AI 引擎服务（Python）

如果你使用 Python 版 `ai-engine`：

```bash
cd ai-engine
pip install -e .
uvicorn main:app --host 127.0.0.1 --port 8000 --reload
```

`ai-analyzer` 插件会通过 HTTP 调用该服务，实现 AI 异常检测。

---

## 🔧 核心配置说明

### 1. Workspace 根 `config.toml`

用于控制 `bot-host` 的插件目录和调度行为，例如：

```toml
[plugin]
mode = "dev"             # dev | prod
dev_dir = "target/debug" # 开发环境插件所在目录
prod_dir = "plugins-bin" # 生产环境插件所在目录
default_interval = 5     # 默认调度周期（秒）
auto_load = true         # 是否自动扫描并加载插件
```

### 2. `.env` 示例

```env
# 插件扫描模式
MONITOR_AI_PLUGIN_MODE=dev

# API 工作流配置路径
API_MONITOR_CONFIG=workflows/api-monitor.toml

# AI 引擎 & API-Server 基础地址
API_SERVER_BASE=http://127.0.0.1:3001
AI_ENGINE_BASE=http://127.0.0.1:8000

# API 流程测试的账号等
USER=test_user
PASS=secret
EXPECTED_USER_ID=123

# AI 后端选择：python | openai | deepseek (由 ai-analyzer 插件使用)
AI_BACKEND=python
```

---

## 🧩 插件开发指南

### 1. 创建一个新的插件

```bash
cargo new plugins/my-plugin --lib
```

`Cargo.toml` 配置：

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
plugin-api = { path = "../../plugin-api" }
```

### 2. 插件 ABI 要求

必须导出：

* `meta()`：返回插件元信息（名称 / 版本 / 类型）
* `run()`（旧版，兼容调试）
* 或 **推荐** `run_with_ctx(ctx: *mut PluginContext)`：可使用上下文进行日志与指标上报

```rust
use plugin_api::{PluginMeta, PluginContext, LogLevel, MetricSample};
use std::ffi::CString;
use std::os::raw::c_char;

static NAME: &[u8] = b"my-plugin\0";
static VERSION: &[u8] = b"0.1.0\0";
static KIND: &[u8] = b"custom\0";

#[unsafe(no_mangle)]
pub extern "C" fn meta() -> PluginMeta {
    PluginMeta {
        name: NAME.as_ptr() as *const c_char,
        version: VERSION.as_ptr() as *const c_char,
        kind: KIND.as_ptr() as *const c_char,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn run() {
    println!("[my-plugin] run() executed");
}

#[unsafe(no_mangle)]
pub extern "C" fn run_with_ctx(ctx: *mut PluginContext) {
    if ctx.is_null() {
        return;
    }
    let ctx = unsafe { &*ctx };

    // 1. 上报日志
    let msg = CString::new("Hello from my-plugin").unwrap();
    (ctx.log_fn)(LogLevel::Info, msg.as_ptr());

    // 2. 上报指标
    let metric_name = CString::new("my_plugin_heartbeat").unwrap();
    let sample = MetricSample {
        name: metric_name.as_ptr(),
        value: 1.0,
        timestamp_ms: current_timestamp_ms(),
    };
    unsafe {
        (ctx.emit_metric_fn)(sample);
    }
}

fn current_timestamp_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    now.as_millis() as i64
}
```

---

## 📊 数据模型：Metric / Log / Alert

### Metric（指标）

在 `core-types` 中定义：

```rust
pub struct Metric {
    pub time: DateTime<Utc>,
    pub plugin: String,
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
}
```

* 插件通过 `emit_metric_fn` 上报
* bot-host 写入 SQLite `metrics` 表
* api-server 通过 `/metrics` 提供查询

### Log（日志）

```rust
pub struct LogEvent {
    pub time: DateTime<Utc>,
    pub level: LogLevel,
    pub plugin: Option<String>,
    pub message: String,
    pub fields: HashMap<String, String>,
}
```

* 插件通过 `log_fn` 上报
* 统一由 host 使用 `tracing` 输出，并写入 SQLite `logs` 表
* api-server 提供 `/logs` 查询

### Alert（可选：告警）

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

* 建议由上层（例如 `ai-analyzer` 插件或外部工具）通过 HTTP `POST /alerts` 写入
* api-server 使用 `storage::insert_alert` 写 SQLite `alerts` 表
* dashboard 通过 `/alerts` 展示告警列表

> **注意：** Alert 写入不需要修改 bot-host / plugin-api，而是通过 HTTP API 完成，避免内核频繁变更。

---

## 🧠 工作流：API 端到端流程测试（api-monitor + workflow-core）

`api-monitor` 插件 + `workflow-core` 提供了强大的 **工作流能力**，可以：

* 定义多步 API 调用流程（登录 → 获取 token → 调用业务接口）
* 从响应 JSON 中提取变量（如 token、user_id）
* 使用 `{{var}}` 作为下一步的请求参数或 Header
* 对每一步做断言（状态码 / JSON 字段值）
* 将整条流程的结果作为 Metric 上报，形成 **定时自动化集成测试**

### 工作流配置示例：`workflows/api-monitor.toml`

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
  token = "data.token"          # 从 JSON 响应 data.token 提取 token

  [[workflows.steps.asserts]]
  status = 200

  [[workflows.steps]]
  id = "get_profile"
  method = "GET"
  path = "/user/profile"
  body = ""

  [workflows.steps.headers]
  Authorization = "Bearer {{token}}"

  [[workflows.steps.asserts]]
  status = 200
  json_path = "data.user.id"
  equals = "{{EXPECTED_USER_ID}}"
```

### 执行效果

每次 bot-host 调用 `api-monitor` 插件时：

* 按顺序执行 steps：

  * `login` → `get_profile`
* 在 `login` 响应中提取 `token` 放入变量表
* 在 `get_profile` 请求头中注入 `Authorization: Bearer {{token}}`
* 按 asserts 校验每一步：

  * 状态码是否为 200
  * 指定 JSON 字段是否与期望值一致
* 最后上报全局 Metric：

  * `api_flow_success`：1.0 表示该工作流整条流程成功，0.0 表示失败
  * `api_flow_duration_ms`：整条流程耗时（毫秒）

你可以在 Dashboard 或 AI 插件中基于这些指标实现：

* 接口稳定性监控
* SLA 统计
* 接口异常自动告警

---

## 🧠 AI 分析：ai-analyzer 插件

`ai-analyzer` 是一个标准插件，它：

1. 从 `api-server` 的 `/metrics` 拉取最新的 Metric（例如 `cpu-monitor` 上报的 `cpu_usage`）
2. 根据配置选择不同 AI 后端：

   * 本地 Python `ai-engine`（FastAPI）
   * OpenAI / DeepSeek 等云端模型接口
3. 对时间序列进行异常检测
4. 上报：

   * `anomaly_score_xxx` Metric（数值分数）
   * （可选）通过 `POST /alerts` 写入告警

AI 后端通过环境变量选择：

```env
AI_BACKEND=python         # python | openai | deepseek
AI_ENGINE_BASE=http://127.0.0.1:8000
OPENAI_API_KEY=sk-xxxx
DEEPSEEK_API_KEY=ds-xxxx
```

> **重要：** AI 插件只是普通插件的一种，不需要修改 bot-host，只使用：
>
> * HTTP 调用（到 api-server / ai-engine）
> * Metric / Log 上报
> * HTTP `POST /alerts` 写 Alert

---

## 🔥 已具备的能力 & 下一步

| 能力                                 | 状态           |
| ---------------------------------- | ------------ |
| 动态插件加载（DLL/so/dylib）               | ✔ 已实现        |
| 插件上下文（日志 + 指标上报）                   | ✔ 已实现        |
| 插件热插拔 / 多插件调度                      | ✔ 已实现        |
| SQLite 持久化 Metric / Log / Alert    | ✔ 已实现        |
| api-server 独立进程（RESTful API）       | ✔ 已实现        |
| React Dashboard 展示                 | ✔ 已实现        |
| 工作流引擎（workflow-core + api-monitor） | ✔ 已实现        |
| AI 分析插件（ai-analyzer）               | ✔ 初版可用       |
| 告警 API（/alerts） & Alert 表结构        | ✔ 设计完成，可随时实现 |
| 图表展示（折线图 / ECharts 等）              | 🔜 待增强       |
| 鉴权、多租户、复杂规则引擎                      | 🔜 可逐步演进     |

你已经有了一个非常灵活的 **“监控 + 工作流 + AI 分析” 平台框架**，后续可以按需在：

* 插件层：增加更多 monitor / workflow / ai 插件
* API 层：扩展更多查询 / 聚合 / 告警接口
* UI 层：增强可视化（折线图 / 仪表盘 / 拖拽式工作流编排）
* AI 层：接入更复杂模型（自有 / 云端）

---

如果你想，我可以下一步帮你：

* 在 dashboard 上加一个 **“API 流程健康度” 折线图**（基于 `api_flow_success` / `api_flow_duration_ms`）
* 或者帮你把 **`/alerts` 接口 + 前端告警列表页** 全部补齐。

```
::contentReference[oaicite:0]{index=0}
```
