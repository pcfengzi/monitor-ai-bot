// src/App.tsx
import React, { Suspense } from "react";
import { Routes, Route } from "react-router-dom";
import DashboardLayout from "./layout/DashboardLayout";
import DashboardHome from "./pages/DashboardHome";

import "./App.css";

// 自动加载插件 entry.ts
import "./plugins/loader";
import { getPlugins } from "./plugins/plugin-registry";

// 其它页面
import MetricsPage from "./pages/MetricsPage";
import LogsPage from "./pages/LogsPage";
import AlertsPage from "./pages/AlertsPage";
import AgentsPage from "./pages/AgentsPage";
import PluginsPage from "./pages/PluginsPage";
import SettingsPage from "./pages/SettingsPage";

const App: React.FC = () => {
  const plugins = getPlugins();

  return (
    <Routes>
      <Route element={<DashboardLayout />}>
        
        {/* ---- 总览页面（DashboardHome） ---- */}
        <Route path="/" element={<DashboardHome />} />
        <Route path="/overview" element={<DashboardHome />} />

        {/* ---- 主导航页面 ---- */}
        <Route path="/metrics" element={<MetricsPage />} />
        <Route path="/logs" element={<LogsPage />} />
        <Route path="/alerts" element={<AlertsPage />} />
        <Route path="/workflows" element={<div>工作流列表（TODO）</div>} />
        <Route path="/agents" element={<AgentsPage />} />
        <Route path="/plugins" element={<PluginsPage />} />
        <Route path="/settings" element={<SettingsPage />} />

        {/* ---- 插件动态挂载 ---- */}
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
