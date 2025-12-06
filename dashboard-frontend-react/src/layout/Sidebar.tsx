// src/layout/Sidebar.tsx
import React from "react";
import { NavLink } from "react-router-dom";
import "../App.css";

import { getPlugins } from "../plugins/plugin-registry";
import type { FrontendPlugin } from "../plugins/types";

export type NavSection = {
  id: string;
  title: string;
  items: { id: string; label: string; path: string }[];
};

const baseSections: NavSection[] = [
  {
    id: "monitor",
    title: "监控",
    items: [
      { id: "overview", label: "总览", path: "/" }, // 总览对应 "/"
      { id: "metrics", label: "指标", path: "/metrics" },
      { id: "logs", label: "日志", path: "/logs" },
      { id: "alerts", label: "告警中心", path: "/alerts" },
    ],
  },
  {
    id: "workflow",
    title: "工作流",
    items: [{ id: "workflows", label: "工作流列表", path: "/workflows" }],
  },
  {
    id: "agent",
    title: "Agent",
    items: [{ id: "agents", label: "Agent 管理", path: "/agents" }],
  },
  {
    id: "plugin",
    title: "插件",
    items: [{ id: "plugins", label: "插件管理", path: "/plugins" }],
  },
  {
    id: "system",
    title: "系统",
    items: [{ id: "settings", label: "系统设置", path: "/settings" }],
  },
];

function buildSections(): NavSection[] {
  const sections = baseSections.map((s) => ({ ...s, items: [...s.items] }));
  const plugins: FrontendPlugin[] = getPlugins();

  // 把所有插件挂在对应分组（按 category）下面
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

interface SidebarProps {
  collapsed?: boolean;
}

const Sidebar: React.FC<SidebarProps> = ({ collapsed }) => {
  const sections = buildSections();

  return (
    <aside className={`sidebar ${collapsed ? "sidebar-collapsed" : ""}`}>
      {sections.map((section) => (
        <div className="sidebar-section" key={section.id}>
          <div className="sidebar-section-title">{section.title}</div>
          <nav className="sidebar-nav">
            {section.items.map((item) => (
              <NavLink
                key={item.id}
                to={item.path}
                className={({ isActive }) =>
                  "sidebar-link" + (isActive ? " sidebar-link-active" : "")
                }
                end={item.path === "/"} // 根路径避免子路由全部高亮
              >
                {item.label}
              </NavLink>
            ))}
          </nav>
        </div>
      ))}
    </aside>
  );
};

export default Sidebar;
