// clients/ui/hooks/useMetrics.ts
import { useEffect, useState } from "react";

export interface Metric {
  time: string;
  plugin: string;
  name: string;
  value: number;
  labels: Record<string, string>;
}

const DEFAULT_API_BASE = "http://127.0.0.1:3001";

export function useMetrics(
  params: { name?: string; plugin?: string; limit?: number } = {},
  apiBase?: string
) {
  const [data, setData] = useState<Metric[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const base = apiBase || DEFAULT_API_BASE;
    const usp = new URLSearchParams();
    if (params.name) usp.set("name", params.name);
    if (params.plugin) usp.set("plugin", params.plugin);
    if (params.limit) usp.set("limit", String(params.limit));

    const url = `${base}/metrics?${usp.toString()}`;

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
  }, [params.name, params.plugin, params.limit, apiBase]);

  return { data, loading, error };
}
