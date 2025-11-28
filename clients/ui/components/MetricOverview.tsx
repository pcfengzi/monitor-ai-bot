// clients/ui/components/MetricOverview.tsx
import React from "react";
import { Metric } from "../hooks/useMetrics";

export interface MetricOverviewProps {
  cpu?: Metric | null;
  apiFlow?: Metric | null;
}

export const MetricOverview: React.FC<MetricOverviewProps> = ({
  cpu,
  apiFlow,
}) => {
  return (
    <div style={{ display: "flex", gap: 16, marginBottom: 24 }}>
      <StatusCard
        title="CPU 当前使用率"
        value={cpu ? `${cpu.value.toFixed(1)} %` : "--"}
        time={cpu?.time}
      />
      <StatusCard
        title="API 流程健康"
        value={
          apiFlow
            ? apiFlow.value >= 0.5
              ? "✅ 正常"
              : "❌ 异常"
            : "--"
        }
        time={apiFlow?.time}
      />
    </div>
  );
};

const StatusCard: React.FC<{ title: string; value: string; time?: string }> = ({
  title,
  value,
  time,
}) => (
  <div
    style={{
      flex: 1,
      borderRadius: 12,
      border: "1px solid #eee",
      padding: 16,
      boxShadow: "0 2px 6px rgba(0,0,0,0.04)",
      background: "#fff",
    }}
  >
    <div style={{ fontSize: 14, color: "#666", marginBottom: 8 }}>{title}</div>
    <div style={{ fontSize: 24, fontWeight: 600, marginBottom: 4 }}>{value}</div>
    {time && (
      <div style={{ fontSize: 12, color: "#999" }}>
        更新时间：{new Date(time).toLocaleString()}
      </div>
    )}
  </div>
);
