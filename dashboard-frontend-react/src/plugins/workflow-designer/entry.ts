// src/plugins/workflow-designer/entry.ts

import { registerPlugin } from "../plugin-registry";
import type { FrontendPlugin } from "../types";

// 从 clients/ui/workflow-designer 正确导入组件名：WorkflowDesignerPage
import { WorkflowDesignerPage } from "../../../../clients/ui/workflow-designer";

const workflowDesignerPlugin: FrontendPlugin = {
  id: "workflow-designer",
  title: "工作流设计器",
  route: "/workflow-designer",
  component: WorkflowDesignerPage, // ← 使用正确的导出名
  category: "workflow",
  order: 10,
};

registerPlugin(workflowDesignerPlugin);
