// dashboard-frontend/src/pages/WorkflowDesignerPage.tsx
import React, { useEffect, useState } from "react";
import { WorkflowEditor } from "../WorkflowEditor";

// Only the list item type is needed here
interface WorkflowDefinition {
  id: number;
  name: string;
  description?: string;
}

const API_BASE = import.meta.env.VITE_API_BASE ?? "http://127.0.0.1:3001";

export const WorkflowDesignerPage: React.FC = () => {
  const [defs, setDefs] = useState<WorkflowDefinition[]>([]);
  const [currentId, setCurrentId] = useState<number | null>(null);

  const loadDefinitions = async () => {
    try {
      const res = await fetch(`${API_BASE}/plugin-api/workflow-engine/workflow/definitions`);
      const data = await res.json();
      setDefs(data);
    } catch (e) {
      console.error(e);
      alert("加载工作流列表失败");
    }
  };

  useEffect(() => {
    loadDefinitions();
  }, []);
  
  // When a workflow is saved in the editor, we should refresh the list
  // and make sure the saved workflow is selected.
  const handleSaveSuccess = (savedId: number) => {
    loadDefinitions();
    setCurrentId(savedId);
  }

  const handleNew = () => {
    setCurrentId(null);
  }

  return (
    <div style={{ display: "flex", height: "100vh", overflow: "hidden" }}>
      {/* 左：流程列表 */}
      <div
        style={{
          width: 260,
          borderRight: "1px solid #e5e7eb",
          padding: 12,
          background: "#f8f9fa",
          overflowY: 'auto'
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <h3 style={{ margin: 0 }}>工作流列表</h3>
        </div>
        <div style={{ marginTop: 12 }}>
          {defs.map((d) => (
            <div
              key={d.id}
              onClick={() => setCurrentId(d.id)}
              style={{
                padding: "6px 8px",
                marginBottom: 6,
                borderRadius: 4,
                cursor: "pointer",
                background: currentId === d.id ? "#e5f0ff" : "#fff",
                border: currentId === d.id ? '1px solid #77aaff' : '1px solid #e5e7eb',
              }}
            >
              <div>{d.name}</div>
              {d.description && (
                <div style={{ fontSize: 12, color: "#6b7280" }}>{d.description}</div>
              )}
            </div>
          ))}
          {defs.length === 0 && <div style={{color: '#666', fontSize: 13}}>暂无工作流，点击「新建工作流」开始</div>}
        </div>
      </div>

      {/* 中：Editor */}
      <div style={{ flex: 1, display: "flex", flexDirection: "column" }}>
        <WorkflowEditor
          apiBaseUrl={API_BASE}
          workflowId={currentId}
          onSaveSuccess={handleSaveSuccess}
          onNew={handleNew}
        />
      </div>
    </div>
  );
};

export default WorkflowDesignerPage;
