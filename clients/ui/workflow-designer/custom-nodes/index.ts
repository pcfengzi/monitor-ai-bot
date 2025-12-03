// clients/ui/workflow-designer/custom-nodes/index.ts
import { StartNode } from "./StartNode";
import { ApiNode } from "./ApiNode";
import { ConditionNode } from "./ConditionNode";

// index.ts
import { API_NODE_TYPE } from "./ApiNode";

export const CUSTOM_NODE_TYPES = [
  { type: "start-node", label: "开始节点" },
  { type: API_NODE_TYPE, label: "API 调用" },
  { type: "condition-node", label: "条件判断" },
];


export const registerCustomNodes = (lf: any) => {
  lf.register(StartNode);
  lf.register(ApiNode);
  lf.register(ConditionNode);
};
