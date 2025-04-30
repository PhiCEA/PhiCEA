<template>
  <n-layout has-sider h-screen>
    <n-layout-sider bordered :width="48" :native-scrollbar="false">
      <activity-bar></activity-bar>
    </n-layout-sider>
    <n-layout-content>
      <router-view></router-view>
    </n-layout-content>
  </n-layout>
</template>

<script lang="ts" setup>
import { NLayout, NLayoutContent, NLayoutSider } from "naive-ui";
import ActivityBar from "@/views/layout/ActivityBar.vue";
</script>

<style scoped></style>
