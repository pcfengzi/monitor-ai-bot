// clients/ui/workflow-designer/custom-nodes/ApiNode.ts
import {
  h,
  LogicFlow,
  RectNode,
  RectNodeModel,
  NodeConfig,
} from "@logicflow/core";

// 自定义 API 节点的数据结构（在 properties 中存）
export interface ApiNodeProperties {
  method?: string;
  url?: string;
  timeout_ms?: number;
  description?: string;
}

export const API_NODE_TYPE = "wf-api";

/**
 * Model：控制数据 & 样式
 */
export class ApiNodeModel extends RectNodeModel {
  properties: ApiNodeProperties;

  constructor(data: NodeConfig, lf: LogicFlow) {
    super(
      {
        // 默认尺寸 & 样式
        ...data,
        width: data.width ?? 140,
        height: data.height ?? 52,
      },
      lf
    );

    const props = (data.properties || {}) as ApiNodeProperties;

    this.properties = {
      method: props.method ?? "GET",
      url: props.url ?? "/api/path",
      timeout_ms: props.timeout_ms ?? 3000,
      description: props.description ?? "",
    };

    // 默认文本显示
    const label =
      `${this.properties.method?.toUpperCase()} ` +
      (data.text?.value || this.properties.url || "API");

    if (!data.text) {
      this.setText(label);
    } else if (!data.text.value) {
      this.setText(label);
    }
  }

  // 统一节点样式
  getNodeStyle() {
    const style = super.getNodeStyle();

    style.stroke = "#2563eb"; // 边框色（蓝）
    style.fill = "#eff6ff"; // 背景色（淡蓝）
    style.strokeWidth = 1.4;
    style.radius = 6;

    return style;
  }

  // 文本样式
  getTextStyle() {
    const style = super.getTextStyle();

    style.fontSize = 12;
    style.color = "#111827";
    style.fontWeight = 500;

    return style;
  }
}

/**
 * View：控制如何渲染 SVG
 */
export class ApiNodeView extends RectNode {
  getShape() {
    const { model } = this.props;
    const { x, y, width, height } = model;
    const props = (model.properties || {}) as ApiNodeProperties;

    const method = (props.method ?? "GET").toUpperCase();
    const url = props.url ?? "";

    const rectRadius = 6;

    return h("g", {}, [
      // 主体矩形
      h("rect", {
        x: x - width / 2,
        y: y - height / 2,
        rx: rectRadius,
        ry: rectRadius,
        width,
        height,
        stroke: "#2563eb",
        strokeWidth: 1.4,
        fill: "#eff6ff",
      }),
      // 左侧 method 标签
      h("rect", {
        x: x - width / 2,
        y: y - height / 2,
        width: 40,
        height,
        rx: rectRadius,
        ry: rectRadius,
        fill: "#2563eb",
      }),
      h(
        "text",
        {
          x: x - width / 2 + 20,
          y: y,
          fill: "#ffffff",
          fontSize: 11,
          fontWeight: 600,
          textAnchor: "middle",
          dominantBaseline: "middle",
        },
        method
      ),
      // 右侧 URL 文本
      h(
        "text",
        {
          x: x - width / 2 + 50,
          y,
          fill: "#111827",
          fontSize: 11,
          textAnchor: "start",
          dominantBaseline: "middle",
        },
        url || (model.text?.value as string) || "API"
      ),
    ]);
  }
}

/**
 * 注册函数：在 Canvas 初始化时调用
 */
export function registerApiNode(lf: LogicFlow) {
  lf.register({
    type: API_NODE_TYPE,
    view: ApiNodeView,
    model: ApiNodeModel,
  });
}

/**
 * 方式一：导出一个配置对象，供 lf.register(ApiNode) 使用
 */
export const ApiNode = {
  type: API_NODE_TYPE,
  view: ApiNodeView,
  model: ApiNodeModel,
};
