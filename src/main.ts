import { createApp } from "vue";
import { createPinia } from "pinia";
import "virtual:uno.css";
import "@unocss/reset/eric-meyer.css";
import App from "./App.vue";

// 路由
import router from "./router/";

// Pinia
const pinia = createPinia();


createApp(App).use(pinia).use(router).mount("#app");

// 禁止默认右键菜单
window.addEventListener("contextmenu", (e) => {
    e.preventDefault();
});