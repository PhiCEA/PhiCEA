<template>
  <n-card title="模型参数" :bordered="false" size="large" aria-modal="true">
    <template v-if="jobs.currentJob?.parameters">
      {{ JSON.stringify(jobs.currentJob?.parameters, null, 2) }}
    </template>
    <n-empty v-else></n-empty>
  </n-card>
</template>

<script setup lang="ts">
import { NCard, NEmpty } from "naive-ui";
import { useJobStore } from "@/stores/job";

const jobs = useJobStore();
</script>
