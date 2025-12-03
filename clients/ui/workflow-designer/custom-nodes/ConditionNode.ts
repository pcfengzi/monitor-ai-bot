// clients/ui/workflow-designer/custom-nodes/ConditionNode.ts
import {
  h,
  LogicFlow,
  PolygonNode,
  PolygonNodeModel,
  NodeConfig,
} from "@logicflow/core";

export interface ConditionNodeProperties {
  expression?: string; // 例如 "status == 200"
  description?: string;
}

export const CONDITION_NODE_TYPE = "wf-condition";

/**
 * Model：菱形条件节点
 */
export class ConditionNodeModel extends PolygonNodeModel {
  properties: ConditionNodeProperties;

  constructor(data: NodeConfig, lf: LogicFlow) {
    // 先构造一个基本菱形形状
    const width = data.width ?? 120;
    const height = data.height ?? 60;

    const points = [
      [0, -height / 2],
      [width / 2, 0],
      [0, height / 2],
      [-width / 2, 0],
    ];

    super(
      {
        ...data,
        points,
      },
      lf
    );

    const props = (data.properties || {}) as ConditionNodeProperties;
    this.properties = {
      expression: props.expression ?? "status == 200",
      description: props.description ?? "",
    };

    if (!data.text || !data.text.value) {
      this.setText(this.properties.expression);
    }
  }

  getNodeStyle() {
    const style = super.getNodeStyle();

    style.stroke = "#f97316"; // 橙色
    style.fill = "#fff7ed"; // 浅橙色
    style.strokeWidth = 1.4;

    return style;
  }

  getTextStyle() {
    const style = super.getTextStyle();

    style.fontSize = 12;
    style.color = "#7c2d12";
    style.fontWeight = 500;

    return style;
  }
}

/**
 * View：自定义渲染（可以加一个小问号图标）
 */
export class ConditionNodeView extends PolygonNode {
  getShape() {
    const { model } = this.props;
    const style = model.getNodeStyle();
    const points = model.points;
    const { x, y } = model;

    // 把 points 转成字符串
    const pointsStr = points
      .map((item) => `${item.x},${item.y}`)
      .join(" ");

    const props = (model.properties || {}) as ConditionNodeProperties;
    const text = props.expression ?? (model.text?.value as string) ?? "条件";

    return h("g", {}, [
      // 菱形本体
      h("polygon", {
        points: pointsStr,
        stroke: style.stroke,
        strokeWidth: style.strokeWidth,
        fill: style.fill,
      }),
      // 中间文本
      h(
        "text",
        {
          x,
          y,
          fill: "#7c2d12",
          fontSize: 11,
          textAnchor: "middle",
          dominantBaseline: "middle",
        },
        text
      ),
    ]);
  }
}

/**
 * 注册函数
 */
export function registerConditionNode(lf: LogicFlow) {
  lf.register({
    type: CONDITION_NODE_TYPE,
    model: ConditionNodeModel,
    view: ConditionNodeView,
  });
}

export const ConditionNode = {
  type: CONDITION_NODE_TYPE,
  view: ConditionNodeView,
  model: ConditionNodeModel,
};