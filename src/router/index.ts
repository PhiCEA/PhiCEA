// 导入router所需的方法
import { createRouter, createWebHistory } from 'vue-router'

// 导入路由页面的配置
import { RouteRecordRaw } from "vue-router";

const routeList: Record<string, RouteRecordRaw> = import.meta.glob(
  ["!./index.ts", "./*.ts"],
  {
    eager: true,
    import: "default",
  }
);
const routes: RouteRecordRaw[] = Object.values(routeList).flat();

// 路由参数配置
const router = createRouter({
    // 使用hash(createWebHashHistory)模式，(createWebHistory是HTML5历史模式，支持SEO)
    history: createWebHistory(),
    routes,
})

// 全局前置守卫，这里可以加入用户登录判断
router.beforeEach((_to, _from) => {
    // 返回 false 以取消导航
})

// 全局后置钩子，这里可以加入改变页面标题等操作
router.afterEach((_to, _from) => {
    
})

// 导出默认值
export default router
