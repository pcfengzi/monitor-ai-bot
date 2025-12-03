// src/App.tsx
import React, { useEffect, useState } from "react";
import {
  BrowserRouter,
  Routes,
  Route,
  NavLink,
} from "react-router-dom";
import MetricChart, { type MetricPoint } from "./components/MetricChart";
import WorkflowDesignerPage from "./pages/WorkflowManagement";

// ---- 公共类型 & 常量 ----

type Metric = {
  time: string;
  plugin: string;
  name: string;
  value: number;
};

const API_BASE = import.meta.env.VITE_API_BASE || "http://127.0.0.1:3001";

// ---- Dashboard 首页 ----

const DashboardHome: React.FC = () => {
  const [metrics, setMetrics] = useState<Metric[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchMetrics = async () => {
    try {
      setLoading(true);
      setError(null);
      const resp = await fetch(`${API_BASE}/metrics`);
      if (!resp.ok) {
        throw new Error(`HTTP ${resp.status}`);
      }
      const data = (await resp.json()) as Metric[];
      setMetrics(data);
    } catch (e: any) {
      console.error(e);
      setError(e.message || "加载失败");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchMetrics();
    const timer = setInterval(fetchMetrics, 5000);
    return () => clearInterval(timer);
  }, []);

  // 指标转换成图表序列
  const toSeries = (plugin: string, name: string): MetricPoint[] =>
    metrics
      .filter((m) => m.plugin === plugin && m.name === name)
      .sort(
        (a, b) =>
          new Date(a.time).getTime() - new Date(b.time).getTime(),
      )
      .map((m) => ({ time: m.time, value: m.value }));

  const cpuSeries = toSeries("cpu-monitor", "cpu_usage");
  const apiFlowSuccess = toSeries("api-monitor", "api_flow_success");
  const apiFlowDuration = toSeries("api-monitor", "api_flow_duration_ms");

  return (
    <div
      style={{
        padding: 24,
        fontFamily:
          "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
      }}
    >
      <h1 style={{ marginBottom: 8 }}>Monitor AI Bot Dashboard</h1>
      <p style={{ color: "#666", marginBottom: 24 }}>
        实时监控：CPU 使用率 &amp; API 流程健康度（由 api-monitor 插件上报）
      </p>

      {error && (
        <div style={{ color: "red", marginBottom: 16 }}>
          加载失败：{error}
        </div>
      )}

      {loading && (
        <div style={{ marginBottom: 16 }}>加载中...</div>
      )}

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr",
          gap: 24,
        }}
      >
        {/* CPU 使用率 */}
        <MetricChart
          title="CPU 使用率（cpu-monitor / cpu_usage）"
          seriesName="cpu_usage %"
          data={cpuSeries}
          yAxisName="%"
          min={0}
          max={100}
        />

        {/* API 流程成功率（0/1） */}
        <MetricChart
          title="API 流程成功（api-monitor / api_flow_success）"
          seriesName="success(1)/fail(0)"
          data={apiFlowSuccess}
          yAxisName="success"
          min={0}
          max={1.2}
        />

        {/* API 流程耗时 */}
        <MetricChart
          title="API 流程耗时（api-monitor / api_flow_duration_ms）"
          seriesName="duration(ms)"
          data={apiFlowDuration}
          yAxisName="ms"
        />
      </div>
    </div>
  );
};

// ---- 顶层 App：路由 + 导航 ----

const App: React.FC = () => {
  return (
    <BrowserRouter>
      <div
        style={{
          minHeight: "100vh",
          display: "flex",
          flexDirection: "column",
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
              to="/workflow-designer"
              style={({ isActive }) => ({
                textDecoration: "none",
                color: isActive ? "#2563eb" : "#4b5563",
                fontWeight: isActive ? 600 : 400,
              })}
            >
              Workflow Designer
            </NavLink>
          </nav>
        </header>

        {/* 页面内容区域 */}
        <main style={{ flex: 1, minHeight: 0 }}>
          <Routes>
            <Route path="/" element={<DashboardHome />} />
            <Route
              path="/workflow-designer"
              element={
                <div style={{ width: "100%", height: "calc(100vh - 56px)" }}>
                  <WorkflowDesignerPage />
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
