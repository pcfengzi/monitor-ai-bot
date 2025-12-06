// src/plugins/frontend-registry.ts
import type React from "react";

// 从 clients/ui 引入插件页面
import {
  WorkflowDesignerPage,
} from "../../clients/ui/workflow-designer";

export type FrontendPluginCategory =
  | "monitor"
  | "workflow"
  | "notification"
  | "system"
  | "custom";

export interface FrontendPlugin {
  id: string;
  title: string;
  route: string;
  component: React.ComponentType<any>;
  category: FrontendPluginCategory;
}

export const frontendPlugins: FrontendPlugin[] = [
  {
    id: "workflow-designer",
    title: "工作流设计器",
    route: "/workflow-designer",
    component: WorkflowDesignerPage,
    category: "workflow",
  },
  // 未来更多插件在这里追加
];
