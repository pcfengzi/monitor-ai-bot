// src/layout/DashboardLayout.tsx
import React, { useState } from "react";
import { Outlet, useLocation } from "react-router-dom";
import Topbar from "./Topbar";
import Sidebar from "./Sidebar";
import "../App.css";

const DashboardLayout: React.FC = () => {
  const [collapsed, setCollapsed] = useState(false);
  const location = useLocation();

  const toggleSidebar = () => setCollapsed((prev) => !prev);

  const segments = location.pathname.split("/").filter(Boolean);
  const current = segments.length === 0 ? "overview" : segments[segments.length - 1];

  return (
    <div className="dashboard-root">
      <Topbar onToggleSidebar={toggleSidebar} />

      <div className="dashboard-body">
        <Sidebar collapsed={collapsed} />

        <main className="dashboard-content">
          <div className="dashboard-page-header">
            <div className="dashboard-breadcrumb">控制台 / {current}</div>
          </div>
          <div className="dashboard-page-body">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  );
};

export default DashboardLayout;
