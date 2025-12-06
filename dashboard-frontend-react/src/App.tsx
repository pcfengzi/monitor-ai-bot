// src/App.tsx
import React, { Suspense } from "react";
import { Routes, Route } from "react-router-dom";
import DashboardLayout from "./layout/DashboardLayout";
import DashboardHome from "./pages/DashboardHome";

import "./App.css";

// 自动加载所有插件 entry.ts（在里面调用 registerPlugin）
import "./plugins/loader";
import { getPlugins } from "./plugins/plugin-registry";

const App: React.FC = () => {
  const plugins = getPlugins();

  return (
    <Routes>
      {/* 整个控制台都用 DashboardLayout 包裹 */}
      <Route element={<DashboardLayout />}>
        {/* 总览页：根路径 "/" */}
        <Route path="/" element={<DashboardHome />} />

        {/* 预留的一堆页面（你后面慢慢做真正内容） */}
        <Route path="/metrics" element={<div>指标页面（TODO）</div>} />
        <Route path="/logs" element={<div>日志页面（TODO）</div>} />
        <Route path="/alerts" element={<div>告警中心（TODO）</div>} />
        <Route path="/workflows" element={<div>工作流列表（TODO）</div>} />
        <Route path="/agents" element={<div>Agent 管理（TODO）</div>} />
        <Route path="/plugins" element={<div>插件管理（TODO）</div>} />
        <Route path="/settings" element={<div>系统设置（TODO）</div>} />

        {/* 插件自动生成的路由 */}
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
