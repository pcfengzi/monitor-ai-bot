import { create } from "zustand";

export type WorkflowSummary = {
  id: string;
  name: string;
  status: "enabled" | "disabled";
  description?: string;
  updatedAt?: string;
};

// LogicFlow JSON 的占位类型
export type WorkflowDefinition = {
  id: string;
  name: string;
  lfJson: any; // 先随意，后面你接上真正的 LogicFlow JSON
};

type WorkflowState = {
  workflows: WorkflowSummary[];
  current?: WorkflowDefinition;
  setWorkflows: (items: WorkflowSummary[]) => void;
  setCurrent: (wf?: WorkflowDefinition) => void;
};

export const useWorkflowStore = create<WorkflowState>((set) => ({
  workflows: [
    // 初始一些假数据，后面可以从后端加载
    {
      id: "login_flow",
      name: "登录流程检查",
      status: "enabled",
      description: "模拟 API 登录 + 获取用户信息的工作流",
      updatedAt: new Date().toISOString(),
    },
    {
      id: "order_flow",
      name: "下单流程检查",
      status: "disabled",
      description: "检查下单接口链路",
      updatedAt: new Date().toISOString(),
    },
  ],
  current: undefined,
  setWorkflows: (items) => set({ workflows: items }),
  setCurrent: (wf) => set({ current: wf }),
}));
