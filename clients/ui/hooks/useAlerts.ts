// clients/ui/hooks/useAlerts.ts
import { useEffect, useState } from "react";

export interface Alert {
  id: number;
  time: string;
  plugin: string;
  metric_name: string;
  severity: "Info" | "Warning" | "Critical";
  title: string;
  message: string;
}

const DEFAULT_API_BASE = "http://127.0.0.1:3001";

export function useAlerts(limit = 20, apiBase?: string) {
  const [data, setData] = useState<Alert[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const base = apiBase || DEFAULT_API_BASE;
    const usp = new URLSearchParams();
    usp.set("limit", String(limit));

    const url = `${base}/alerts?${usp.toString()}`;

    let cancelled = false;
    setLoading(true);
    setError(null);

    fetch(url)
      .then((r) => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        return r.json();
      })
      .then((json) => {
        if (!cancelled) setData(json);
      })
      .catch((e) => {
        if (!cancelled) setError(e.message || "加载失败");
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [limit, apiBase]);

  return { data, loading, error };
}
