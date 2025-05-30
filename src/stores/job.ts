import { defineStore } from "pinia";
import { computed, ref, watch } from "vue";
import { Channel, invoke } from "@tauri-apps/api/core";
import { decode } from "@msgpack/msgpack";
import { useConfigStore } from "./config";

export interface JobInfo {
  id: number;
  name: string;
  queue: string;
  cpus: number;
  parameters?: object;
}

// FIXME: 错误处理 addJob, removeJob
const useJobStore = defineStore("job", () => {
  const config = useConfigStore();

  const list = ref<JobInfo[]>([]);
  const currentJobIndex = ref(-1);
  const currentJob = computed<JobInfo | undefined>(() => {
    if (list.value.length > 0) {
      return list.value[currentJobIndex.value];
    } else {
      return undefined;
    }
  });

  function addToList(jobId: number) {
    invoke<JobInfo>("find_job", { jobId }).then((newJob) => {
      list.value.push(newJob);
      list.value.sort((a, b) => a.id - b.id);
      currentJobIndex.value = list.value.indexOf(newJob);
    });
  }

  function removeJob(jobId: number) {
    invoke("remove_job", { jobId }).then(() => {
      list.value = list.value.filter((job) => job.id !== jobId);
      if (currentJob.value?.id === jobId) {
        currentJobIndex.value = -1;
      }
    });
  }

  function setCurrent(jobId: number) {
    currentJobIndex.value = list.value.findIndex((job) => job.id === jobId);
  }

  function updateList() {
    let currentJobId = currentJob.value?.id;
    const channel = new Channel();
    channel.onmessage = (response) => {
      const jobList = decode(response as ArrayBuffer) as Array<
        [number, string, string, number, object]
      >;
      list.value = jobList.map(([id, name, queue, cpus, parameters]) => ({
        id,
        name,
        queue,
        cpus,
        parameters,
      }));

      if (currentJobIndex.value < 0) {
        // first time
        currentJobIndex.value = 0;
      } else if (currentJobId) {
        // update job list
        setCurrent(currentJobId);
      }
    };
    invoke<JobInfo[]>("get_job_list", { channel }).catch((reason) => {
      console.error(reason);
      list.value = [];
    });
  }

  watch(
    ()  => config.promise,
    (_curr, _prev) => {
      updateList();
    },
    { immediate: true }
  );
  return {
    list,
    currentJob,
    addJob: addToList,
    removeJob,
    updateList,
    setCurrent,
  };
});

export { useJobStore };
