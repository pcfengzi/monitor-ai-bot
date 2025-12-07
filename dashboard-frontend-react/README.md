好，直接给你一份 **包含这 3 个新增章节的最终版 README**，你可以整份覆盖 `dashboard-frontend-react/README.md`。

---

````markdown
# Monitor AI Bot Dashboard（dashboard-frontend-react）

Monitor AI Bot Dashboard 是整个 **Monitor AI Bot 平台的管理控制台前端**，用于：

- 查看核心监控指标（CPU、API 流程、Agent 状态等）
- 管理工作流、插件、分布式 Agent
- 作为前端插件（如工作流设计器 workflow-designer）的集成入口

技术栈：

- **React + TypeScript**
- **Vite** 作为开发与构建工具
- **React Router v6** 路由
- 自定义 **前端插件注册机制**（前端插件自动挂载）
- 轻量 CSS（`index.css` + `App.css`），后续可平滑迁移到 Tailwind / CSS Modules

> 🔧 React / React DOM 的具体版本由仓库根目录（Workspace）统一管理，这个子项目本身不直接固定版本。

---

## 📁 目录结构概览

```text
dashboard-frontend-react/
├── public/                       # 静态资源
├── src/
│   ├── assets/                   # 静态图片、图标等（预留）
│   ├── components/
│   │   └── PageCard.tsx          # 通用页面卡片容器
│   ├── layout/
│   │   ├── DashboardLayout.tsx   # 顶栏 + 侧边栏 + 主体布局
│   │   ├── Sidebar.tsx           # 左侧导航
│   │   └── Topbar.tsx            # 顶部导航
│   ├── pages/                    # 路由页面
│   │   ├── DashboardHome.tsx     # 总览页（Dashboard 首页）
│   │   ├── MetricsPage.tsx       # 指标页
│   │   ├── LogsPage.tsx          # 日志页
│   │   ├── AlertsPage.tsx        # 告警中心
│   │   ├── AgentsPage.tsx        # Agent 管理
│   │   ├── PluginsPage.tsx       # 插件管理
│   │   └── SettingsPage.tsx      # 系统设置
│   ├── plugins/
│   │   ├── loader.ts             # 自动加载各插件 entry（import.meta.glob）
│   │   ├── plugin-registry.ts    # 前端插件注册表（registerPlugin / getPlugins）
│   │   └── types.ts              # 前端插件类型定义 FrontendPlugin
│   ├── routes/                   # （预留，将来可拆 Router 配置）
│   ├── store/                    # （预留，全局状态）
│   ├── App.tsx                   # 顶层路由结构（嵌套 DashboardLayout）
│   ├── main.tsx                  # React 入口，挂载 BrowserRouter + App
│   ├── App.css                   # 布局相关样式（Topbar/Sidebar/Content）
│   └── index.css                 # 全局 reset + 基础样式
│
├── package.json
├── tsconfig.json
├── vite.config.ts
└── README.md                     # 本文档
````

> 跨项目依赖：
>
> * 前端插件页面来自 `clients/ui`，例如 `clients/ui/workflow-designer`
> * 通用业务组件（MetricOverview、AlertList 等）也在 `clients/ui` 中，由多个前端项目共享

---

## 🚀 启动与开发

> ⚠️ 本仓库使用 **Workspace（多包）结构**，依赖应在仓库根目录统一安装。

### 1. 安装依赖（在仓库根目录）

```bash
cd /path/to/monitor-ai-bot
npm install
```

### 2. 启动 Dashboard 开发环境

```bash
cd dashboard-frontend-react
npm run dev
```

Vite 默认启动在 `http://127.0.0.1:5173` 或 `http://127.0.0.1:5174`（端口冲突时）。

### 3. 构建与预览

```bash
# 构建
npm run build

# 预览构建结果
npm run preview
```

---

## 🧱 布局架构

整体布局为经典的 **Topbar + Sidebar + 内容区** 三段式管理控制台：

```text
┌─────────────────────────────────────────────────────────────┐
│ 顶栏 Topbar                                                │
│  左：Logo / 项目名 / 当前环境      右：搜索 / 用户 / 设置等   │
├───────────────┬────────────────────────────────────────────┤
│ 侧边栏 Sidebar│  右侧主体：按路由切换的页面                   │
│ - 总览        │  ┌───────────────────────────────────────┐ │
│ - 指标        │  │ 面包屑 / 页面标题                     │ │
│ - 日志        │  ├───────────────────────────────────────┤ │
│ - 告警        │  │ 页面内容：图表 / 表格 / 卡片等         │ │
│ - 工作流      │  │                                       │ │
│ - Agent管理   │  └───────────────────────────────────────┘ │
│ - 插件管理    │                                            │
│ - 系统设置    │                                            │
└───────────────┴────────────────────────────────────────────┘
```

### 关键布局组件

#### `layout/Topbar.tsx`

* 显示项目 Logo / 名称
* 环境标签（如 `DEV` / `STAGING`）
* 全局搜索输入框（预留）
* 用户信息 / 设置按钮（预留）
* 可通过 props 触发侧边栏折叠（窄屏支持）

#### `layout/Sidebar.tsx`

* 左侧导航栏，分区（对应 `NavSection`）：

  * 监控：总览 / 指标 / 日志 / 告警中心
  * 工作流：工作流列表 + 工作流相关插件
  * Agent：Agent 管理
  * 插件：插件管理
  * 系统：系统设置
* 菜单来源：

  * 静态菜单：来自 `baseSections`
  * 动态菜单：通过 `getPlugins()` 自动挂载前端插件

#### `layout/DashboardLayout.tsx`

* 顶级布局容器，包含：

  * `<Topbar />`
  * `<Sidebar />`
  * 右侧 `<main>` 部分：

    * 面包屑 / 页面标题区域
    * `<Outlet />`（渲染各路由页面）
* 被 React Router 作为“嵌套路由 Layout”使用。

---

## 🔀 路由与页面

路由使用 **React Router v6**，在 `App.tsx` 中定义整体结构：

```tsx
// src/App.tsx（核心结构简化版）

import React, { Suspense } from "react";
import { Routes, Route } from "react-router-dom";
import DashboardLayout from "./layout/DashboardLayout";
import DashboardHome from "./pages/DashboardHome";
import MetricsPage from "./pages/MetricsPage";
import LogsPage from "./pages/LogsPage";
import AlertsPage from "./pages/AlertsPage";
import AgentsPage from "./pages/AgentsPage";
import PluginsPage from "./pages/PluginsPage";
import SettingsPage from "./pages/SettingsPage";

import "./App.css";
import "./plugins/loader";
import { getPlugins } from "./plugins/plugin-registry";

const App: React.FC = () => {
  const plugins = getPlugins();

  return (
    <Routes>
      <Route element={<DashboardLayout />}>
        {/* 总览页：根路径 "/" 以及 "/overview" */}
        <Route path="/" element={<DashboardHome />} />
        <Route path="/overview" element={<DashboardHome />} />

        {/* 固定功能页 */}
        <Route path="/metrics" element={<MetricsPage />} />
        <Route path="/logs" element={<LogsPage />} />
        <Route path="/alerts" element={<AlertsPage />} />
        <Route path="/workflows" element={<div>工作流列表（TODO）</div>} />
        <Route path="/agents" element={<AgentsPage />} />
        <Route path="/plugins" element={<PluginsPage />} />
        <Route path="/settings" element={<SettingsPage />} />

        {/* 插件自动挂载的路由 */}
        {plugins.map((plugin) => (
          <Route
            key={plugin.id}
            path={plugin.route}
            element={
              <Suspense fallback={<div style={{ padding: 24 }}>加载插件...</div>}>
                <plugin.component />
              </Suspense>
            }
          />
        ))}
      </Route>
    </Routes>
  );
};

export default App;
```

### 内置页面简要说明

* `DashboardHome`（`/`、`/overview`）：总览页，建议展示 CPU / API / Agent 等关键 KPI
* `MetricsPage`（`/metrics`）：指标列表 / 可视化图表
* `LogsPage`（`/logs`）：日志查询 / 查看
* `AlertsPage`（`/alerts`）：告警中心
* `AgentsPage`（`/agents`）：Agent 管理
* `PluginsPage`（`/plugins`）：后端插件 / 前端插件管理
* `SettingsPage`（`/settings`）：系统设置

所有页面通常使用统一的容器组件：

#### `components/PageCard.tsx`

```tsx
<PageCard title="指标面板（Metrics）">
  {/* 页面内容 */}
</PageCard>
```

用于右侧内容区的统一样式包裹（白底、圆角、阴影、标题区域）。

---

## 🧩 前端插件系统

Dashboard 提供一个 **可插拔的前端插件机制**，用于挂载如：

* 工作流设计器（workflow-designer）
* 通知中心 UI
* AI 分析面板
* 其它业务/监控插件

### 类型定义：`src/plugins/types.ts`

```ts
export type FrontendPluginCategory =
  | "monitor"
  | "workflow"
  | "notification"
  | "agent"
  | "plugin"
  | "system"
  | "custom";

export interface FrontendPlugin {
  id: string;                          // 唯一 ID
  title: string;                       // 在导航中展示的标题
  route: string;                       // 对应路由路径，如 "/workflow-designer"
  component: React.ComponentType<any>; // 页面组件
  category: FrontendPluginCategory;    // 挂载到 Sidebar 哪个分组
  order?: number;                      // 同分组下排序
}
```

### 注册中心：`src/plugins/plugin-registry.ts`

```ts
import type { FrontendPlugin } from "./types";

const plugins: FrontendPlugin[] = [];

export const registerPlugin = (plugin: FrontendPlugin) => {
  if (plugins.some((p) => p.id === plugin.id)) {
    console.warn(`Plugin with id "${plugin.id}" is already registered.`);
    return;
  }
  plugins.push(plugin);
};

export const getPlugins = (): FrontendPlugin[] => {
  return [...plugins].sort((a, b) => (a.order ?? 99) - (b.order ?? 99));
};
```

### 自动加载插件 entry：`src/plugins/loader.ts`

```ts
// 自动加载 src/plugins/*/entry.ts
const modules = import.meta.glob("./*/entry.ts", { eager: true });
export {};
```

### 示例：工作流设计器插件注册

各插件在 `src/plugins/<plugin-name>/entry.ts` 中调用 `registerPlugin` 即可：

```ts
// src/plugins/workflow-designer/entry.ts
import { registerPlugin } from "../plugin-registry";
import type { FrontendPlugin } from "../types";

// 从 clients/ui/workflow-designer 引入页面
import { WorkflowManagementPage } from "../../../clients/ui/workflow-designer";

const plugin: FrontendPlugin = {
  id: "workflow-designer",
  title: "工作流设计器",
  route: "/workflow-designer",
  component: WorkflowManagementPage,
  category: "workflow",
  order: 10,
};

registerPlugin(plugin);
```

> `App.tsx` 中 `import "./plugins/loader";` 会触发所有 entry 文件执行，从而完成插件注册。

### Sidebar 中的动态菜单挂载

`layout/Sidebar.tsx` 中：

1. 定义基础分组 `baseSections`
2. 调用 `getPlugins()` 拿到所有前端插件
3. 按 `plugin.category` 挂载到对应分组下

```ts
import { getPlugins } from "../plugins/plugin-registry";

const baseSections = [
  {
    id: "monitor",
    title: "监控",
    items: [
      { id: "overview", label: "总览", path: "/overview" },
      { id: "metrics", label: "指标", path: "/metrics" },
      { id: "logs", label: "日志", path: "/logs" },
      { id: "alerts", label: "告警中心", path: "/alerts" },
    ],
  },
  // workflow / agent / plugin / system ...
];

function buildSections() {
  const sections = baseSections.map((s) => ({ ...s, items: [...s.items] }));
  const plugins = getPlugins();

  plugins.forEach((plugin) => {
    const target = sections.find((s) => s.id === plugin.category);
    if (target) {
      target.items.push({
        id: plugin.id,
        label: plugin.title,
        path: plugin.route,
      });
    }
  });

  return sections;
}
```

因此：

* 新增前端插件 = 新增一个 `entry.ts` + 调用 `registerPlugin`
* 路由与 Sidebar 菜单都会自动生效，无需手动改 App.tsx 或 Sidebar.tsx

---

## 🌐 与 api-server 的 HTTP API 约定

Dashboard 主要通过 `api-server` 暴露的 HTTP API 获取数据并驱动 UI 展示。关键约定如下：

### 1. 核心监控数据接口

* `GET /metrics`

  * 功能：查询指标（Metric）
  * 用于：Dashboard 总览、`MetricsPage`、`MetricOverview` 等
* `GET /logs`

  * 功能：查询日志（LogEvent）
  * 用于：`LogsPage`
* `GET /alerts`

  * 功能：查询告警（AlertEvent）
  * 用于：`AlertsPage`、告警面板、告警侧边抽屉等

接口返回的字段应与后端 `core-types::Metric` / `LogEvent` / `AlertEvent` 对应，例如（简化示意）：

```ts
// Metric
type MetricDto = {
  time: string;           // ISO 时间
  plugin: string;
  name: string;
  value: number;
  labels: Record<string, string>;
};

// LogEvent
type LogEventDto = {
  time: string;
  level: "Info" | "Warning" | "Error" | "Debug";
  plugin?: string;
  message: string;
  fields: Record<string, string>;
};

// AlertEvent
type AlertEventDto = {
  time: string;
  plugin: string;
  metric_name: string;
  severity: "Info" | "Warning" | "Critical";
  title: string;
  message: string;
};
```

前端通常会在 `clients/ui/hooks` 中实现对应的请求钩子（如 `useMetrics`、`useAlerts`），Dashboard 和 Web Client 共享使用。

### 2. Agent 上报入口（给 Agent 用）

* `POST /agent/metrics`

  * 用于：Agent 探针（Rust bin）上报指标
  * Dashboard 一般不会直接调用，但会展示它写入的指标（如 Agent CPU、内存）

### 3. 插件 API 网关

* `ANY /plugin-api/{plugin}/*rest`

作用：

* 统一入口转发到各插件内部 HTTP Server
* `api-server` 会从 SQLite `plugin_apis` 表中查 `plugin -> base_url`
* 再将请求转发到相应插件的 `base_url + rest`

示例：

* `GET /plugin-api/workflow-engine/workflows`
* `POST /plugin-api/workflow-engine/execute`
* `GET /plugin-api/ai-analyzer/status`

> 对 Dashboard / Web Client / Mobile 来说，它们只认 `api-server` 的 HTTP 地址，不关心插件实际监听的端口。

---

## 📦 在 Dashboard 中使用 `clients/ui` 的通用组件

`clients/ui` 是前端共享组件库，目前已经包含：

* `components/MetricOverview.tsx`
* `components/AlertList.tsx`
* `hooks/useMetrics.ts`
* `hooks/useAlerts.ts`
* `workflow-designer/*`（工作流设计器相关）

### 1. 安装与引用方式

由于使用 Workspace 结构，`dashboard-frontend-react` 可以直接通过相对路径引用：

```ts
// 示例：在 DashboardHome 或 MetricsPage 中
import MetricOverview from "../../clients/ui/components/MetricOverview";
import AlertList from "../../clients/ui/components/AlertList";
```

如果未来把 `clients/ui` 封装成包（如 `@monitor/ui`），再改为：

```ts
import { MetricOverview, AlertList } from "@monitor/ui";
```

### 2. MetricOverview 使用示例

```tsx
import React from "react";
import PageCard from "../components/PageCard";
import MetricOverview from "../../clients/ui/components/MetricOverview";

const MetricsPage: React.FC = () => {
  return (
    <PageCard title="指标面板（Metrics）">
      <MetricOverview
        apiBase={import.meta.env.VITE_API_BASE || "http://127.0.0.1:3001"}
      />
    </PageCard>
  );
};

export default MetricsPage;
```

组件内部会调用 `GET /metrics`（通常通过 `clients/ui/hooks/useMetrics`），将返回的数据按卡片形式展示。

### 3. AlertList 使用示例

```tsx
import React from "react";
import PageCard from "../components/PageCard";
import AlertList from "../../clients/ui/components/AlertList";

const AlertsPage: React.FC = () => {
  return (
    <PageCard title="告警中心（Alerts）">
      <AlertList
        apiBase={import.meta.env.VITE_API_BASE || "http://127.0.0.1:3001"}
      />
    </PageCard>
  );
};

export default AlertsPage;
```

`AlertList` 一般会调用 `GET /alerts` 并以列表形式展示告警记录（可按 severity、时间、插件等过滤）。

> 推荐约定：
>
> * 所有前端项目通过同一 `VITE_API_BASE` 配置 `api-server` 地址
> * `clients/ui` 中的 hooks / 组件都接受 `apiBase` 作为可选 props，方便不同环境切换。

---

## 🔁 与工作流引擎插件（workflow-engine）的前后端协作流程

工作流相关前端有两块：

* Dashboard 内的 **工作流设计器 UI 插件**：`clients/ui/workflow-designer`
* 后端 **工作流引擎插件**：`plugins/workflow-engine`（通过 `plugin-api` 暴露 HTTP API）

### 1. 典型交互路径

1. 用户在 Dashboard 打开 “工作流设计器” 菜单（`/workflow-designer`）
2. 前端加载 `WorkflowManagementPage`（来自 `clients/ui/workflow-designer`）
3. 该页面调用 `api-server` 暴露的插件 API，例如：

   * `GET  /plugin-api/workflow-engine/workflows`        — 获取全部工作流列表
   * `GET  /plugin-api/workflow-engine/workflows/{id}`   — 获取单个工作流定义
   * `POST /plugin-api/workflow-engine/workflows`        — 创建/更新工作流
   * `DELETE /plugin-api/workflow-engine/workflows/{id}` — 删除工作流
   * `POST /plugin-api/workflow-engine/execute`          — 测试执行某个工作流
4. `api-server` 根据 `plugin_apis` 中的映射，将请求转发到 `workflow-engine` 插件内部 HTTP Server
5. 插件执行 LogicFlow JSON / workflow-core 逻辑，并返回结果
6. 前端将执行成功/失败、耗时、变量输出等信息可视化展示

### 2. 前端代码示意（WorkflowDesignerPage）

```tsx
// 来自 clients/ui/workflow-designer（简化示例）
const API_BASE = import.meta.env.VITE_API_BASE || "http://127.0.0.1:3001";

async function fetchWorkflows() {
  const res = await fetch(`${API_BASE}/plugin-api/workflow-engine/workflows`);
  return res.json();
}

async function saveWorkflow(definition: any) {
  await fetch(`${API_BASE}/plugin-api/workflow-engine/workflows`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(definition),
  });
}

async function executeWorkflow(payload: { id: string; input: any }) {
  await fetch(`${API_BASE}/plugin-api/workflow-engine/execute`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
}
```

### 3. 与 LogicFlow / workflow-core 的关系

* 前端使用 **LogicFlow** 图形编辑器（在 `clients/ui/workflow-designer` 内）

  * 用户通过拖拽节点、连线来编排工作流
  * 最终生成一个 LogicFlow JSON（包含节点、边、位置信息等）
* 后端 `workflow-engine` 插件负责：

  * 将 LogicFlow JSON 映射到内部 `workflow-core` 的 `Workflow` / `Step` 结构
  * 负责执行 API 调用、变量注入、断言、错误收集等
  * 将执行结果回写到 Metric / Log / Alert，供 Dashboard 其他页面展示

### 4. Dashboard 与 workflow-engine 的职责边界

* Dashboard（前端）：

  * 负责工作流的 **可视化编辑 / 列表管理 / 手动执行**
  * 只与 `api-server` 通信（`/plugin-api/workflow-engine/...`）
  * **不直接操作数据库 / 不直接调用插件内部端口**
* workflow-engine 插件（后端）：

  * 提供工作流的持久化 / 执行引擎
  * 对外接口统一通过 `api-server` 的插件 API 网关
  * 本身可以运行在 `bot-host` 加载的 Rust 插件里，独立演进

---

## 🎨 样式结构

当前样式分为两层：

### 1. `src/index.css`（全局基础样式）

* 重置：`html, body, #root` 的 margin / padding / height
* 全局 `box-sizing: border-box`
* 全局字体、行高、文本颜色、背景色
* `<a>` 和 `<button>` 的基础样式

### 2. `src/App.css`（Dashboard 布局样式）

* `.dashboard-root`：根布局容器
* `.topbar*`：顶部导航条样式
* `.sidebar*`：侧边栏导航样式
* `.dashboard-content` / `.dashboard-page-*`：右侧内容区域样式
* 支持 Sidebar 折叠（`.sidebar-collapsed`）

> 后续如果页面业务样式变复杂，可以考虑：
>
> * 将公共布局保留在 `App.css`
> * 业务组件使用 CSS Modules 或 Tailwind

---

## ➕ 如何扩展

### 新增一个主导航页面

1. 在 `src/pages/` 下创建页面组件，例如 `XxxPage.tsx`

2. 在 `App.tsx` 中注册路由：

   ```tsx
   import XxxPage from "./pages/XxxPage";
   <Route path="/xxx" element={<XxxPage />} />
   ```

3. 在 `layout/Sidebar.tsx` 的 `baseSections` 中对应分组增加一条菜单：

   ```ts
   { id: "xxx", label: "XXX 管理", path: "/xxx" },
   ```

---

### 新增一个前端插件页面（推荐方式）

1. 在 `clients/ui` 中实现实际页面，例如 `clients/ui/ai-analyzer/`

2. 在 `src/plugins/ai-analyzer/entry.ts` 中注册插件：

   ```ts
   import { registerPlugin } from "../plugin-registry";
   import type { FrontendPlugin } from "../types";
   import { AiAnalyzerPage } from "../../../clients/ui/ai-analyzer";

   const plugin: FrontendPlugin = {
     id: "ai-analyzer",
     title: "AI 分析面板",
     route: "/ai-analyzer",
     component: AiAnalyzerPage,
     category: "monitor", // 或 "custom"
     order: 20,
   };

   registerPlugin(plugin);
   ```

3. 无需修改 Sidebar 或 App.tsx，插件即可自动出现在菜单和路由中。

---

这份 README 更聚焦于：

* Dashboard 自身的布局与结构
* 与 `api-server` 的 HTTP API 协作方式
* 如何使用 `clients/ui` 提供的通用组件
* 与 workflow-engine 插件的前后端联动流程

团队新同事看这一份，就能很快理解整个控制台前端怎么跑、怎么扩展、怎么接后端。
你后面只要在实现具体页面时照这个约定走，就会非常顺。🚀

```
```
