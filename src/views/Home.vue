<template>
  <n-empty
    size="huge"
    description="这里什么也没有..."
    class="h-full justify-center"
  >
    <template #icon>
      <n-icon>
        <ConstructionFilled />
      </n-icon>
    </template>
  </n-empty>
</template>

<script setup lang="ts">
import { NEmpty, NIcon } from "naive-ui";
import { ConstructionFilled } from "@vicons/material";
</script>

<style scoped></style>
