// src/components/MetricChart.tsx
import React from "react";
import ReactECharts from "echarts-for-react";

export type MetricPoint = {
  time: string;   // ISO 字符串
  value: number;
};

type MetricChartProps = {
  title: string;
  seriesName?: string;
  data: MetricPoint[];
  yAxisName?: string;
  min?: number;
  max?: number;
};

const MetricChart: React.FC<MetricChartProps> = ({
  title,
  seriesName = "value",
  data,
  yAxisName,
  min,
  max,
}) => {
  const option = {
    title: {
      text: title,
      left: "center",
    },
    tooltip: {
      trigger: "axis",
    },
    grid: {
      left: 40,
      right: 20,
      bottom: 40,
      top: 60,
    },
    xAxis: {
      type: "time",
      boundaryGap: false,
    },
    yAxis: {
      type: "value",
      name: yAxisName,
      min,
      max,
    },
    series: [
      {
        name: seriesName,
        type: "line",
        smooth: true,
        showSymbol: false,
        data: data.map((p) => [p.time, p.value]),
      },
    ],
  };

  return <ReactECharts option={option} style={{ height: 300, width: "100%" }} />;
};

export default MetricChart;
