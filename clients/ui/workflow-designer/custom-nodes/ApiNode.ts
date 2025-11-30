// clients/ui/workflow-designer/custom-nodes/ApiNode.ts
export const ApiNode = {
  type: "api-node",
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
