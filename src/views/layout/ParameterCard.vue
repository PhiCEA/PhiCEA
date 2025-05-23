<template>
  <n-card title="模型参数" :bordered="false" size="large" aria-modal="true">
    <n-scrollbar v-if="jobs.currentJob?.parameters" max-h-lg trigger="none">
      <pre>
        {{ JSON.stringify(jobs.currentJob?.parameters, null, 16) }}
      </pre>
    </n-scrollbar>
    <n-empty v-else></n-empty>
  </n-card>
</template>

<script setup lang="ts">
import { NCard, NScrollbar, NEmpty } from "naive-ui";
import { useJobStore } from "@/stores/job";

const jobs = useJobStore();
</script>
