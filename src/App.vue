<template>
  <n-config-provider
    :locale="locale"
    :date-locale="dateLocale"
    :theme-overrides="themeOverrides"
  >
    <n-message-provider>
      <layout />
    </n-message-provider>
  </n-config-provider>
</template>

<script lang="ts" setup>
import {
  NMessageProvider,
  NConfigProvider,
  zhCN,
  dateZhCN,
  GlobalThemeOverrides,
} from "naive-ui";
import Layout from "@/views/Layout.vue";
import { ref } from "vue";

const locale = ref(zhCN);
const dateLocale = ref(dateZhCN);

const themeOverrides: GlobalThemeOverrides = {
  // common: {
  //   bodyColor: "#f8f8f8",
  // },
  // Drawer: {
  //   color: "#f8f8f8",
  // },
  // Split: {
  //   color: "#f8f8f8",
  //   resizableTriggerColor: "#e5e5e5"
  // },
};
</script>

<style scoped></style>
