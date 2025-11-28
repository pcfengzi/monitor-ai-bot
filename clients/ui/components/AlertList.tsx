// clients/ui/components/AlertList.tsx
import React from "react";
import { Alert } from "../hooks/useAlerts";

export const AlertList: React.FC<{ alerts: Alert[] }> = ({ alerts }) => {
  if (!alerts.length) {
    return <div style={{ color: "#999" }}>暂无告警</div>;
  }

  const color = (sev: Alert["severity"]) =>
    sev === "Critical" ? "#d32f2f" : sev === "Warning" ? "#ed6c02" : "#1976d2";

  return (
    <div style={{ borderRadius: 12, border: "1px solid #eee", overflow: "hidden" }}>
      {alerts.map((a) => (
        <div
          key={a.id}
          style={{
            display: "flex",
            padding: "8px 12px",
            borderBottom: "1px solid #f3f3f3",
            alignItems: "center",
          }}
        >
          <span
            style={{
              width: 8,
              height: 8,
              borderRadius: "50%",
              backgroundColor: color(a.severity),
              marginRight: 8,
            }}
          />
          <div style={{ flex: 1 }}>
            <div style={{ fontSize: 14, fontWeight: 500 }}>{a.title}</div>
            <div style={{ fontSize: 12, color: "#666" }}>{a.message}</div>
            <div style={{ fontSize: 11, color: "#999", marginTop: 2 }}>
              {new Date(a.time).toLocaleString()} · {a.plugin} · {a.metric_name}
            </div>
          </div>
        </div>
      ))}
    </div>
  );
};
