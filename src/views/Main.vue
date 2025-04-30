<template>
  <n-button @click="handleImport"> 导入 </n-button>
  <n-button @click="toggleDrawer"> 显示列表 </n-button>
  <dashboard></dashboard>
  <n-drawer
    v-model:show="active"
    :default-width="528"
    placement="right"
    resizable
  >
    <n-drawer-content>
      <template #header>
        <div class="flex justify-between">
          <div>Job List</div>
          <div>{{ jobs.currentJob?.id }}</div>
        </div>
      </template>
      <n-data-table
        :columns="columns"
        :data="jobs.list"
        :pagination="pagination"
        :bordered="false"
        :row-props="rowProps"
        class="h-full"
        flex-height
      />
      <n-dropdown
        placement="bottom-start"
        trigger="manual"
        :x="x"
        :y="y"
        :options="options"
        :show="showDropdown"
        :on-clickoutside="onClickoutside"
        @select="handleSelect"
      />
    </n-drawer-content>
  </n-drawer>
</template>

<script setup lang="ts">
import {
  NButton,
  NDrawer,
  NDataTable,
  NDrawerContent,
  NDropdown,
  useMessage,
} from "naive-ui";
import type {
  DataTableColumns,
  DropdownOption,
  PaginationProps,
} from "naive-ui";
import { nextTick, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { JobInfo, useJobStore } from "@/stores/job";
import Dashboard from "@/views/Dashboard.vue";

const jobs = useJobStore();
const message = useMessage();
const handleImport = async () => {
  const file = await open({
    multiple: false,
    filters: [
      {
        name: "Log Files",
        extensions: ["log"],
      },
    ],
  });
  if (file) {
    const jobId = await invoke<number>("import_log", { file });
    jobs.addJob(jobId);
    message.info(`导入成功  ${jobId}`);
  }
};

const active = ref(false);
const pagination: PaginationProps = {
  pageSize: 10,
};
const toggleDrawer = () => {
  active.value = !active.value;
};

function createColumns(): DataTableColumns<JobInfo> {
  return [
    {
      title: "ID",
      key: "id",
      width: 50,
    },
    {
      title: "名字",
      key: "name",
      width: 120,
      ellipsis: true,
    },
    {
      title: "队列",
      key: "queue",
      width: 50,
    },
    {
      title: "核数",
      key: "num_cpu",
      width: 30,
    },
  ];
}

const columns = ref(createColumns());

const options = ref([
  {
    label: "显示",
    key: "show",
  },
]);
const showDropdown = ref(false);
const x = ref(0);
const y = ref(0);
const onClickoutside = () => {
  showDropdown.value = false;
  dropDownSelectedId.value = null;
};
const handleSelect = (key: string | number, _option: DropdownOption) => {
  showDropdown.value = false;
  if (key === "show") {
    toggleDrawer();
    jobs.setCurrent(dropDownSelectedId.value!);
    message.info(`显示  ${dropDownSelectedId.value!}`);
    dropDownSelectedId.value = null;
  }
};

const dropDownSelectedId = ref<number | null>(null);

const rowProps = (jobInfo: JobInfo) => {
  return {
    onContextmenu: (e: MouseEvent) => {
      e.preventDefault();
      showDropdown.value = false;
      nextTick().then(() => {
        showDropdown.value = true;
        dropDownSelectedId.value = jobInfo.id;
        x.value = e.clientX;
        y.value = e.clientY;
      });
    },
  };
};
</script>

<style scoped></style>
