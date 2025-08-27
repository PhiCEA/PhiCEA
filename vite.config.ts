import { defineConfig } from "vite";
import { fileURLToPath, URL } from "url";
import vue from "@vitejs/plugin-vue";
import UnoCSS from "unocss/vite";
import vueDevTools from "vite-plugin-vue-devtools";
import { visualizer } from "rollup-plugin-visualizer";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    vue(),
    UnoCSS(),
    vueDevTools(),
    visualizer({
      filename: "dist/stats.html",
      open: true, // 构建完成后自动打开报告页面
      gzipSize: true, // 显示 gzip 压缩大小
    }),
  ],

  resolve: {
    alias: [
      {
        find: "@",
        replacement: fileURLToPath(new URL("./src", import.meta.url)),
      },
      {
        find: "@assets",
        replacement: fileURLToPath(new URL("./src/assets", import.meta.url)),
      },
      {
        find: "@stores",
        replacement: fileURLToPath(new URL("./src/stores", import.meta.url)),
      },
      {
        find: "@views",
        replacement: fileURLToPath(new URL("./src/views", import.meta.url)),
      },
    ],
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
