import { create } from "zustand";

export type WorkflowSummary = {
  id: string;
  name: string;
  status: "enabled" | "disabled";
  description?: string;
  updatedAt?: string;
};

export type WorkflowDefinition = {
  id: string;
  name: string;
  description?: string;
  enabled: boolean;
  lfJson: any;
  updatedAt?: string;
};

const API_BASE =
  import.meta.env.VITE_API_BASE || "http://127.0.0.1:3001";

type WorkflowState = {
  workflows: WorkflowSummary[];
  current?: WorkflowDefinition;
  loading: boolean;
  error?: string | null;
  loadWorkflows: () => Promise<void>;
  loadWorkflowById: (id: string) => Promise<void>;
  setCurrentLfJson: (lfJson: any) => void;
};

export const useWorkflowStore = create<WorkflowState>((set, get) => ({
  workflows: [],
  current: undefined,
  loading: false,
  error: null,

  async loadWorkflows() {
    try {
      set({ loading: true, error: null });
      const resp = await fetch(
        `${API_BASE}/plugin-api/workflow-engine/workflows`,
      );
      if (!resp.ok) {
        throw new Error(`HTTP ${resp.status}`);
      }
      const data = (await resp.json()) as any[];
      const list: WorkflowSummary[] = data.map((item) => ({
        id: item.id,
        name: item.name,
        status: item.enabled ? "enabled" : "disabled",
        description: item.description ?? undefined,
        updatedAt: item.updated_at,
      }));
      set({ workflows: list });
    } catch (e: any) {
      console.error(e);
      set({ error: e.message || "加载工作流列表失败" });
    } finally {
      set({ loading: false });
    }
  },

  async loadWorkflowById(id: string) {
    try {
      set({ loading: true, error: null });
      const resp = await fetch(
        `${API_BASE}/plugin-api/workflow-engine/workflows/${id}`,
      );
      if (!resp.ok) {
        throw new Error(`HTTP ${resp.status}`);
      }
      const data = (await resp.json()) as any;
      const def: WorkflowDefinition = {
        id: data.id,
        name: data.name,
        description: data.description ?? undefined,
        enabled: !!data.enabled,
        lfJson: data.lf_json,
        updatedAt: data.updated_at,
      };
      set({ current: def });
    } catch (e: any) {
      console.error(e);
      set({ error: e.message || "加载工作流详情失败" });
    } finally {
      set({ loading: false });
    }
  },

  setCurrentLfJson(lfJson: any) {
    const cur = get().current;
    if (!cur) return;
    set({ current: { ...cur, lfJson } });
  },
}));
