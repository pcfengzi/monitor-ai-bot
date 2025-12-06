下面这份你可以直接当成 `dashboard-frontend-react` 子项目的 `README.md` 用，有需要再按实际情况微调下项目名和截图。

---

# Monitor AI Bot Dashboard（dashboard-frontend-react）

Monitor AI Bot Dashboard 是整个 **Monitor AI Bot 平台的管理控制台前端**，用于：

* 查看核心监控指标（CPU、API 流程、Agent 状态等）
* 管理工作流、插件、分布式 Agent
* 作为前端插件（workflow-designer 等）的集成入口

技术栈：

* **React 18** + **TypeScript**
* **Vite** 开发构建
* **React Router v6** 路由
* 自定义 **插件注册机制**（前端插件自动挂载）
* 原子化 CSS（`index.css` + `App.css`），后续可平滑迁移到 Tailwind / CSS Modules

---

## 目录结构概览

```text
dashboard-frontend-react/
├── public/                 # 静态资源
├── src/
│   ├── assets/             # 静态图片、图标等（预留）
│   ├── components/
│   │   └── PageCard.tsx    # 通用页面卡片容器
│   ├── layout/
│   │   ├── DashboardLayout.tsx  # 顶栏 + 侧边栏 + 主体布局
│   │   ├── Sidebar.tsx          # 左侧导航
│   │   └── Topbar.tsx           # 顶部导航
│   ├── pages/              # 路由页面
│   │   ├── DashboardHome.tsx    # 总览页（Dashboard 首页）
│   │   ├── MetricsPage.tsx      # 指标页
│   │   ├── LogsPage.tsx         # 日志页
│   │   ├── AlertsPage.tsx       # 告警中心
│   │   ├── AgentsPage.tsx       # Agent 管理
│   │   ├── PluginsPage.tsx      # 插件管理
│   │   └── SettingsPage.tsx     # 系统设置
│   ├── plugins/
│   │   ├── loader.ts            # 自动加载各插件 entry
│   │   ├── plugin-registry.ts   # 前端插件注册表（registerPlugin/getPlugins）
│   │   └── types.ts             # 前端插件类型定义 FrontendPlugin
│   ├── routes/             # （预留，将来可拆 Router 配置）
│   ├── store/              # （预留，全局状态）
│   ├── App.tsx             # 顶层路由结构（嵌套 DashboardLayout）
│   ├── main.tsx            # React 入口，挂载 BrowserRouter + App
│   ├── App.css             # 布局相关样式（Topbar/Sidebar/Content）
│   └── index.css           # 全局 reset + 基础样式
│
├── package.json
├── tsconfig.json
├── vite.config.ts
└── README.md               # 本文档
```

> 跨项目依赖：
>
> * 前端插件页面来自 `clients/ui`，例如 `clients/ui/workflow-designer`
> * 后续业务通用组件也建议放在 `clients/ui`，由各前端项目共享

---

## 启动与开发

在仓库根目录或 `dashboard-frontend-react/` 目录下：

```bash
# 安装依赖
npm install

# 开发模式
npm run dev

# 打包
npm run build

# 预览构建结果
npm run preview
```

Vite 默认开发地址为 `http://127.0.0.1:5173`（以实际配置为准）。

---

## 布局架构

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

### 关键文件

* **`layout/Topbar.tsx`**

  * 显示项目 Logo、环境标签（如 DEV）、全局搜索框、用户信息、设置按钮
  * 预留了 `onToggleSidebar` 回调，可折叠/展开侧边栏（手机端/窄屏场景）

* **`layout/Sidebar.tsx`**

  * 左侧导航栏，分区：监控 / 工作流 / Agent / 插件 / 系统
  * 静态菜单项：总览、指标、日志、告警、Agent 管理、插件管理、系统设置
  * **动态菜单项**：从前端插件注册表 `getPlugins()` 中读取，自动挂到对应分组

* **`layout/DashboardLayout.tsx`**

  * 顶层布局组件，包含：

    * `<Topbar />`
    * `<Sidebar />`
    * 右侧 `<main>` 内部的 面包屑 + `<Outlet />`（承载具体页面）
  * 被 React Router 作为嵌套路由的 Layout 使用

---

## 路由与页面

路由使用 **React Router v6**，核心结构写在 `App.tsx` 中：

```tsx
// App.tsx 中的核心结构（简化版）

<Routes>
  <Route element={<DashboardLayout />}>
    {/* 总览（Dashboard 首页） */}
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

    {/* 插件自动挂载的路由（见下文插件系统） */}
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
```

### 内置页面说明

* **`DashboardHome`**：总览页（Dashboard）

  * 默认路径 `/` 和 `/overview`
  * 适合放整体 CPU / API / Agent 等核心指标总览

* **`MetricsPage`**：指标页 `/metrics`

  * 当前使用 `<PageCard>` 做简单占位
  * 后续接入 `/metrics` 接口 + ECharts 或 `clients/ui` 的 `MetricOverview`

* **`LogsPage`**：日志页 `/logs`

* **`AlertsPage`**：告警中心 `/alerts`

* **`AgentsPage`**：Agent 管理 `/agents`

* **`PluginsPage`**：插件管理 `/plugins`

* **`SettingsPage`**：系统设置 `/settings`

所有这些页面都采用了统一的容器组件：

### 通用页面卡片：`components/PageCard.tsx`

```tsx
// 用于在右侧内容区渲染一个统一风格的白色卡片
<PageCard title="指标面板（Metrics）">
  {/* 页面内容 */}
</PageCard>
```

---

## 前端插件系统

Dashboard 支持 **前端插件自动挂载**，用于集成诸如：

* `workflow-designer` 工作流设计器页面
* 未来的通知中心 UI、AI 分析面板等

### 类型定义：`plugins/types.ts`

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
  id: string;                      // 唯一 ID
  title: string;                   // 在导航中展示的标题
  route: string;                   // 对应路由路径，如 "/workflow-designer"
  component: React.ComponentType<any>; // 页面组件
  category: FrontendPluginCategory;    // 挂载到 Sidebar 哪个分组
  order?: number;                  // 同分组下排序
}
```

### 注册中心：`plugins/plugin-registry.ts`

```ts
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

### 自动加载插件 entry：`plugins/loader.ts`

```ts
// 自动加载 src/plugins/*/entry.ts
const modules = import.meta.glob("./*/entry.ts", {
  eager: true,
});

export {};
```

各插件在 `src/plugins/<plugin-name>/entry.ts` 中调用 `registerPlugin` 完成自注册，例如（示例）：

```ts
// src/plugins/workflow-designer/entry.ts
import { registerPlugin } from "../plugin-registry";
import type { FrontendPlugin } from "../types";

// 来自 clients/ui/workflow-designer
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

> `plugins/loader.ts` 会在 `App.tsx` 中被 `import "./plugins/loader";`，
> 从而在应用启动时自动执行所有 entry 文件，完成插件注册。

### Sidebar 中的插件菜单挂载

`layout/Sidebar.tsx` 中：

1. 定义静态分组 `baseSections`
2. 调用 `getPlugins()` 取出所有注册插件
3. 按 `plugin.category` 把插件菜单挂到对应分组下

```ts
const baseSections: NavSection[] = [
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

function buildSections(): NavSection[] {
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

这样：

* **新增插件** → 只需要：增加 `entry.ts` + 注册 `FrontendPlugin`
* **Sidebar 菜单 & 路由** 都会自动增加插件对应的页面

---

## 样式结构

项目目前使用两层样式文件：

1. **`index.css`**：全局基础样式

   * Reset：`html, body, #root` 高度、margin、padding
   * 全局 `box-sizing: border-box`
   * 默认字体、文本颜色、背景色
   * `<a>`、`<button>` 的基础样式（可按需求调整或删减）

2. **`App.css`**：Dashboard 布局样式

   * `.dashboard-root`：根容器，`flex + column`，撑满全屏
   * `.topbar*`：顶栏相关
   * `.sidebar*`：侧边栏相关
   * `.dashboard-content` / `.dashboard-page-*`：右侧内容区域布局

> 业务组件的局部样式建议：
>
> * 简单场景用内联 `style`（当前已采用这种方式）
> * 后续如果变复杂，可以在组件内部使用 CSS Modules 或引入 Tailwind

---

## 如何扩展

### 新增一个主导航页面

1. 在 `src/pages/` 下新增 `XxxPage.tsx`

2. 在 `App.tsx` 里增加路由：

   ```tsx
   import XxxPage from "./pages/XxxPage";

   <Route path="/xxx" element={<XxxPage />} />
   ```

3. 在 `layout/Sidebar.tsx` 中的 `baseSections` 对应分组中增加菜单项：

   ```ts
   { id: "xxx", label: "XXX 管理", path: "/xxx" },
   ```

---

### 新增一个前端插件页面

1. 在 `clients/ui` 中实现插件 UI（例如 `clients/ui/notification-center`）

2. 在 `src/plugins/notification-center/entry.ts` 中注册插件：

   ```ts
   import { registerPlugin } from "../plugin-registry";
   import type { FrontendPlugin } from "../types";
   import { NotificationCenterPage } from "../../../clients/ui/notification-center";

   const plugin: FrontendPlugin = {
     id: "notification-center",
     title: "通知中心",
     route: "/notification-center",
     component: NotificationCenterPage,
     category: "plugin",
     order: 20,
   };

   registerPlugin(plugin);
   ```

3. 无需修改 Sidebar / App.tsx，菜单和路由将自动生效。

---

后续如果要在 README 里补充：

* 与 `api-server` 的接口约定（/metrics /logs /alerts）
* 与后端插件（workflow-engine、notification-center）的路由约定（/plugin-api/{plugin}/...）
* 使用 `clients/ui` 的 `MetricOverview` / `AlertList` 等通用组件

可以再加一个「后端交互与数据流」章节，专门讲 API 设计。
