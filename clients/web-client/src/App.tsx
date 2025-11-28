import React, { useEffect, useState } from "react";
import { MetricOverview } from "../../ui/components/MetricOverview";
import { AlertList } from "../../ui/components/AlertList";
import { useMetrics } from "../../ui/hooks/useMetrics";
import { useAlerts } from "../../ui/hooks/useAlerts";
import type { Metric } from "../../ui/hooks/useMetrics";

const API_BASE = import.meta.env.VITE_API_BASE || "http://127.0.0.1:3001";

const App: React.FC = () => {
  const { data: cpuMetrics } = useMetrics(
    { name: "cpu_usage", plugin: "cpu-monitor", limit: 1 },
    API_BASE
  );
  const { data: apiFlowMetrics } = useMetrics(
    { name: "api_flow_success", plugin: "api-monitor", limit: 1 },
    API_BASE
  );
  const { data: alerts } = useAlerts(20, API_BASE);

  const cpu: Metric | null = cpuMetrics[0] ?? null;
  const apiFlow: Metric | null = apiFlowMetrics[0] ?? null;

  return (
    <div style={{ padding: 24, fontFamily: "system-ui" }}>
      <h1 style={{ marginBottom: 8 }}>Monitor AI Web Client</h1>
      <p style={{ color: "#666", marginBottom: 24 }}>
        面向业务 / 测试的简版监控客户端：关键指标 + 告警。
      </p>

      <MetricOverview cpu={cpu} apiFlow={apiFlow} />

      <h2 style={{ marginBottom: 8 }}>最近告警</h2>
      <AlertList alerts={alerts} />
    </div>
  );
};

export default App;
