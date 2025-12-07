下面是一段 **专业、清晰、简洁、适合团队协作的「前端 Workspace 使用说明」**，你可以直接加入根目录 `README.md` 的 “开发环境 / 前端说明” 部分。

---

# 🧩 Frontend Workspace 使用说明（适用于 dashboard / UI 组件库 / 多前端客户端）

Monitor AI Bot 前端部分采用 **Node Workspaces（Monorepo）结构**，用于统一管理多个前端项目及其共享 UI 组件库。这种组织模式可让各前端子项目共享依赖、共享组件、共享构建工具，并且非常适合本项目的插件化架构。

当前前端项目包括：

```
dashboard-frontend-react/     ← 管理控制台
clients/ui/                   ← UI 组件库（工作流设计器等插件）
clients/web-client/           ← Web 客户端
clients/desktop-client/       ← Tauri 桌面客户端
clients/mobile-client/        ← React Native App
```

所有这些前端项目都属于同一个 Workspace，由根目录统一管理依赖。

---

## 📦 1. Workspaces 配置（根目录 package.json）

根目录的 `package.json` 定义了 workspace 项目范围：

```json
{
  "name": "monitor-ai-bot",
  "private": true,
  "workspaces": [
    "dashboard-frontend-react",
    "clients/ui",
    "clients/web-client",
    "clients/desktop-client",
    "clients/mobile-client"
  ]
}
```

在 Workspace 中：

* 所有依赖安装都建议通过 **根目录执行**
* 根目录的 `node_modules` 会成为所有前端项目的共享依赖树
* 各子项目保留自己的 `package.json`，但依赖自动链接，无需重复安装

---

## 📥 2. 安装依赖（统一入口）

> **请始终在根目录执行依赖操作，而不是在子项目中。**

安装所有前端项目的依赖：

```bash
npm install
```

这样：

* `react` / `react-dom` 等核心依赖只安装一次
* `clients/ui`（共享组件库）可以直接使用同一份依赖
* dashboard 与 workflow-designer 等插件间不会产生版本冲突

---

## ▶️ 3. 运行某个前端项目

进入目标项目并运行即可。例如：

### Dashboard 控制台：

```bash
cd dashboard-frontend-react
npm run dev
```

### Web 客户端：

```bash
cd clients/web-client
npm run dev
```

### Desktop / Mobile 类似。

---

## 🔁 4. 修改共享 UI 组件库（clients/ui）

`clients/ui` 是所有前端项目共享的 UI/插件组件库（如工作流设计器）。

修改这里的代码后：

* 会自动被 Dashboard/Web/Client 等前端引用
* 不需要重新打包或发布
* 不需要 npm link / yarn link

Vite 会自动重新编译依赖的文件，极大提高开发效率。

---

## 📦 5. 如何为某个子项目添加依赖？

推荐在根目录添加依赖：

```bash
npm install zustand
npm install reactflow
npm install axios
```

npm workspaces 会自动将这些依赖链接给需要它们的所有前端项目。

如果某个依赖只属于某个子项目，也可以在子项目目录单独添加：

```bash
cd clients/web-client
npm install echarts --workspace clients/web-client
```

但一般建议统一安装，便于管理和复用。

---

## 🧪 6. 如何开发前端插件（如 workflow-designer）？

1. 在 `clients/ui/<plugin-name>/` 开发前端插件页面
2. 导出组件（如 `WorkflowDesignerPage`）
3. 在 dashboard 的 `src/plugins/<plugin>/entry.ts` 调用 `registerPlugin()`
4. Dashboard 会自动加载插件、自动生成路由、自动出现在 Sidebar 菜单中

无需额外配置。

---

## 🚀 7. Workspace 的好处总结

| 能力          | 作用                                 |
| ----------- | ---------------------------------- |
| 📦 单一依赖树    | 避免重复安装依赖、版本冲突、路径无法解析等问题            |
| 🔗 子项目自动链接  | dashboard 不需要复杂路径就能引用 clients/ui   |
| ⚡ 热更新高效     | Vite 自动刷新跨包代码，无需构建 UI 包            |
| 🧩 插件架构天然支持 | 为工作流设计器、AI 分析器、插件管理器等 UI 插件打好基础    |
| 👥 团队协作更流畅  | clone → npm install → 任何前端项目都能直接运行 |

---

## 📘 8. 小贴士

* **永远不要在子项目随意执行 `npm install`**（除非明确使用 `--workspace`）
* 如果出现依赖解析错误（如 “找不到 React”），通常是因为依赖只安装在子项目目录
* 修改 UI 组件库后，无需构建，直接在 dashboard / web-client 等项目中实时生效

---

如果需要，我还可以为你补充：

* pnpm workspace 版本
* TurboRepo / Nx 加速工作流
* 为 clients/ui 添加构建配置（发布 npm 包或内部分享）
* 完整的前端插件开发规范 README

你想继续增强前端工程体系吗？
