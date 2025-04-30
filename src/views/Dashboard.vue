<template>
  <v-chart class="h-110 w-full" :option="optionSummary" :autoresize="autoresize" />
  <v-chart class="h-110 w-full" :option="optionError" :autoresize="autoresizeError" />
</template>

<script setup lang="ts">
import { computed } from "vue";
import VChart from "vue-echarts";
import { use } from "echarts/core";
import { LineChart } from "echarts/charts";
import {
  TooltipComponent,
  GridComponent,
  ToolboxComponent,
  DataZoomComponent,
  LegendComponent,
  DatasetComponent,
} from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import type { ComposeOption } from "echarts/core";
import type { LineSeriesOption } from "echarts/charts";
import type {
  TooltipComponentOption,
  GridComponentOption,
  ToolboxComponentOption,
  DataZoomComponentOption,
  LegendComponentOption,
  DatasetComponentOption,
} from "echarts/components";
import { ErrorLog, useLogStore } from "@/stores/errorLog";
import { CallbackDataParams } from "echarts/types/dist/shared";

use([
  TooltipComponent,
  GridComponent,
  ToolboxComponent,
  DataZoomComponent,
  LineChart,
  CanvasRenderer,
  LegendComponent,
  DatasetComponent,
]);

type EChartsOption = ComposeOption<
  | TooltipComponentOption
  | GridComponentOption
  | ToolboxComponentOption
  | DataZoomComponentOption
  | LineSeriesOption
  | LegendComponentOption
  | DatasetComponentOption
>;

// provide(THEME_KEY, "dark");

const autoresize = {
  throttle: 20,
};
const autoresizeError = {
  throttle: 250,
};
const msFormatter = (sec: number) => {
  // 计算分钟数
  const minutes = Math.floor(sec / 60);
  // 计算剩余的秒数
  const seconds = sec % 60;

  let formattedSeconds = seconds.toFixed(3);
  while (formattedSeconds.endsWith("0")) {
    formattedSeconds = formattedSeconds.slice(0, -1);
  }

  if (formattedSeconds.endsWith(".")) {
    formattedSeconds = formattedSeconds.slice(0, -1);
  }

  // 如果分钟数为0，则只返回秒数
  if (minutes === 0) {
    return formattedSeconds + "秒";
  } else if (seconds === 0) {
    // 如果秒数为0，则只返回分钟数
    return minutes + "分种";
  } else {
    // 否则返回分钟数和秒数
    return minutes + "分种" + formattedSeconds + "秒";
  }
};
const logs = useLogStore();

const optionSummary = computed<EChartsOption>(() => ({
  animation: false,
  legend: {},
  dataset: {
    dimensions: ["load", "cost", "iters", "load"],
    source: logs.summary as any,
  },
  tooltip: {
    trigger: "axis",
  },
  xAxis: {
    type: "value",
  },
  yAxis: [
    {
      type: "value",
      name: "耗时",
      axisLabel: {
        formatter: msFormatter,
      },
    },
    {
      type: "value",
      name: "迭代次数",
    },
  ],
  toolbox: {
    feature: {
      dataZoom: {
        yAxisIndex: "none",
      },
    },
  },
  dataZoom: [
    {
      type: "slider",
      filterMode: "none",
    },
    {
      type: "inside",
      filterMode: "none",
    },
  ],
  series: [
    {
      type: "line",
      yAxisIndex: 0,
      name: "耗时",
      encode: {
        x: 0,
        y: 1,
      },
      sampling: "lttb",
      tooltip: {
        valueFormatter(value: any, _dataIndex: any) {
          if (value === null) {
            return "-";
          } else {
            return msFormatter(value);
          }
        },
      },
    },
    {
      type: "line",
      yAxisIndex: 1,
      name: "迭代次数",
      encode: {
        x: 0,
        y: 2,
      },
      sampling: "lttb",
    },
  ],
}));

const formatError = (value: number | null) => {
  if (value) {
    return value.toExponential();
  } else {
    return "0";
  }
};

const optionError = computed<EChartsOption>(() => ({
  animation: false,
  legend: {},
  dataset: {
    dimensions: ["iters", "error_u", "error_phi"],
    source: logs.errors as any[],
  },
  tooltip: {
    trigger: "axis",
    // valueFormatter(value, _dataIndex) {
    //   return formatError(value as number | null);
    // },
    formatter: (params) => {
      params = params as CallbackDataParams[];
      const tooltip_error = params
        .map((param) => {
          // Ensure param.value is defined and properly typed
          const value = param.value as ErrorLog;
          const error = value[param.dimensionNames![param.encode!.y[0]]];
          const name = param.seriesName;
          const color = param.color;
          return `<div class="flex justify-between">
                    <div>
                      <span class="inline-block mr-1 h-2.5 w-2.5 rounded-full" style="background-color: ${color};"></span>
                      <span>${name}</span>
                    </div>
                    <span class="pl-5 fw-900">${formatError(error)}</span>
                  </div>`;
        })
        .join("");

      const data = params[0]?.data as ErrorLog;

      return `<div style="font: 14px / 21px "Microsoft YaHei;">
        load    <span class="fw-900">${data.load}</span>
        ${tooltip_error}
      </div>`;
    },
  },
  xAxis: {
    type: "value",
  },
  yAxis: {
    type: "log",
    // name: "迭代次数",
    alignTicks: true,
    axisTick: {
      show: true,
    },
    minorSplitLine: {
      show: true,
    },
    axisLabel: {
      formatter(value: number) {
        if (value === 0) {
          return "0";
        } else if (value === 1) {
          return "1";
        } else {
          return value.toExponential();
        }
      },
    },
  },
  toolbox: {
    right: 10,
    feature: {
      dataZoom: {
        yAxisIndex: "none",
      },
    },
  },
  dataZoom: [
    {
      type: "slider",
      startValue: 1,
      endValue: 1000,
    },
    {
      type: "inside",
    },
  ],
  series: [
    {
      type: "line",
      name: "u",
      encode: {
        x: 0,
        y: 1,
      },
      sampling: "minmax",
    },
    {
      type: "line",
      name: "phi",
      encode: {
        x: 0,
        y: 2,
      },
      sampling: "minmax",
    },
  ],
}));
</script>

<style scoped></style>
