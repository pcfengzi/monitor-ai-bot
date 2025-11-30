// clients/ui/workflow-designer/custom-nodes/StartNode.ts
export const StartNode = {
  type: "start-node",
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
