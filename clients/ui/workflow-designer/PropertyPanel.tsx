// clients/ui/workflow-designer/PropertyPanel.tsx
import React from "react";

export const PropertyPanel: React.FC<{ node: any }> = ({ node }) => {
  if (!node) return <div>请选择一个节点</div>;

  return (
    <div>
      <h4>节点属性</h4>

      <div style={{ marginTop: 8 }}>
        <div>ID: {node.id}</div>
        <div>类型: {node.type}</div>
      </div>

      {/* TODO: 根据 node.type 渲染不同属性表单 */}
      {node.type === "api-node" && (
        <div>
          <label>API URL：</label>
          <input
            style={{ width: "100%", marginTop: 6 }}
            placeholder="https://api.xxx.com"
          />
        </div>
      )}
    </div>
  );
};
