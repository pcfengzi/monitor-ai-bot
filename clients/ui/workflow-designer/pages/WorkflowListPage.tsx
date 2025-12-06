import React, { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useWorkflowStore } from "../store/workflowStore";


const WorkflowListPage: React.FC = () => {
  const navigate = useNavigate();
  const { workflows, loading, error, loadWorkflows } = useWorkflowStore(
    (s) => ({
      workflows: s.workflows,
      loading: s.loading,
      error: s.error,
      loadWorkflows: s.loadWorkflows,
    }),
  );

  useEffect(() => {
    loadWorkflows();
  }, [loadWorkflows]);

  return (
    <div style={{ padding: 24 }}>
      <h2 style={{ marginBottom: 16 }}>工作流列表</h2>
      <p style={{ color: "#666", marginBottom: 16 }}>
        这里展示 workflow-engine 插件中存储的所有工作流定义。
      </p>

      {error && (
        <div style={{ color: "red", marginBottom: 12 }}>{error}</div>
      )}
      {loading && (
        <div style={{ marginBottom: 12 }}>加载中...</div>
      )}

      <table
        style={{
          width: "100%",
          borderCollapse: "collapse",
          background: "#fff",
        }}
      >
        {/* 表头略，与之前相同 */}
        {/* ... */}
        <thead>
          <tr
            style={{
              background: "#f3f4f6",
              textAlign: "left",
            }}
          >
            <th style={{ padding: "8px 12px" }}>ID</th>
            <th style={{ padding: "8px 12px" }}>名称</th>
            <th style={{ padding: "8px 12px" }}>状态</th>
            <th style={{ padding: "8px 12px" }}>更新时间</th>
            <th style={{ padding: "8px 12px" }}>操作</th>
          </tr>
        </thead>
        <tbody>
          {workflows.map((wf) => (
            <tr key={wf.id} style={{ borderTop: "1px solid #e5e7eb" }}>
              <td style={{ padding: "8px 12px", fontFamily: "monospace" }}>
                {wf.id}
              </td>
              <td style={{ padding: "8px 12px" }}>{wf.name}</td>
              <td style={{ padding: "8px 12px" }}>
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
              </td>
              <td style={{ padding: "8px 12px", fontSize: 12, color: "#6b7280" }}>
                {wf.updatedAt
                  ? new Date(wf.updatedAt).toLocaleString()
                  : "-"}
              </td>
              <td style={{ padding: "8px 12px" }}>
                <button
                  style={{
                    padding: "4px 10px",
                    fontSize: 13,
                    borderRadius: 6,
                    border: "1px solid #d1d5db",
                    background: "#fff",
                    cursor: "pointer",
                  }}
                  onClick={() => navigate(`/workflows/${wf.id}`)}
                >
                  详情 / 设计
                </button>
              </td>
            </tr>
          ))}
          {workflows.length === 0 && !loading && (
            <tr>
              <td
                colSpan={5}
                style={{ padding: 12, textAlign: "center", color: "#6b7280" }}
              >
                暂无工作流，可在 Designer 中创建并保存。
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
};

export default WorkflowListPage;
