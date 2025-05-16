<template>
  <n-drawer
    v-model:show="show"
    :default-width="528"
    placement="right"
    resizable
  >
    <n-drawer-content>
      <template #header>
        <n-flex justify="space-between" align="center">
          <div select-none>任务列表</div>
          <n-button @click="handleImport" secondary>
            <template #icon>
              <div class="i-line-md:file-import-filled"></div>
            </template>
            导入
          </n-button>
        </n-flex>
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
  NFlex,
  useDialog,
} from "naive-ui";
import { h, nextTick, ref } from "vue";
import { JobInfo, useJobStore } from "@/stores/job";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { renderIcon } from "@/components/common";
import { RowDelete } from "@vicons/carbon";
import { ShowChartFilled } from "@vicons/material";

const jobs = useJobStore();
const message = useMessage();
const dialog = useDialog();

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
    icon: renderIcon(ShowChartFilled),
  },
  {
    label: () => h("span", { class: "text-red-600 font-500" }, "删除"),
    key: "delete",
    icon: renderIcon(RowDelete),
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
    dialog.warning({
      title: "删除任务",
      content: `确定要删除任务 ${dropDownSelectedId.value} 吗？`,
      positiveText: "删除",
      negativeText: "取消",
      onPositiveClick: () => {
        jobs.removeJob(dropDownSelectedId.value!);
        message.success(`删除成功  ${dropDownSelectedId.value}`);
      },
    });
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
