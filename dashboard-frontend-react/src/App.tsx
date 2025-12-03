import React, { Suspense } from "react";
import {
  BrowserRouter,
  Routes,
  Route,
  NavLink,
} from "react-router-dom";
import DashboardHome from "./pages/DashboardHome";
import WorkflowListPage from "./pages/workflows/WorkflowListPage";
import WorkflowDetailPage from "./pages/workflows/WorkflowDetailPage";

// “前端插件”式：按需动态加载 Workflow Designer
const WorkflowDesignerLazy = React.lazy(
  () => import("./pages/WorkflowManagement"),
);

const App: React.FC = () => {
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
          <nav style={{ display: "flex", gap: 16 }}>
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

            <NavLink
              to="/workflows"
              style={({ isActive }) => ({
                textDecoration: "none",
                color: isActive ? "#2563eb" : "#4b5563",
                fontWeight: isActive ? 600 : 400,
              })}
            >
              Workflows
            </NavLink>

            <NavLink
              to="/workflow-designer"
              style={({ isActive }) => ({
                textDecoration: "none",
                color: isActive ? "#2563eb" : "#4b5563",
                fontWeight: isActive ? 600 : 400,
              })}
            >
              Designer
            </NavLink>
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

            {/* 工作流列表页 */}
            <Route path="/workflows" element={<WorkflowListPage />} />

            {/* 工作流详情页 */}
            <Route
              path="/workflows/:id"
              element={<WorkflowDetailPage />}
            />

            {/* LogicFlow Designer 作为“前端插件”懒加载 */}
            <Route
              path="/workflow-designer"
              element={
                <div
                  style={{
                    width: "100%",
                    height: "calc(100vh - 56px)",
                    background: "#fff",
                  }}
                >
                  <Suspense fallback={<div style={{ padding: 24 }}>加载工作流设计器...</div>}>
                    <WorkflowDesignerLazy />
                  </Suspense>
                </div>
              }
            />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  );
};

export default App;
