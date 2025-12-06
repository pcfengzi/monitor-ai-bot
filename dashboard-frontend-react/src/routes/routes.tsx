// src/router/routes.tsx
import React from "react";
import { RouteObject } from "react-router-dom";
import { frontendPlugins } from "../plugins/frontend-registry";

import OverviewPage from "../pages/OverviewPage";
import MetricsPage from "../pages/MetricsPage";
import LogsPage from "../pages/LogsPage";
import AlertsPage from "../pages/AlertsPage";
import AgentsPage from "../pages/AgentsPage";
import PluginsPage from "../pages/PluginsPage";
import SettingsPage from "../pages/SettingsPage";

export const baseRoutes: RouteObject[] = [
  { path: "/overview", element: <OverviewPage /> },
  { path: "/metrics", element: <MetricsPage /> },
  { path: "/logs", element: <LogsPage /> },
  { path: "/alerts", element: <AlertsPage /> },
  { path: "/agents", element: <AgentsPage /> },
  { path: "/plugins", element: <PluginsPage /> },
  { path: "/settings", element: <SettingsPage /> },
];

// 插件路由动态挂载
export const pluginRoutes: RouteObject[] = frontendPlugins.map((p) => ({
  path: p.route,
  element: <p.component />,
}));

export const allRoutes: RouteObject[] = [...baseRoutes, ...pluginRoutes];
