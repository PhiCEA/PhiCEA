import { useDatabase } from "@/components/common";
import { defineStore } from "pinia";
import { computed, ref } from "vue";

export interface JobInfo {
  id: number;
  name: string;
  queue: string;
  num_cpu: number;
}

const useJobStore = defineStore("job", () => {
  const databasePromise = useDatabase();

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
    databasePromise.then((db) => {
      db.select<JobInfo[]>(
        "SELECT id, name, queue, num_cpu FROM job_info WHERE id = $1;",
        [jobId]
      ).then((newJob) => {
        const job = newJob[0];
        list.value.push(job);
        list.value.sort((a, b) => a.id - b.id);
        currentJobIndex.value = list.value.indexOf(job);
      });
    });
  }

  function setCurrent(jobId: number) {
    currentJobIndex.value = list.value.findIndex((job) => job.id === jobId);
  }

  function updateList() {
    databasePromise.then((db) => {
      let currentJobId = currentJob.value?.id;
      db.select<JobInfo[]>(
        "SELECT id, name, queue, num_cpu FROM job_info;"
      ).then((jobList) => {
        list.value = jobList;

        if (currentJobIndex.value < 0) {
          // first time
          currentJobIndex.value = 0;
        } else if (currentJobId) {
          // update job list
          setCurrent(currentJobId);
        }
      });
    });
  }

  updateList();
  return {
    list,
    currentJob,
    addJob,
    updateList,
    setCurrent,
  };
});

export { useJobStore };
