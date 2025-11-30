// clients/ui/workflow-designer/custom-nodes/ConditionNode.ts
export const ConditionNode = {
  type: "condition-node",
  view: {
    render() {
      return `<div style="padding:6px 12px;border-radius:12px;background:#4caf50;color:#fff;">开始</div>`;
    },
  },
  model: {
    default: {
      width: 80,
      height: 32,
    },
  },
};
