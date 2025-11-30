# 🚀 Monitor AI Bot（监控 AI 机器人）

Monitor AI Bot 是一个基于 **Rust 插件系统（动态库） + 多进程架构 + SQLite 存储 + 多终端客户端** 的轻量级 **监控 & 流程引擎平台**。

你可以通过扩展 Rust 插件（`cdylib`）来自定义各种能力，例如：

- 系统监控：CPU / 内存 / 磁盘 / 自定义业务指标
- **API 端到端流程测试**（登录 → 多接口链路 → 断言结果）
- 分布式探针（Agent）上报多台服务器 / 设备状态
- AI 异常检测 & 智能告警
- 通知中心：短信 / 邮件 / Push / 站内信 / 钉钉 / 企业微信
- **工作流引擎插件（workflow-engine）**：执行 LogicFlow JSON 定义的工作流，并预留 Flowable / Zeebe 等外部 BPM 引擎对接能力
- **系统级定时器插件（timer-scheduler）**：统一的任务调度中心，其他插件按约定注册自己的 cron / interval 任务
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
           │   - 扫描插件目录，加载所有 cdylib                  │
           │   - 调用 run_with_ctx                              │
           │   - 通过 storage 写入 SQLite                       │
           │   - 调用 plugin_api_info 注册插件 API 映射         │
           └───────────────▲────────────────▲──────────────────┘
                           │ FFI            │ 插件 API 映射
                           │                │ (plugin_apis 表)
       ┌───────────────────┴────────────────┴────────────────────┐
       │                    Rust 插件系统                         │
       │   cpu-monitor / api-monitor / ai-analyzer /             │
       │   notification-center / workflow-engine /               │
       │   timer-scheduler / ...                                │
       │   - 动态库：*.dll / *.so / *.dylib                      │
       │   - 可选自带 HTTP Server（经 api-server 网关对外暴露）  │
       └─────────────────────────────────────────────────────────┘

        （分布式探针）        
      ┌────────────────────────────┐
      │   clients/agent-probe      │
      │   每台服务器/设备上的 Agent │
      │   定时采集 → POST /agent/...│
      └────────────────────────────┘
```

---

## 📂 项目结构

简化后的目录结构（按当前规划）：

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
├── workflow-core/                 # 工作流核心库（LogicFlow JSON 执行器 + Flowable/Zeebe 预留）
│   └── src/lib.rs
│
├── plugins/                       # 各种插件实现（cdylib）
│   ├── cpu-monitor/               # CPU 等系统指标监控
│   │   └── src/lib.rs
│   ├── api-monitor/               # 旧版基于 TOML 的 API 工作流插件（逐步迁移到 workflow-engine）
│   │   └── src/lib.rs
│   ├── workflow-engine/           # 新工作流引擎插件（LogicFlow JSON + 可切换执行引擎）
│   │   └── src/lib.rs
│   ├── ai-analyzer/               # AI 异常检测插件
│   │   └── src/lib.rs
│   ├── notification-center/       # 通知中心（短信/邮件/Push/钉钉/企微 等）
│   │   └── src/lib.rs
│   ├── timer-scheduler/           # 系统级定时器插件（其他插件统一由此调度）
│   │   └── src/lib.rs
│   └── ...                        # 其它插件（探针汇聚、业务插件等）
│
├── workflows/                     # 工作流配置文件
│   ├── api-monitor.toml           # 旧版 API 工作流配置
│   └── *.json                     # LogicFlow JSON 工作流定义（由 LogicFlow 设计器导出）
│
├── dashboard-frontend/            # 管理端 Web（React + Vite + ECharts）
│   └── src/App.tsx
│
├── clients/
│   ├── ui/                        # Web + Desktop 共用组件 & hooks
│   │   ├── components/
│   │   └── hooks/
│   ├── web-client/                # 业务/测试 Web 客户端（React + Vite）
│   ├── desktop-client/            # Tauri 桌面客户端（Rust + React）
│   ├── mobile-client/             # 手机 App（Expo + React Native）
│   └── agent-probe/               # 分布式探针 Agent（Rust bin）
│
├── plugins-bin/                   # 生产环境插件目录（拷贝编译好的 DLL/so）
├── database/                      # SQLite 数据文件目录（monitor_ai.db）
├── config.toml                    # Host 配置（插件扫描、调度等）
├── .env                           # 全局环境变量
├── Cargo.toml                     # Workspace 根配置
└── README.md
```

---

## ⚙️ 环境与配置

### 1. `config.toml`（Host 配置）

现在插件扫描逻辑已经简化为：**只按目录 + 扩展名，不再使用 name_pattern 过滤**。

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

# 默认调度周期（秒）（用于简单轮询型插件）
default_interval = 5
```

> 扫描规则：
>
> - dev 模式：加载 `target/debug` 目录下所有 `.dll` / `.so` / `.dylib` 文件
> - prod 模式：加载 `plugins-bin` 目录下所有动态库文件
> - 名字不做额外限制，是否是“插件”的语义由你把什么文件放进去决定。

### 2. `.env` 示例（根目录）

```env
# 插件运行模式（优先于 config.toml）
MONITOR_AI_PLUGIN_MODE=dev

# 数据库 URL（bot-host / api-server 共用）
MONITOR_AI_DB_URL=sqlite://database/monitor_ai.db

# 旧版工作流配置路径（给 api-monitor 用）
API_MONITOR_CONFIG=workflows/api-monitor.toml

# API / AI 引擎等
API_SERVER_BASE=http://127.0.0.1:3001
AI_ENGINE_BASE=http://127.0.0.1:8000

# API 流程测试账号（被 workflow-core / api-monitor 使用）
USER=test_user
PASS=secret
EXPECTED_USER_ID=123

# AI 后端选择：python | openai | deepseek（由 ai-analyzer 插件使用）
AI_BACKEND=python

# （可选）云端大模型的 Key
OPENAI_API_KEY=sk-xxx
DEEPSEEK_API_KEY=ds-xxx

# 工作流引擎类型：local_json | flowable | zeebe
# local_json：使用本地 Rust 代码执行 LogicFlow JSON
# flowable ：通过 HTTP 调用 Flowable 引擎（预留）
# zeebe    ：通过 gRPC/HTTP 调用 Zeebe（预留）
WORKFLOW_ENGINE=local_json
```

### 3. Agent 端 `.env` 示例（部署在被监控机器）

```env
AGENT_ID=server-001
MONITOR_AI_API_BASE=http://中心机IP:3001
AGENT_INTERVAL_SECS=5
```

---

## 🚀 快速启动（开发环境）

假设你在项目根目录 `monitor-ai-bot/`。

### 1. 启动 bot-host

```bash
cargo run -p bot-host
```

效果：

- 读取 `config.toml` 与 `.env`
- 根据模式（dev/prod）扫描插件目录下所有动态库
- 对每个插件：
  - 加载动态库（`libloading::Library`）
  - 调用 `meta()` 获取 `name/version/kind`
  - 若存在 `plugin_api_info()` 则写入 `plugin_apis` 表（插件 → base_url 映射）
  - 调用 `run_with_ctx()` 执行插件逻辑
- 为避免运行中起 HTTP Server 的插件被卸载，host 会在成功执行后对 `Library` 调用 `std::mem::forget(lib)`，使插件在进程生命周期内常驻内存。

### 2. 启动 api-server

```bash
cargo run -p api-server
```

默认监听 `http://127.0.0.1:3001`，提供：

- `GET /metrics`
- `GET /logs`
- `GET /alerts`（若已实现）
- `POST /agent/metrics`（Agent 上报）
- **`ANY /plugin-api/{plugin}/*rest` 插件 API 网关**

插件 API 网关会：

- 从 SQLite 表 `plugin_apis` 读取 `plugin -> base_url`
- 把请求转发到插件内部 HTTP 服务，例如 `http://127.0.0.1:5601/...`
- 前端 / Agent / 其它服务统一访问 `http://api-server:3001/plugin-api/{plugin}/xxx`，不关心插件端口和实现

### 3. 启动管理端 Dashboard（`dashboard-frontend`）

```bash
cd dashboard-frontend
npm install
npm run dev
# 浏览器访问 http://127.0.0.1:5173
```

### 4. 启动 Web 客户端（`clients/web-client`）

```bash
cd clients/web-client
npm install
npm run dev
# 浏览器访问 http://127.0.0.1:5173（Vite 默认端口）
```

### 5. 启动桌面客户端（Tauri，`clients/desktop-client`）

```bash
cd clients/desktop-client
npm install
npm run tauri dev
```

### 6. 启动移动端 App（`clients/mobile-client`）

```bash
cd clients/mobile-client
npm install
npm start
# 使用 Expo Go / 模拟器 打开
```

移动端需要配置中心 API 地址，例如：

```env
EXPO_PUBLIC_API_BASE=http://<中心机局域网IP>:3001
```

### 7. 启动 Agent 探针（`clients/agent-probe`）

```bash
cd clients/agent-probe
cargo run --release
```

---

## 🔌 插件系统（ABI + 插件 API 网关）

### 1. 基本 ABI：`meta` + `run_with_ctx`

在 `plugin-api` 中定义（简化）：

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

插件最少要导出：

```rust
#[unsafe(no_mangle)]
pub extern "C" fn meta() -> PluginMeta { ... }

#[unsafe(no_mangle)]
pub extern "C" fn run_with_ctx(ctx: *mut PluginContext) { ... }
```

### 2. 插件 API 信息：`PluginApiInfo + plugin_api_info`

为了让插件自带 HTTP API，通过网关暴露，定义：

```rust
#[repr(C)]
pub struct PluginApiInfo {
    /// 插件内部 HTTP server 监听端口，例如 5601
    pub port: u16,
    /// 统一前缀，例如 "/" 或 "/api"
    pub prefix: *const c_char,
}

pub type PluginApiInfoFunc = unsafe extern "C" fn() -> PluginApiInfo;
```

插件（如 `workflow-engine` / `notification-center`）可选实现：

```rust
#[unsafe(no_mangle)]
pub extern "C" fn plugin_api_info() -> PluginApiInfo {
    PluginApiInfo {
        port: 5601,
        prefix: c_string("/"),
    }
}
```

---

## 📊 核心数据模型

### Metric

```rust
pub struct Metric {
    pub time: DateTime<Utc>,
    pub plugin: String,
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
}
```

示例：

- `plugin = "cpu-monitor", name = "cpu_usage"`
- `plugin = "api-monitor", name = "api_flow_success"`
- `plugin = "agent-probe", name = "agent_cpu_usage"`
- `plugin = "notification-center", name = "ntf_send_total"`
- `plugin = "workflow-engine", name = "workflow_run_success"`

### LogEvent

```rust
pub struct LogEvent {
    pub time: DateTime<Utc>,
    pub level: LogLevel,
    pub plugin: Option<String>,
    pub message: String,
    pub fields: HashMap<String, String>,
}
```

### AlertEvent

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

### PluginApis 映射表

```sql
CREATE TABLE IF NOT EXISTS plugin_apis (
    plugin      TEXT PRIMARY KEY,
    base_url    TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
```

---

## 🧠 工作流引擎（workflow-core + workflow-engine 插件）

### 1. LogicFlow JSON 作为“标准 DSL”

- 前端使用 LogicFlow 画流程图（拖拖拽拽）
- 导出 LogicFlow JSON 文件，放到 `workflows/*.json`
- `workflow-engine` 插件在启动时加载这些 JSON，转换为内部的 `WorkflowDefinition`

### 2. workflow-core：抽象执行引擎

`workflow-core` 提供统一接口：

- `EngineKind`：`LocalJson | Flowable | Zeebe`
- `WorkflowDefinition::from_logicflow_json(...)`
- `WorkflowEngineRunner::run_once(def, input)`：执行一次工作流（同步/一次性运行）
- `WorkflowRunResult`：返回 success/duration/output 等信息

### 3. workflow-engine 插件：执行入口 + HTTP API

- 插件导出 `meta / plugin_api_info / run_with_ctx`
- 在 `run_with_ctx` 中：
  - 只启动一次 Axum HTTP Server（例如 `127.0.0.1:5601`）
  - 暴露接口：
    - `GET /health`
    - `GET /workflows`：列出所有 LogicFlow 工作流
    - `POST /workflows/:key/run`：执行一次指定工作流（输入为 JSON）
- 执行时：
  - 根据 `WORKFLOW_ENGINE` 选择 `LocalJson / Flowable / Zeebe`
  - 调用 `workflow-core` 的执行器完成实际业务
  - 把结果转成 `WorkflowRunResult`，可同步返回，也可写 Metric + Log

> 将来要接入 Flowable / Zeebe，只需要在 `workflow-core` 中实现对应 EngineKind 的执行逻辑即可，插件和整体架构不用改。

---

## ⏱ 系统级定时器插件（timer-scheduler）

### 1. 角色定位

- 作为 **平台级调度中心**，统一负责：
  - 周期任务（interval）
  - cron 表达式任务
  - 延迟任务
- 其他插件不再各自实现 `tokio::sleep` 的死循环调度，而是通过约定：
  - 向 `timer-scheduler` 注册任务
  - 或由 `timer-scheduler` 定期调用各插件暴露的 HTTP API

### 2. 使用方式示例（约定）

- `timer-scheduler` 插件自带 HTTP API（通过 `/plugin-api/timer-scheduler/...` 暴露）：
  - `POST /jobs`：注册一个定时任务，指定：
    - `target_plugin`: `workflow-engine`
    - `path`: `/workflows/login_and_get_profile/run`
    - `schedule`: `*/5 * * * *`（每 5 分钟）
    - `payload`: `{ ... }`
- 插件只需要提供一个“执行一次”的 API，比如 `workflow-engine` 的 `POST /workflows/:key/run`。
- 定时调度由 `timer-scheduler` 完成，bot-host 和其它插件逻辑保持简单。

> 这样一来，“平台调度”与“具体业务执行”彻底解耦，后续你还可以把 `timer-scheduler` 换成企业级调度组件（如 Quartz / PowerJob），而不会影响业务插件。

---

## 📢 通知中心插件（notification-center）

`notification-center` 插件实现统一通知能力：

- 多渠道：短信 / 邮件 / Push / 站内信 / 钉钉 / 企业微信
- 模板管理（scene + channel + lang + content）
- 简单风控（黑名单 / 频率控制）
- 消息落库（history 表，带 trace_id / msg_id）
- 自带 HTTP API（通过 `/plugin-api/notification-center/...` 暴露）：
  - `POST /send`
  - `GET /message/{msg_id}`
  - `GET /templates`
  - `POST /template_render_preview`
  - `GET /stats`

可以与 `workflow-engine` / `ai-analyzer` 等插件联动：

- 工作流执行失败 → 生成 Alert → 通知中心发钉钉/企微/短信/邮件。

---

## ✅ 功能一览 & 未来规划

| 能力                                   | 状态      |
|----------------------------------------|-----------|
| 动态加载插件（cdylib）                 | ✔ 已实现  |
| 插件上下文：日志 & 指标上报            | ✔ 已实现  |
| 插件 API 网关（/plugin-api/{plugin})   | ✔ 已实现  |
| SQLite 持久化 metrics / logs / alerts  | ✔ 已实现  |
| 分布式 Agent 探针                      | ✔ 已实现  |
| 旧版 API 工作流插件（api-monitor）     | ✔ 已实现  |
| LogicFlow 工作流引擎插件（workflow-engine） | ✔ 初版可用 |
| AI 分析插件（ai-analyzer）             | ✔ 初版可用 |
| 通知中心插件（notification-center）    | ✔ 初版可用 |
| 系统级定时器插件（timer-scheduler）    | ✔ 设计完成，逐步落地 |
| 多终端客户端（Web / Desktop / Mobile） | ✔ 骨架完成 |
| 更复杂规则引擎 / 多租户 / 鉴权         | 🔜 规划中 |
| Dashboard 图表 & 报表（ECharts）       | 🔜 持续增强 |

---

如果你接下来想做：

- 把 **workflow-engine 从 LogicFlow JSON → 执行 → 告警 → 通知中心** 串成一条完整链路；
- 或者把 **timer-scheduler + 任意插件** 做成“图形化 / AI 驱动的定时任务配置平台”；

我们可以直接按这个 README 当前架构，把那条链路的代码（后端 + 插件 + 前端）一次性补全。
