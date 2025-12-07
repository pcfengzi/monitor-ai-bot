// clients/ui/workflow-designer/WorkflowEditor.tsx
import React, { useEffect, useState } from "react";
import type { GraphConfigData } from "@logicflow/core";
import { WorkflowDesigner } from "./WorkflowDesigner";

// Re-defining types here to make the component self-contained
interface WorkflowDefinition {
  id: number;
  name: string;
  description?: string;
  lf_json: GraphConfigData;
}

interface RunResponse {
  instance_id: number;
  status: string;
}

interface InstanceDetail {
  id: number;
  workflow_id: number;
  status: string;
  steps: any[];
  started_at: string;
  finished_at?: string;
  error?: string | null;
}

export interface WorkflowEditorProps {
  apiBaseUrl: string;
  workflowId: number | null;
  // Callback to inform parent that a save occurred, so it can refetch list
  onSaveSuccess: (workflowId: number) => void;
  // When creating a new workflow
  onNew: () => void;
}

export const WorkflowEditor: React.FC<WorkflowEditorProps> = ({
  apiBaseUrl,
  workflowId,
  onSaveSuccess,
  onNew,
}) => {
  const [currentId, setCurrentId] = useState<number | null>(workflowId);
  const [currentName, setCurrentName] = useState<string>("");
  const [currentDesc, setCurrentDesc] = useState<string>("");
  const [graph, setGraph] = useState<GraphConfigData | undefined>();
  
  const [saving, setSaving] = useState(false);
  const [running, setRunning] = useState(false);
  const [instance, setInstance] = useState<InstanceDetail | null>(null);
  const [aiPrompt, setAiPrompt] = useState<string>("用户登录 → 获取资料 → 发送欢迎通知");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // When workflowId prop changes, load the new workflow
    setCurrentId(workflowId);
    if (workflowId) {
      setLoading(true);
      fetch(`${apiBaseUrl}/plugin-api/workflow-engine/definitions/${workflowId}`)
        .then(res => res.json())
        .then((data: WorkflowDefinition) => {
          setCurrentName(data.name);
          setCurrentDesc(data.description ?? "");
          setGraph(data.lf_json);
          setInstance(null);
        })
        .catch(e => {
          console.error(e);
          alert("加载工作流失败");
        })
        .finally(() => setLoading(false));
    } else {
      // It's a new workflow
      setCurrentName("未命名工作流");
      setCurrentDesc("");
      setGraph({ nodes: [], edges: [] } as any);
      setInstance(null);
    }
  }, [workflowId, apiBaseUrl]);

  // 保存
  const handleSave = async () => {
    if (!graph) return;
    setSaving(true);
    try {
      const body = {
        id: currentId,
        name: currentName || "未命名工作流",
        description: currentDesc || null,
        lf_json: graph,
      };
      const res = await fetch(`${apiBaseUrl}/plugin-api/workflow-engine/definitions`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });
      const data = (await res.json()) as { id: number };
      setCurrentId(data.id);
      onSaveSuccess(data.id); // Notify parent
      alert("保存成功");
    } catch (e) {
      console.error(e);
      alert("保存失败，请看控制台");
    } finally {
      setSaving(false);
    }
  };

  // 执行
  const handleRun = async () => {
    if (!currentId) {
      alert("请先保存工作流");
      return;
    }
    setRunning(true);
    setInstance(null);
    try {
      const res = await fetch(`${apiBaseUrl}/plugin-api/workflow-engine/workflow/run`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ workflow_id: currentId }),
      });
      const data = (await res.json()) as RunResponse;
      const res2 = await fetch(
        `${apiBaseUrl}/plugin-api/workflow-engine/workflow/instances/${data.instance_id}`
      );
      const inst = (await res2.json()) as InstanceDetail;
      const stepsArr = Array.isArray(inst.steps) ? inst.steps : [];
      setInstance({ ...inst, steps: stepsArr });
    } catch (e) {
      console.error(e);
      alert("执行失败，请看控制台");
    } finally {
      setRunning(false);
    }
  };

  // AI 生成
  const handleAiGenerate = async () => {
    try {
      const res = await fetch(
        `${apiBaseUrl}/plugin-api/workflow-engine/workflow/ai-generate`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ prompt: aiPrompt }),
        }
      );
      const data = await res.json();
      setGraph(data.lf_json);
    } catch (e) {
      console.error(e);
      alert("AI 生成失败，请看控制台");
    }
  };

  if (loading) {
    return <div>加载中...</div>;
  }

  return (
    <div style={{ display: "flex", height: "100%", flexDirection: 'column' }}>
       {/* 顶部工具栏 */}
      <div
        style={{
          padding: "8px 12px",
          borderBottom: "1px solid #e5e7eb",
          background: "#fff",
          display: "flex",
          alignItems: "center",
          gap: 8,
        }}
      >
        <button onClick={onNew}>新建工作流</button>
        <input
          value={currentName}
          onChange={(e) => setCurrentName(e.target.value)}
          placeholder="工作流名称"
          style={{ minWidth: 220 }}
          disabled={!graph}
        />
        <input
          value={currentDesc}
          onChange={(e) => setCurrentDesc(e.target.value)}
          placeholder="描述"
          style={{ flex: 1 }}
          disabled={!graph}
        />
        <button onClick={handleSave} disabled={saving || !graph}>
          {saving ? "保存中..." : "保存"}
        </button>
        <button onClick={handleRun} disabled={running || !currentId}>
          {running ? "执行中..." : "执行一次"}
        </button>
        <div style={{ marginLeft: "auto", display: "flex", alignItems: "center", gap: 4 }}>
          <input
            style={{ width: 260 }}
            value={aiPrompt}
            onChange={(e) => setAiPrompt(e.target.value)}
            placeholder="描述一下你想要的流程..."
          />
          <button onClick={handleAiGenerate}>AI 生成</button>
        </div>
      </div>
      
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        {/* 主 Designer */}
        <div style={{ flex: 1 }}>
          {graph ? (
            <WorkflowDesigner
              value={graph}
              onChange={(data) => {
                setGraph(data);
              }}
            />
          ) : (
             <div style={{ textAlign: 'center', marginTop: 40, color: '#666' }}>
                请从左侧选择一个工作流，或点击“新建工作流”
             </div>
          )}
        </div>

        {/* 右：执行结果 */}
        {instance && (
          <div
            style={{
              width: 280,
              borderLeft: "1px solid #e5e7eb",
              padding: 12,
              background: "#fff",
              overflowY: 'auto'
            }}
          >
            <h3 style={{ marginTop: 0 }}>执行结果</h3>
            <div style={{ fontSize: 13 }}>
              <div>
                实例 ID：<b>{instance.id}</b>
              </div>
              <div>状态：{instance.status}</div>
              <div>开始时间：{instance.started_at}</div>
              <div>结束时间：{instance.finished_at ?? "-"}</div>
              {instance.error && <div>错误：{instance.error}</div>}
              <div style={{ marginTop: 8, fontWeight: "bold" }}>步骤：</div>
              <ul>
                {instance.steps.map((s: any, idx: number) => (
                  <li key={idx}>
                    [{s.status}] {s.node_type} ({s.node_id})
                  </li>
                ))}
              </ul>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
