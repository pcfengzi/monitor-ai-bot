// dashboard-frontend/src/pages/WorkflowDesignerPage.tsx
import React, { useEffect, useState } from "react";
import type { GraphConfigData } from "@logicflow/core";
import { WorkflowDesigner } from "../../../clients/ui/workflow-designer/WorkflowDesigner";

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

const API_BASE = import.meta.env.VITE_API_BASE ?? "http://127.0.0.1:3001";

export const WorkflowDesignerPage: React.FC = () => {
  const [defs, setDefs] = useState<WorkflowDefinition[]>([]);
  const [currentId, setCurrentId] = useState<number | null>(null);
  const [currentName, setCurrentName] = useState<string>("");
  const [currentDesc, setCurrentDesc] = useState<string>("");
  const [graph, setGraph] = useState<GraphConfigData | undefined>();
  const [saving, setSaving] = useState(false);
  const [running, setRunning] = useState(false);
  const [instance, setInstance] = useState<InstanceDetail | null>(null);
  const [aiPrompt, setAiPrompt] = useState<string>("用户登录 → 获取资料 → 发送欢迎通知");

  // 1. 加载 workflow 列表
  const loadDefinitions = async () => {
    const res = await fetch(`${API_BASE}/plugin-api/workflow-engine/workflow/definitions`);
    const data = await res.json();
    setDefs(data);
  };

  useEffect(() => {
    loadDefinitions();
  }, []);

  // 选中一个 definition
  const handleSelectDef = (def: WorkflowDefinition) => {
    setCurrentId(def.id);
    setCurrentName(def.name);
    setCurrentDesc(def.description ?? "");
    setGraph(def.lf_json);
    setInstance(null);
  };

  // 新建
  const handleNew = () => {
    setCurrentId(null);
    setCurrentName("未命名工作流");
    setCurrentDesc("");
    setGraph({ nodes: [], edges: [] } as any);
    setInstance(null);
  };

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
      const res = await fetch(`${API_BASE}/plugin-api/workflow-engine/workflow/definitions`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });
      const data = (await res.json()) as { id: number };
      setCurrentId(data.id);
      await loadDefinitions();
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
      const res = await fetch(`${API_BASE}/plugin-api/workflow-engine/workflow/run`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ workflow_id: currentId }),
      });
      const data = (await res.json()) as RunResponse;
      // 同步执行，直接查实例详情
      const res2 = await fetch(
        `${API_BASE}/plugin-api/workflow-engine/workflow/instances/${data.instance_id}`
      );
      const inst = (await res2.json()) as InstanceDetail;
      // steps 可能是对象，要转成数组
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
        `${API_BASE}/plugin-api/workflow-engine/workflow/ai-generate`,
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

  return (
    <div style={{ display: "flex", height: "100vh", overflow: "hidden" }}>
      {/* 左：流程列表 */}
      <div
        style={{
          width: 260,
          borderRight: "1px solid #e5e7eb",
          padding: 12,
          background: "#fff",
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <h3 style={{ margin: 0 }}>工作流列表</h3>
          <button onClick={handleNew}>新建</button>
        </div>
        <div style={{ marginTop: 12 }}>
          {defs.map((d) => (
            <div
              key={d.id}
              onClick={() => handleSelectDef(d)}
              style={{
                padding: "6px 8px",
                marginBottom: 6,
                borderRadius: 4,
                cursor: "pointer",
                background: currentId === d.id ? "#e5f0ff" : "#f8f9fa",
              }}
            >
              <div>{d.name}</div>
              {d.description && (
                <div style={{ fontSize: 12, color: "#6b7280" }}>{d.description}</div>
              )}
            </div>
          ))}
          {defs.length === 0 && <div>暂无工作流，点击上方「新建」</div>}
        </div>
      </div>

      {/* 中：Designer + 顶部工具栏 */}
      <div style={{ flex: 1, display: "flex", flexDirection: "column" }}>
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
          <input
            value={currentName}
            onChange={(e) => setCurrentName(e.target.value)}
            placeholder="工作流名称"
            style={{ minWidth: 220 }}
          />
          <input
            value={currentDesc}
            onChange={(e) => setCurrentDesc(e.target.value)}
            placeholder="描述"
            style={{ flex: 1 }}
          />
          <button onClick={handleSave} disabled={saving}>
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

        <div style={{ flex: 1 }}>
          <WorkflowDesigner
            value={graph}
            onChange={(data) => {
              setGraph(data);
            }}
          />
        </div>
      </div>

      {/* 右：执行结果 */}
      <div
        style={{
          width: 280,
          borderLeft: "1px solid #e5e7eb",
          padding: 12,
          background: "#fff",
        }}
      >
        <h3 style={{ marginTop: 0 }}>执行结果</h3>
        {!instance && <div>点击「执行一次」查看结果</div>}
        {instance && (
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
        )}
      </div>
    </div>
  );
};

export default WorkflowDesignerPage;
