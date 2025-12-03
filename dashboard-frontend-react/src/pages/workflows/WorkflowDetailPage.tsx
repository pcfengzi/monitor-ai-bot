import React, { useState } from "react";
import { useParams, useNavigate, Link } from "react-router-dom";
import { useWorkflowStore } from "../../store/workflowStore";
import WorkflowDesignerPage from "../WorkflowManagement";

const WorkflowDetailPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const workflows = useWorkflowStore((s) => s.workflows);
  const [showDesigner, setShowDesigner] = useState(false);

  const wf = workflows.find((w) => w.id === id);

  if (!wf) {
    return (
      <div style={{ padding: 24 }}>
        <p>未找到工作流：{id}</p>
        <button
          style={{
            marginTop: 12,
            padding: "4px 10px",
            borderRadius: 6,
            border: "1px solid #d1d5db",
            background: "#fff",
            cursor: "pointer",
          }}
          onClick={() => navigate("/workflows")}
        >
          返回列表
        </button>
      </div>
    );
  }

  const handleSaveLfJson = async (lfJson: any) => {
    try {
      // 1) 本地先更新 store
      setCurrentLfJson(lfJson);

      // 2) 调用后端更新
      const resp = await fetch(
        `${API_BASE}/plugin-api/workflow-engine/workflows/${current.id}`,
        {
          method: "PUT",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            lf_json: lfJson,
          }),
        },
      );

      if (!resp.ok) {
        throw new Error(`HTTP ${resp.status}`);
      }

      // 可选：把返回的最新 updated_at 再写回 store
      // const data = await resp.json();
      // ...

      alert("保存成功");
    } catch (e: any) {
      console.error(e);
      alert(`保存失败：${e.message || e}`);
    }
  };

  return (
    <div style={{ padding: 24 }}>
      {/* 面包屑导航 */}
      <div style={{ marginBottom: 12, fontSize: 14 }}>
        <Link to="/workflows" style={{ color: "#2563eb" }}>
          工作流列表
        </Link>{" "}
        / <span style={{ color: "#4b5563" }}>{wf.name}</span>
      </div>

      <h2 style={{ marginBottom: 8 }}>
        工作流详情：{wf.name}
      </h2>
      <p style={{ color: "#6b7280", marginBottom: 16 }}>
        ID: <code>{wf.id}</code>
      </p>

      <div style={{ marginBottom: 24 }}>
        <p style={{ marginBottom: 4 }}>
          状态：
          <span
            style={{
              padding: "2px 8px",
              borderRadius: 999,
              fontSize: 12,
              backgroundColor:
                wf.status === "enabled" ? "#dcfce7" : "#f3f4f6",
              color:
                wf.status === "enabled" ? "#166534" : "#4b5563",
            }}
          >
            {wf.status === "enabled" ? "启用" : "停用"}
          </span>
        </p>
        {wf.description && (
          <p style={{ marginTop: 4, color: "#4b5563" }}>
            描述：{wf.description}
          </p>
        )}
        {wf.updatedAt && (
          <p
            style={{
              marginTop: 4,
              fontSize: 12,
              color: "#6b7280",
            }}
          >
            更新：{new Date(wf.updatedAt).toLocaleString()}
          </p>
        )}
      </div>

      <button
        style={{
          padding: "6px 14px",
          borderRadius: 8,
          border: "none",
          background: "#2563eb",
          color: "#fff",
          fontSize: 14,
          cursor: "pointer",
        }}
        onClick={() => setShowDesigner(true)}
      >
        打开工作流设计器（LogicFlow）
      </button>

      {/* 简单全屏 Modal 放设计器 */}
      {showDesigner && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            backgroundColor: "rgba(15,23,42,0.65)",
            zIndex: 50,
            display: "flex",
            flexDirection: "column",
          }}
        >
          <div
            style={{
              height: 56,
              background: "#0f172a",
              color: "#e5e7eb",
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              padding: "0 16px",
            }}
          >
            <div>
              工作流设计器：<strong>{wf.name}</strong>
            </div>
            <button
              style={{
                padding: "4px 10px",
                borderRadius: 6,
                border: "1px solid #475569",
                background: "transparent",
                color: "#e5e7eb",
                cursor: "pointer",
              }}
              onClick={() => setShowDesigner(false)}
            >
              关闭
            </button>
          </div>

          <div style={{ flex: 1, background: "#fff" }}>
            {/* 这里直接复用你已有的 WorkflowDesignerPage */}
            <WorkflowDesignerPage
              workflowId={current.id}
              initialData={current.lfJson}
              onSave={handleSaveLfJson}
            />
          </div>
        </div>
      )}
    </div>
  );
};

export default WorkflowDetailPage;
