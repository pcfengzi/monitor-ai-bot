import { useEffect, useState } from "react";

type LogLevel = "Debug" | "Info" | "Warn" | "Error";

interface LogEvent {
  time: string;
  level: LogLevel;
  plugin?: string | null;
  message: string;
}

interface Metric {
  time: string;
  plugin: string;
  name: string;
  value: number;
}

function App() {
  const [logs, setLogs] = useState<LogEvent[]>([]);
  const [metrics, setMetrics] = useState<Metric[]>([]);

  useEffect(() => {
    const apiBase = "http://127.0.0.1:3001";

    fetch(`${apiBase}/logs`)
      .then((res) => res.json())
      .then(setLogs)
      .catch(console.error);

    fetch(`${apiBase}/metrics`)
      .then((res) => res.json())
      .then(setMetrics)
      .catch(console.error);
  }, []);

  return (
    <div style={{ padding: "16px", fontFamily: "sans-serif" }}>
      <h1>监控 AI 机器人 - 仪表盘</h1>

      <section>
        <h2>最近指标</h2>
        <table border={1} cellPadding={4}>
          <thead>
            <tr>
              <th>时间</th>
              <th>插件</th>
              <th>指标名</th>
              <th>数值</th>
            </tr>
          </thead>
          <tbody>
            {metrics.map((m, idx) => (
              <tr key={idx}>
                <td>{m.time}</td>
                <td>{m.plugin}</td>
                <td>{m.name}</td>
                <td>{m.value}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section style={{ marginTop: "24px" }}>
        <h2>最近日志</h2>
        <ul>
          {logs.map((l, idx) => (
            <li key={idx}>
              [{l.time}] [{l.level}] {l.message}
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}

export default App;
