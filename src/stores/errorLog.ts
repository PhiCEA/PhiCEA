import { defineStore } from "pinia";
import { useJobStore } from "./job";
import { computedAsync } from "@vueuse/core";
import { computed, shallowRef, watchEffect } from "vue";
import { Channel, invoke } from "@tauri-apps/api/core";
import { decode } from "@msgpack/msgpack";

export interface ErrorLogSumary {
  load: number;
  iters: number;
  cost: number | null;

  [key: string]: number | null;
}

export interface ErrorLog {
  iters: number | null; // `null` 表示echarts在这里断开
  load: number;
  error_u: number | null; // `null` 表示没有 error 为 0
  error_phi: number | null;

  [key: string]: number | null;
}

const useLogStore = defineStore("errorLog", () => {
  const jobs = useJobStore();

  const summary = shallowRef<ErrorLogSumary[]>([]);
  const errors = shallowRef<ErrorLog[]>([]);

  const parseResponse = (response: ArrayBuffer) => {
    // MessagePack decoding
    const [summaryArray, errorLogArray] = decode(response) as [
      Array<[number, number, number]>,
      Array<[number, number, number, number]>
    ];

    // error log
    const errorLog = errorLogArray.map(([iters, load, error_u, error_phi]) => ({
      iters,
      load,
      error_u,
      error_phi,
    }));
    // 加工数据，以用于echarts画图
    splitErrorLog(errorLog);
    errors.value = errorLog;

    // error summary
    summary.value = summaryArray.map(([load, iters, cost]) => ({
      load,
      iters,
      cost,
    }));
  };

  watchEffect(() => {
    if (jobs.currentJob) {
      const channel = new Channel<Uint8Array>();
      channel.onmessage = parseResponse;
      invoke("get_error_log", {
        jobId: jobs.currentJob.id,
        channel,
      });
    } else {
      summary.value = [];
      errors.value = [];
    }
  });

  const iterations = computed(() => {
    if (errors.value.length > 0) {
      return errors.value[errors.value.length - 1].iters!;
    } else {
      return 0;
    }
  });

  const toltalTime = computedAsync<string>(async (_onCancel) => {
    if (!jobs.currentJob) {
      return "-";
    }
    const sec = await invoke<number>("get_total_time", {
      jobId: jobs.currentJob.id,
    });

    // 计算秒数
    const seconds = Math.floor(sec % 60);
    // 计算分钟数
    const minutes = Math.floor(sec / 60);
    // 计算小时数
    const hours = Math.floor(minutes / 60);
    // 计算天数
    const days = Math.floor(hours / 24);
    const duration = {
      days,
      hours: hours % 24,
      minutes: minutes % 60,
      seconds,
    };
    // @ts-ignore
    return new Intl.DurationFormat(navigator.language).format(duration);
  }, "-");

  return {
    summary,
    errors,
    toltalTime,
    iterations,
  };
});

function splitErrorLog(errors: Array<ErrorLog>) {
  let idx = 0;
  while (idx < errors.length) {
    let error_u: number | null = null;
    let error_phi: number | null = null;
    const value = errors[idx];

    // 舍入小值
    if (value.error_u !== null && value.error_u > 1e-25) {
      error_u = value.error_u;
    }
    if (value.error_phi !== null && value.error_phi > 1e-25) {
      error_phi = value.error_phi;
    }
    errors[idx] = { ...value, error_u, error_phi };

    // 插入断点标记在 load 改变的地方
    if (
      idx > 0 &&
      errors[idx - 1].iters !== null && // 防止死循环
      value.load !== errors[idx - 1].load
    ) {
      errors.splice(idx, 0, {
        iters: null,
        load: value.load,
        error_u: null,
        error_phi: null,
      });
    }
    idx += 1;
  }
}

export { useLogStore };
