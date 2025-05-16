<template>
  <n-flex :size="0" vertical align="center" h-full>
    <n-popover
      style="padding: 5px 10px"
      trigger="hover"
      :delay="popoverDelay"
      placement="right"
    >
      <template #trigger>
        <n-flex
          class="h-10 w-full cursor-pointer bg-#0078d4"
          color="active:#f8f8f8 hover:#f8f8f8 #e5e5e5"
          align="center"
          justify="center"
          @click="toggleExplorer"
        >
          <n-icon :size="24">
            <AppsList24Regular></AppsList24Regular>
          </n-icon>
        </n-flex>
      </template>
      <template #default>
        <span class="text-xs">任务列表</span>
      </template>
    </n-popover>
    <template v-for="act in activities">
      <n-popover
        trigger="hover"
        :delay="popoverDelay"
        placement="right"
        style="padding: 5px 10px"
      >
        <template #trigger>
          <router-link
            class="w-full h-10 button-override"
            un-color="hover:#1f1f1f #616161"
            :class="{ active: act.key === currentKey }"
            v-bind="act.link"
            @click="handleClickLink(act.key)"
          >
            <n-icon w-full py="2" :size="24" :component="act.icon" />
          </router-link>
        </template>
        <template #default>
          <span class="text-xs">
            {{ act.tip }}
          </span>
        </template>
      </n-popover>
    </template>

    <div class="grow"></div>
    <n-flex
      class="h-10 w-full cursor-pointer"
      color="hover:#1f1f1f #616161"
      align="center"
      justify="center"
      @click="openSettingsModal"
    >
      <n-icon :size="24">
        <settings-icon></settings-icon>
      </n-icon>
    </n-flex>
    <div class="basis-2"></div>
  </n-flex>
  <job-explorer v-model:show="showExplorer" />
  <suspense>
    <settings v-model:show="showSettingsModal" />
  </suspense>
</template>

<script setup lang="ts">
import { NFlex, NPopover, NIcon } from "naive-ui";
import { ref } from "vue";
import { RouterLink, useRouter } from "vue-router";
import { ChartLineData, Settings as SettingsIcon } from "@vicons/carbon";
import { AppsList24Regular } from "@vicons/fluent";
import JobExplorer from "./JobExplorer.vue";
import Settings from "./Settings.vue";

const activities = [
  {
    key: "dashboard",
    tip: "过程分析",
    icon: ChartLineData,
    link: {
      to: {
        name: "dashboard",
      },
    },
  },
];

const currentKey = ref<String | null>(null);
const router = useRouter();
const handleClickLink = (key: string) => {
  currentKey.value = currentKey.value === key ? null : key;
  if (currentKey.value === null) {
    router.replace({ name: "home" });
  }
};

/* Popover */
const popoverDelay = 500; // ms

/* Job Explorer */
const showExplorer = ref(false);
const toggleExplorer = () => {
  showExplorer.value = !showExplorer.value;
};

/* Settings */
// 打开设置弹窗
const showSettingsModal = ref(false);
const openSettingsModal = () => {
  showSettingsModal.value = true;
};
</script>

<style scoped>
.button-override {
  border-left: 4px solid transparent;
}
.button-override.active {
  border-left: 4px solid #0078d4;
  color: #1f1f1f;
}
</style>
