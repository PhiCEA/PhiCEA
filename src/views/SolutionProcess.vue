<template>
  <div mx-10 pt-8>
    <h1 select-none font-medium mb-sm text="3xl #1F2225">过程分析</h1>
    <n-flex class="ml-0.5 h-min" :size="18" align="center">
      <n-button text @click="showParams = true"> 参数 </n-button>
      <n-button text icon-placement="right" @click="handleDisplayDetails">
        <template #icon>
          <n-icon>
            <component :is="displayDetails ? IosArrowUp : IosArrowDown" />
          </n-icon>
        </template>
        <template #default>
          <span>详情</span>
        </template>
      </n-button>
      <n-divider vertical />
      <n-switch v-model:value="isComparing">
        <template #checked-icon>
          <n-icon :component="RectangleLandscape12Regular" />
        </template>
        <template #unchecked-icon>
          <n-icon :component="SplitscreenRound" />
        </template>
      </n-switch>
    </n-flex>
  </div>
  <div class="mx-xl">
    <n-card v-if="displayDetails" :bordered="false">
      <n-grid :cols="3" :y-gap="8" :x-gap="12">
        <n-gi
          v-for="opt in detailsOptions"
          :span="opt.span ?? 1"
          :key="opt.label"
        >
          <n-statistic :label="opt.label">
            <n-ellipsis>
              {{ opt.value }}
            </n-ellipsis>
          </n-statistic>
        </n-gi>
      </n-grid>
    </n-card>
  </div>
  <n-modal v-model:show="showParams">
    <parameter-card style="width: 600px" />
  </n-modal>

  <v-chart
    class="w-full"
    :class="chartHeightClass"
    :theme="matplotlibTheme"
    :option="optionSummary"
    :autoresize="autoresize"
  />
  <v-chart
    class="w-full"
    :class="chartHeightClass"
    :theme="matplotlibTheme"
    :option="optionError"
    :autoresize="autoresizeError"
  />
</template>

<script setup lang="ts">
import {
  NGrid,
  NGi,
  NStatistic,
  NCard,
  NFlex,
  NButton,
  NIcon,
  NModal,
  NEllipsis,
  NDivider,
  NSwitch,
} from "naive-ui";
import { IosArrowDown, IosArrowUp } from "@vicons/ionicons4";
import { computed, ref } from "vue";
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
import { CallbackDataParams } from "echarts/types/dist/shared";
import type { ErrorLog, ErrorLogSumary } from "@/stores/errorLog";
import { useLogStore } from "@/stores/errorLog";
import { useJobStore } from "@/stores/job";
import ParameterCard from "./layout/ParameterCard.vue";
import matplotlibTheme from "@assets/matplotlib.theme.json";
import { SplitscreenRound } from "@vicons/material";
import { RectangleLandscape12Regular } from "@vicons/fluent";

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

const logs = useLogStore();
const jobs = useJobStore();

/*  模型详情  */
const displayDetails = ref(true);
const handleDisplayDetails = () => {
  displayDetails.value = !displayDetails.value;
};
const iterForatter = new Intl.NumberFormat(navigator.language);
const detailsOptions = computed(() => [
  {
    label: "任务ID",
    value: jobs.currentJob?.id ?? "-",
  },
  {
    label: "任务名",
    value: jobs.currentJob?.name ?? "-",
    span: 2,
  },
  {
    label: "时间",
    value: logs.toltalTime,
  },
  {
    label: "迭代",
    value: jobs.currentJob ? `${iterForatter.format(logs.iterations)} 次` : "-",
  },
]);

/* 折线图 */
const isComparing = ref(false);
const chartHeightClass = computed(() => [
  isComparing.value ? "h-full" : "h-50vh",
]);
const autoresize = {
  throttle: 20,
};
const autoresizeError = {
  throttle: 250,
};

const secFormatter = (sec: number) => {
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

const axisLabelSize = 14;

const optionSummary = computed<EChartsOption>(() => ({
  animation: false,
  legend: {
    top: "20px",
    textStyle: {
      fontSize: axisLabelSize,
      fontFamily: "Noto Sans SC",
    },
  },
  dataset: {
    dimensions: ["load", "cost", "iters", "load"],
    source: logs.summary as any,
  },
  tooltip: {
    trigger: "axis",
    textStyle: {
      fontSize: axisLabelSize + 1,
      fontFamily: "Noto Sans SC",
    },
    formatter: (params) => {
      params = params as CallbackDataParams[];
      const tooltip_error = params
        .map((param) => {
          // Ensure param.value is defined and properly typed
          const value = param.value as ErrorLogSumary;
          const key = param.dimensionNames![param.encode!.y[0]];
          const point = value[key];
          const name = param.seriesName;
          const color = param.color;
          let formattedItem;
          if (key === "cost") {
            if (point === null) {
              formattedItem = "-";
            } else {
              formattedItem = secFormatter(point);
            }
          } else {
            formattedItem = point;
          }
          return `<div class="flex justify-between leading-relaxed">
                    <div>
                      <span class="inline-block mr-1 h-2.5 w-2.5 rounded-full" style="background-color: ${color};"></span>
                      <span>${name}</span>
                    </div>
                    <span class="ml-5 font-600">${formattedItem}</span>
                  </div>`;
        })
        .join("");

      const data = params[0]?.data as ErrorLog;

      return `<div>
        <div mb-2>
          Load <span class="pl-1 font-600">${data.load}</span>
        </div>
        ${tooltip_error}
      </div>`;
    },
  },
  xAxis: {
    type: "value",
    axisLabel: {
      fontSize: axisLabelSize,
      fontFamily: "Noto Sans SC",
    },
  },
  yAxis: [
    {
      type: "value",
      name: "耗时",
      nameTextStyle: {
        fontSize: axisLabelSize,
        fontFamily: "Noto Sans SC",
      },
      axisLabel: {
        formatter: secFormatter,
        fontSize: axisLabelSize,
        fontFamily: "Noto Sans SC",
      },
    },
    {
      type: "value",
      name: "迭代次数",
      nameTextStyle: {
        fontSize: axisLabelSize,
        fontFamily: "Noto Sans SC",
      },
      axisLabel: {
        fontSize: axisLabelSize,
        fontFamily: "Noto Sans SC",
      },
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
      textStyle: {
        fontFamily: "Noto Sans SC",
      },
    },
    {
      type: "inside",
      filterMode: "none",
      textStyle: {
        fontFamily: "Noto Sans SC",
      },
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
      // 统一样式
      // tooltip: {
      //   valueFormatter(value: any, _dataIndex: any) {
      //     if (value === null) {
      //       return "-";
      //     } else {
      //       return secFormatter(value);
      //     }
      //   },
      // },
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
  legend: {
    top: "20px",
    textStyle: {
      fontFamily: "Noto Sans SC",
    },
  },
  dataset: {
    dimensions: ["iters", "error_u", "error_phi"],
    source: logs.errors as any[],
  },
  tooltip: {
    trigger: "axis",
    textStyle: {
      fontSize: axisLabelSize + 1,
      fontFamily: "Noto Sans SC",
    },
    formatter: (params) => {
      params = params as CallbackDataParams[];
      const tooltip_error = params
        .map((param) => {
          // Ensure param.value is defined and properly typed
          const value = param.value as ErrorLog;
          const error = value[param.dimensionNames![param.encode!.y[0]]];
          const name = param.seriesName;
          const color = param.color;
          return `<div class="flex justify-between leading-relaxed">
                    <div>
                      <span class="inline-block mr-1 h-2.5 w-2.5 rounded-full" style="background-color: ${color};"></span>
                      <span>${name}</span>
                    </div>
                    <span class="pl-5 font-600">${formatError(error)}</span>
                  </div>`;
        })
        .join("");

      const data = params[0]?.data as ErrorLog;

      return `<div>
                <div mb-2>
                  Load <span class="pl-1 font-600">${data.load}</span>
                </div>
                ${tooltip_error}
              </div>`;
    },
  },
  xAxis: {
    type: "value",
    axisLabel: {
      fontSize: axisLabelSize,
      fontFamily: "Noto Sans SC",
    },
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
      fontSize: axisLabelSize,
      fontFamily: "Noto Sans SC",
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
      textStyle: {
        fontFamily: "Noto Sans SC",
      },
    },
    {
      type: "inside",
      textStyle: {
        fontFamily: "Noto Sans SC",
      },
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

/* Parameters card */
const showParams = ref(false);
</script>

<style scoped></style>
