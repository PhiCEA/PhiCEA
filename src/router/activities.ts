import type { RouteRecordRaw } from "vue-router";

export default [
  {
    path: "/",
    redirect: "/home",
  },
  {
    name: "home",
    path: "/home",
    component: () => import("@/views/Home.vue"),
  },
  {
    name: "dashboard",
    path: "/dashboard",
    component: () => import("@/views/Dashboard.vue"),
  },
] as RouteRecordRaw[];
