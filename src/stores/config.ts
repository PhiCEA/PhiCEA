import type { Config } from "@/config";
import { invoke } from "@tauri-apps/api/core";
import { computedAsync } from "@vueuse/core";
import { defineStore } from "pinia";
import { ref, shallowRef } from "vue";

const useConfigStore = defineStore("config", () => {
  const promise = shallowRef(invoke<Config>("read_config"));
  const evaluating = ref(false);

  /**
   * 用于与用户交互，不应用于任务逻辑。
   *
   * 注意：使用时应尽量配合 `evaluating` 使用。
   */
  const config = computedAsync<Config>(
    async (_onCancel) => await promise.value,
    {
      database: {
        host: "",
        port: -1,
        database: "",
        user: "",
        password: "",
      },
    } as Config,
    evaluating
  );

  /**
   * 重新读取配置文件
   */
  function reload() {
    promise.value = invoke<Config>("read_config");
  }

  /**
   * 保存配置文件，并且立马重新读取
   */
  function save(config: Config) {
    invoke<Config>("write_config", { config }).then(reload);
  }

  return {
    promise,
    evaluating,
    config,
    reload,
    save,
  };
});

export { useConfigStore };
