import { defineStore, storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import { useDatabaseStore } from "./database";

export interface JobInfo {
  id: number;
  name: string;
  queue: string;
  num_cpu: number;
  parameters?: object;
}

// FIXME: 错误处理 addJob, removeJob
const useJobStore = defineStore("job", () => {
  const database = useDatabaseStore();
  const { promise: dbPromise } = storeToRefs(database);

  const list = ref<JobInfo[]>([]);
  const currentJobIndex = ref(-1);
  const currentJob = computed<JobInfo | undefined>(() => {
    if (list.value.length > 0) {
      return list.value[currentJobIndex.value];
    } else {
      return undefined;
    }
  });

  function addJob(jobId: number) {
    dbPromise.value.then((db) => {
      db.select<JobInfo[]>(
        "SELECT id, name, queue, num_cpu, parameters FROM job_info WHERE id = $1;",
        [jobId]
      ).then((newJob) => {
        const job = newJob[0];
        list.value.push(job);
        list.value.sort((a, b) => a.id - b.id);
        currentJobIndex.value = list.value.indexOf(job);
      });
    });
  }

  function removeJob(jobId: number) {
    dbPromise.value.then((db) => {
      db.execute("DELETE FROM job_info WHERE id = $1;", [jobId]).then(() => {
        list.value = list.value.filter((job) => job.id !== jobId);
        if (currentJob.value?.id === jobId) {
          currentJobIndex.value = -1;
        }
      });
    });
  }

  function setCurrent(jobId: number) {
    currentJobIndex.value = list.value.findIndex((job) => job.id === jobId);
  }

  function updateList() {
    dbPromise.value.then((db) => {
      let currentJobId = currentJob.value?.id;
      db.select<JobInfo[]>(
        "SELECT id, name, queue, num_cpu, parameters FROM job_info;"
      )
        .then((jobList) => {
          list.value = jobList;

          if (currentJobIndex.value < 0) {
            // first time
            currentJobIndex.value = 0;
          } else if (currentJobId) {
            // update job list
            setCurrent(currentJobId);
          }
        })
        .catch((reason) => {
          console.error(reason);
          list.value = [];
        });
    });
  }

  watch(
    dbPromise,
    (_curr, _prev) => {
      updateList();
    },
    { immediate: true }
  );
  return {
    list,
    currentJob,
    addJob,
    removeJob,
    updateList,
    setCurrent,
  };
});

export { useJobStore };
