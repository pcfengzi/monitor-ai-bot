// clients/ui/workflow-designer/NodePanel.tsx
import React from "react";
import { CUSTOM_NODE_TYPES } from "./custom-nodes";

export const NodePanel: React.FC = () => {
  return (
    <div>
      <h4>节点列表</h4>

      {CUSTOM_NODE_TYPES.map((type) => (
        <div
          key={type.type}
          draggable
          onDragStart={(e) => e.dataTransfer.setData("node-type", type.type)}
          style={{
            padding: "8px 12px",
            background: "#f8f9fa",
            border: "1px solid #e5e7eb",
            borderRadius: 6,
            marginTop: 8,
            cursor: "grab",
          }}
        >
          {type.label}
        </div>
      ))}
    </div>
  );
};
