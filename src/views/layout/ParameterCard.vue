<template>
  <n-card title="模型参数" :bordered="false" size="large" aria-modal="true">
    <n-scrollbar v-if="jobs.currentJob?.parameters" max-h-lg trigger="none">
      <vue-json-pretty :data="(jobs.currentJob?.parameters as JSONDataType)" />
    </n-scrollbar>
    <n-empty v-else></n-empty>
  </n-card>
</template>

<script setup lang="ts">
import { NCard, NScrollbar, NEmpty } from "naive-ui";
import { useJobStore } from "@/stores/job";
import VueJsonPretty from "vue-json-pretty";
import "vue-json-pretty/lib/styles.css";
import { JSONDataType } from "vue-json-pretty/types/utils";

const jobs = useJobStore();
</script>
