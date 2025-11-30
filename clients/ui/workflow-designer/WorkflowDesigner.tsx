// clients/ui/workflow-designer/WorkflowDesigner.tsx
import React, { useState } from "react";
import { GraphConfigData } from "@logicflow/core";

import { Canvas } from "./Canvas";
import { NodePanel } from "./NodePanel";
import { PropertyPanel } from "./PropertyPanel";

export interface WorkflowDesignerProps {
  value?: GraphConfigData;
  onChange?: (data: GraphConfigData) => void;
}

export const WorkflowDesigner: React.FC<WorkflowDesignerProps> = ({
  value,
  onChange,
}) => {
  const [selectedNode, setSelectedNode] = useState<any>(null);
  const [graphData, setGraphData] = useState<GraphConfigData | undefined>(
    value
  );

  return (
    <div
      style={{
        display: "flex",
        height: "100%",
        width: "100%",
        background: "#f5f6f8",
        overflow: "hidden",
      }}
    >
      {/* 左侧：节点面板 */}
      <div
        style={{
          width: 200,
          borderRight: "1px solid #e5e7eb",
          background: "#fff",
          padding: 12,
        }}
      >
        <NodePanel />
      </div>

      {/* 中间：画布 */}
      <div style={{ flex: 1 }}>
        <Canvas
          value={graphData}
          onSelectNode={(node) => setSelectedNode(node)}
          onChange={(data) => {
            setGraphData(data);
            onChange?.(data);
          }}
        />
      </div>

      {/* 右侧：属性面板 */}
      <div
        style={{
          width: 260,
          borderLeft: "1px solid #e5e7eb",
          background: "#fff",
          padding: 12,
        }}
      >
        <PropertyPanel node={selectedNode} />
      </div>
    </div>
  );
};
