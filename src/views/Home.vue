<template>
  <n-flex w-full h-full align="center" justify="center">
    <n-empty pb-12 size="huge" description="你什么也找不到">
      <template #icon>
        <n-icon>
          <ConstructionFilled />
        </n-icon>
      </template>
    </n-empty>
  </n-flex>
</template>

<script setup lang="ts">
import { NFlex, NEmpty, NIcon } from "naive-ui";
import { ConstructionFilled } from "@vicons/material";
</script>

<style scoped></style>
