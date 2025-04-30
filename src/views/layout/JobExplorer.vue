<template>
  <n-drawer
    v-model:show="show"
    :default-width="528"
    placement="right"
    resizable
  >
    <n-drawer-content>
      <template #header>
        <div class="flex justify-between items-center">
          <div select-none>Job List</div>
          <div>
            <n-button @click="handleImport" class="grow-0"> 导入 </n-button>
          </div>
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
  DropdownOption,
  useMessage,
  PaginationProps,
  DataTableColumns,
  NDrawer,
  NDropdown,
  NDrawerContent,
  NDataTable,
  NButton,
} from "naive-ui";
import { nextTick, ref } from "vue";
import { JobInfo, useJobStore } from "@/stores/job";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

const jobs = useJobStore();
const message = useMessage();

const show = defineModel<boolean>("show", { default: false });
const pagination: PaginationProps = {
  pageSize: 10,
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
    key: "display",
  },
  {
    label: "删除",
    key: "delete",
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
  if (key === "display") {
    show.value = false;
    jobs.setCurrent(dropDownSelectedId.value!);
    message.info(`显示  ${dropDownSelectedId.value!}`);
    dropDownSelectedId.value = null;
  } else if (key === "delete") {
    // TODO: delete job
    message.info(`删除  ${dropDownSelectedId.value!}`);
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
</script>

<style scoped></style>
