// clients/ui/workflow-designer/Canvas.tsx
import React, { useEffect, useRef } from "react";
import LogicFlow, { GraphConfigData } from "@logicflow/core";
import { Control, MiniMap } from "@logicflow/extension";

import {
  registerCustomNodes,
  CUSTOM_NODE_TYPES,
} from "./custom-nodes";

import "@logicflow/core/dist/index.css";
import "@logicflow/extension/lib/style/index.css";

interface Props {
  value?: GraphConfigData;
  onChange?: (data: GraphConfigData) => void;
  onSelectNode?: (node: any) => void;
}

export const Canvas: React.FC<Props> = ({
  value,
  onChange,
  onSelectNode,
}) => {
  const ref = useRef<HTMLDivElement>(null);
  const lfRef = useRef<LogicFlow | null>(null);

  useEffect(() => {
    if (!ref.current) return;

    // 初始化 LF
    const lf = new LogicFlow({
      container: ref.current,
      grid: true,
      plugins: [Control, MiniMap],
    });

    registerCustomNodes(lf);

    lf.render(value ?? { nodes: [], edges: [] });

    // 点击事件
    lf.on("node:click", ({ data }) => {
      onSelectNode?.(data);
    });

    // Graph 变化
    const fireChange = () => {
      onChange?.(lf.getGraphData());
    };

    ["node:add", "node:delete", "node:move", "edge:add", "edge:delete"].forEach((ev) =>
      lf.on(ev, fireChange)
    );

    lfRef.current = lf;

    return () => {
      lf.destroy();
    };
  }, []);

  return <div ref={ref} style={{ width: "100%", height: "100%" }} />;
};
