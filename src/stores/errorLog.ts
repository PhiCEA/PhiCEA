import { defineStore } from "pinia";
import { useJobStore } from "./job";
import { computedAsync } from "@vueuse/core";
import { useDatabase } from "@/components/common";
import { computed } from "vue";

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
  const summary = computedAsync<ErrorLogSumary[]>(async (_onCancel) => {
    if (jobs.currentJob) {
      const db = await useDatabase();
      const result = await db.select<ErrorLogSumary[]>(
        "SELECT load, iters, extract(EPOCH from lag(timestamp, -1) over (order by load) - timestamp)::DOUBLE PRECISION as cost FROM error_log_summary WHERE job_id = $1;",
        [jobs.currentJob.id]
      );
      return result;
    } else {
      return [];
    }
  }, []);

  const toltalTime = computedAsync<string>(async (_onCancel) => {
    if (jobs.currentJob) {
      const db = await useDatabase();
      const result = await db.select<[{ total: number }]>(
        "SELECT extract(EPOCH from max(timestamp) - min(timestamp))::DOUBLE PRECISION as total FROM error_log WHERE job_id = $1;",
        [jobs.currentJob.id]
      );

      const sec = result[0].total;
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
    } else {
      return "-";
    }
  }, "-");

  const errors = computedAsync<ErrorLog[]>(async (_onCancel) => {
    if (jobs.currentJob) {
      const db = await useDatabase();
      const result = await db.select<ErrorLog[]>(
        "SELECT ROW_NUMBER() OVER (ORDER BY timestamp) as iters, load, error_u, error_phi FROM error_log WHERE job_id = $1 ORDER BY timestamp;",
        [jobs.currentJob.id]
      );
      let idx = 0;
      while (idx < result.length) {
        let error_u: number | null = null;
        let error_phi: number | null = null;
        const value = result[idx];

        // 舍入小值
        if (value.error_u !== null && value.error_u > 1e-25) {
          error_u = value.error_u;
        }
        if (value.error_phi !== null && value.error_phi > 1e-25) {
          error_phi = value.error_phi;
        }
        result[idx] = { ...value, error_u, error_phi };

        // 插入断点标记在 load 改变的地方
        if (
          idx > 0 &&
          result[idx - 1].iters !== null && // 防止死循环
          value.load !== result[idx - 1].load
        ) {
          result.splice(idx, 0, {
            iters: null,
            load: value.load,
            error_u: null,
            error_phi: null,
          });
        }
        idx += 1;
      }
      return result;
    } else {
      return [];
    }
  }, []);

  const iterations = computed(() => {
    if (errors.value.length > 0) {
      return errors.value[errors.value.length - 1].iters!;
    } else {
      return 0;
    }
  });

  return {
    summary,
    errors,
    toltalTime,
    iterations,
  };
});

export { useLogStore };
