import React, { Suspense } from "react";
import {
  BrowserRouter,
  Routes,
  Route,
  NavLink,
} from "react-router-dom";
import DashboardHome from "./pages/DashboardHome";
import { getPlugins } from "./plugin-registry";

const App: React.FC = () => {
  const plugins = getPlugins();

  return (
    <BrowserRouter>
      <div
        style={{
          minHeight: "100vh",
          display: "flex",
          flexDirection: "column",
          fontFamily:
            "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
        }}
      >
        {/* 顶部导航栏 */}
        <header
          style={{
            height: 56,
            borderBottom: "1px solid #e5e7eb",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "0 24px",
          }}
        >
          <div style={{ fontWeight: 700 }}>Monitor AI Bot</div>
          <nav style={{ display: "flex", gap: 16, alignItems: 'center' }}>
            <NavLink
              to="/"
              style={({ isActive }) => ({
                textDecoration: "none",
                color: isActive ? "#2563eb" : "#4b5563",
                fontWeight: isActive ? 600 : 400,
              })}
              end
            >
              Dashboard
            </NavLink>

            {/* Dynamically generate navigation from plugins */}
            {plugins.map((plugin) => (
              <NavLink
                key={plugin.id}
                to={plugin.path}
                style={({ isActive }) => ({
                  textDecoration: "none",
                  color: isActive ? "#2563eb" : "#4b5563",
                  fontWeight: isActive ? 600 : 400,
                  display: 'flex',
                  alignItems: 'center',
                  gap: '4px'
                })}
              >
                {plugin.icon}
                {plugin.name}
              </NavLink>
            ))}
          </nav>
        </header>

        {/* 主内容区域 */}
        <main
          style={{
            flex: 1,
            minHeight: 0,
            background: "#f9fafb",
          }}
        >
          <Routes>
            <Route path="/" element={<DashboardHome />} />

            {/* Dynamically generate routes from plugins */}
            {plugins.map((plugin) => (
              <Route
                key={plugin.id}
                path={plugin.path}
                element={
                  <div
                    style={{
                      width: "100%",
                      height: "calc(100vh - 56px)",
                      background: "#fff",
                    }}
                  >
                    <Suspense fallback={<div style={{ padding: 24 }}>加载插件...</div>}>
                      <plugin.component />
                    </Suspense>
                  </div>
                }
              />
            ))}
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  );
};

export default App;
