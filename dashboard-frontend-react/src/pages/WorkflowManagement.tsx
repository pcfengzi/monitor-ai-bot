import React, { useState } from "react";
import { WorkflowDesigner } from "../../clients/ui/workflow-designer/WorkflowDesigner";
import type { GraphConfigData } from "@logicflow/core";

export function WorkflowManagementPage() {
  const [data, setData] = useState<GraphConfigData | undefined>(undefined);

  return (
    <div style={{ height: "100vh", padding: 16, boxSizing: "border-box" }}>
      <h2>工作流设计器（LogicFlow）</h2>
      <div style={{ height: "80vh", marginTop: 12 }}>
        <WorkflowDesigner
          value={data}
          onChange={(d) => {
            setData(d);
            // 这里将来可以调用 /plugin-api/workflow-engine/definitions 保存
            // console.log("workflow changed", d);
          }}
        />
      </div>
    </div>
  );
}
