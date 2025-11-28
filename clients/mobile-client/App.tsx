import React, { useEffect, useState } from "react";
import {
  SafeAreaView,
  View,
  Text,
  StyleSheet,
  ScrollView,
  RefreshControl,
} from "react-native";

const API_BASE = process.env.EXPO_PUBLIC_API_BASE || "http://127.0.0.1:3001";

type Metric = {
  time: string;
  plugin: string;
  name: string;
  value: number;
  labels: Record<string, string>;
};

type Alert = {
  id: number;
  time: string;
  plugin: string;
  metric_name: string;
  severity: "Info" | "Warning" | "Critical";
  title: string;
  message: string;
};

export default function App() {
  const [cpu, setCpu] = useState<Metric | null>(null);
  const [apiFlow, setApiFlow] = useState<Metric | null>(null);
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [refreshing, setRefreshing] = useState(false);

  const load = async () => {
    setRefreshing(true);
    try {
      const [cpuRes, flowRes, alertsRes] = await Promise.all([
        fetch(`${API_BASE}/metrics?name=cpu_usage&plugin=cpu-monitor&limit=1`),
        fetch(
          `${API_BASE}/metrics?name=api_flow_success&plugin=api-monitor&limit=1`
        ),
        fetch(`${API_BASE}/alerts?limit=20`),
      ]);
      const [cpuData, flowData, alertsData] = await Promise.all([
        cpuRes.json(),
        flowRes.json(),
        alertsRes.json(),
      ]);
      setCpu(cpuData[0] ?? null);
      setApiFlow(flowData[0] ?? null);
      setAlerts(alertsData);
    } catch (e) {
      console.warn(e);
    } finally {
      setRefreshing(false);
    }
  };

  useEffect(() => {
    load();
  }, []);

  return (
    <SafeAreaView style={styles.safe}>
      <ScrollView
        contentContainerStyle={styles.container}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={load} />
        }
      >
        <Text style={styles.title}>Monitor AI Mobile</Text>
        <Text style={styles.subtitle}>随时查看监控状态和告警</Text>

        <View style={styles.row}>
          <StatusCard
            title="CPU 使用率"
            value={cpu ? `${cpu.value.toFixed(1)} %` : "--"}
            time={cpu?.time}
          />
          <StatusCard
            title="API 流程"
            value={
              apiFlow
                ? apiFlow.value >= 0.5
                  ? "✅ 正常"
                  : "❌ 异常"
                : "--"
            }
            time={apiFlow?.time}
          />
        </View>

        <Text style={styles.sectionTitle}>最近告警</Text>
        {alerts.length === 0 ? (
          <Text style={styles.empty}>暂无告警</Text>
        ) : (
          alerts.map((a) => (
            <View key={a.id} style={styles.alertItem}>
              <View style={styles.dot(a.severity)} />
              <View style={{ flex: 1 }}>
                <Text style={styles.alertTitle}>{a.title}</Text>
                <Text style={styles.alertMsg}>{a.message}</Text>
                <Text style={styles.alertMeta}>
                  {new Date(a.time).toLocaleString()} · {a.plugin} ·{" "}
                  {a.metric_name}
                </Text>
              </View>
            </View>
          ))
        )}
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: "#f7f7f7" },
  container: { padding: 16 },
  title: { fontSize: 24, fontWeight: "700", marginBottom: 4 },
  subtitle: { fontSize: 13, color: "#666", marginBottom: 16 },
  row: { flexDirection: "row", gap: 12, marginBottom: 20 },
  card: {
    flex: 1,
    backgroundColor: "#fff",
    borderRadius: 12,
    padding: 12,
    elevation: 2,
  },
  cardTitle: { fontSize: 12, color: "#666", marginBottom: 4 },
  cardValue: { fontSize: 20, fontWeight: "600", marginBottom: 2 },
  cardTime: { fontSize: 10, color: "#999" },
  sectionTitle: { fontSize: 16, fontWeight: "600", marginBottom: 8 },
  empty: { fontSize: 12, color: "#999" },
  alertItem: {
    flexDirection: "row",
    paddingVertical: 8,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: "#e5e5e5",
  },
  dot: (sev: Alert["severity"]) => ({
    width: 8,
    height: 8,
    borderRadius: 4,
    marginRight: 8,
    marginTop: 6,
    backgroundColor:
      sev === "Critical" ? "#d32f2f" : sev === "Warning" ? "#ed6c02" : "#1976d2",
  }),
  alertTitle: { fontSize: 14, fontWeight: "500" },
  alertMsg: { fontSize: 12, color: "#555" },
  alertMeta: { fontSize: 10, color: "#999", marginTop: 2 },
});

const StatusCard: React.FC<{ title: string; value: string; time?: string }> = ({
  title,
  value,
  time,
}) => (
  <View style={styles.card}>
    <Text style={styles.cardTitle}>{title}</Text>
    <Text style={styles.cardValue}>{value}</Text>
    {time && (
      <Text style={styles.cardTime}>
        {new Date(time).toLocaleString()}
      </Text>
    )}
  </View>
);
