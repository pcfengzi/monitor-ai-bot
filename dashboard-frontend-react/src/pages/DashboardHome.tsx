import React, { useEffect, useState } from "react";
import MetricChart, { type MetricPoint } from "../components/MetricChart";

type Metric = {
  time: string;
  plugin: string;
  name: string;
  value: number;
};

const API_BASE =
  import.meta.env.VITE_API_BASE || "http://127.0.0.1:3001";

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

  const toSeries = (plugin: string, name: string): MetricPoint[] =>
    metrics
      .filter((m) => m.plugin === plugin && m.name === name)
      .sort(
        (a, b) =>
          new Date(a.time).getTime() - new Date(b.time).getTime(),
      )
      .map((m) => ({ time: m.time, value: m.value }));

  const cpuSeries = toSeries("cpu-monitor", "cpu_usage");
  const apiFlowSuccess = toSeries(
    "api-monitor",
    "api_flow_success",
  );
  const apiFlowDuration = toSeries(
    "api-monitor",
    "api_flow_duration_ms",
  );

  return (
    <div style={{ padding: 24 }}>
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
        <MetricChart
          title="CPU 使用率（cpu-monitor / cpu_usage）"
          seriesName="cpu_usage %"
          data={cpuSeries}
          yAxisName="%"
          min={0}
          max={100}
        />

        <MetricChart
          title="API 流程成功（api-monitor / api_flow_success）"
          seriesName="success(1)/fail(0)"
          data={apiFlowSuccess}
          yAxisName="success"
          min={0}
          max={1.2}
        />

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

export default DashboardHome;
