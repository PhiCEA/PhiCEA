<template>
  <n-modal
    v-model:show="show"
    :mask-closable="false"
    max-w-160
    preset="card"
    draggable
  >
    <template #header>
      <n-icon px-1 size="24" color="#0078d4">
        <settings-icon />
      </n-icon>
      <span text-6 align-bottom>设置</span>
    </template>
    <n-form
      :model="config.database"
      name="settings"
      label-width="80"
      m="t-sm r-xl b-sm"
    >
      <n-tabs
        type="line"
        placement="left"
        size="small"
        min-h-120
        pane-class="ml-4"
        animated
      >
        <n-tab-pane name="database" tab="数据库">
          <n-form-item
            path="database.host"
            label="主机地址"
            :label-props="{ for: 'database.host' }"
          >
            <n-input
              v-model:value="config.database.host"
              placeholder="请输入主机地址"
              :input-props="{ id: 'database.host' }"
            />
          </n-form-item>
          <n-form-item
            path="database.port"
            label="端口"
            :label-props="{ for: 'database.port' }"
          >
            <n-input-number
              v-model:value="config.database.port"
              placeholder="请输入端口号"
              :input-props="{ id: 'database.port' }"
            />
          </n-form-item>
          <n-form-item
            path="database.database"
            label="数据库名称"
            :label-props="{ for: 'database.database' }"
          >
            <n-input
              v-model:value="config.database.database"
              placeholder="请输入数据库名称"
              :input-props="{ id: 'database.database' }"
            />
          </n-form-item>
          <n-form-item
            path="database.user"
            label="用户名"
            :label-props="{ for: 'database.user' }"
          >
            <n-input
              v-model:value="config.database.user"
              placeholder="请输入用户名"
              :input-props="{ id: 'database.user' }"
            />
          </n-form-item>
          <n-form-item
            path="database.password"
            label="密码"
            :label-props="{ for: 'database.password' }"
          >
            <n-input
              v-model:value="config.database.password"
              type="password"
              show-password-on="mousedown"
              placeholder="请输入密码"
              :input-props="{ id: 'database.password' }"
            />
          </n-form-item>
          <n-button w-full :type="statusButtonType" @click="testConnection">
            <template #icon>
              <div :class="statusIcon"></div>
            </template>
            {{ statusText }}
          </n-button>
        </n-tab-pane>
        <n-tab-pane name="database.others" tab="其它">
          <n-empty description="占个位置..." class="h-full justify-center" />
        </n-tab-pane>
      </n-tabs>
      <div class="flex justify-end mt-4">
        <n-button @click="show = false" class="mr-2">取消</n-button>
        <n-button :type="changed ? 'primary' : 'default'" @click="saveSettings">
          <template #icon>
            <div class="i-ic:round-save-as" v-if="changed"></div>
          </template>
          保存
        </n-button>
      </div>
    </n-form>
  </n-modal>
</template>

<script setup lang="ts">
import {
  NButton,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NModal,
  NIcon,
  NTabs,
  NTabPane,
  NEmpty,
  useMessage,
} from "naive-ui";
import { Settings as SettingsIcon } from "@vicons/carbon";
import { useConfigStore } from "@/stores/config";
import { computed, ref, shallowRef, watch } from "vue";
import { isEqual } from "lodash-es";
import Database from "@tauri-apps/plugin-sql";

const show = defineModel<boolean>("show", { default: false });

const conf = useConfigStore();
let rawConfig = shallowRef(await conf.promise);
const config = ref(structuredClone(rawConfig.value));
const changed = computed(() => !isEqual(config.value, rawConfig.value));
watch(
  () => conf.promise,
  async (curr, _prev) => {
    rawConfig.value = await curr;
  }
);

const message = useMessage();
const testStatus = ref<"pending" | "testing" | "passed" | "failed">("passed");
watch(changed, (curr, prev) => {
  if (curr && !prev) {
    testStatus.value = "pending";
  } else if (!curr && prev) {
    testStatus.value = "passed";
  }
});
const statusText = computed(() => {
  if (testStatus.value === "pending") {
    return "测试连接";
  } else if (testStatus.value === "testing") {
    return "正在测试";
  } else if (testStatus.value === "passed") {
    return "连接成功";
  } else if (testStatus.value === "failed") {
    return "连接失败";
  }
});
const statusButtonType = computed(() => {
  if (testStatus.value === "pending") {
    return "warning";
  } else if (testStatus.value === "testing") {
    return "info";
  } else if (testStatus.value === "passed") {
    return "success";
  } else if (testStatus.value === "failed") {
    return "error";
  }
});
const statusIcon = computed(() => {
  if (testStatus.value === "pending") {
    return "i-fluent:question-circle-24-regular";
  } else if (testStatus.value === "testing") {
    return "i-svg-spinners:ring-resize";
  } else if (testStatus.value === "passed") {
    return "i-line-md:circle-to-confirm-circle-twotone-transition";
  } else if (testStatus.value === "failed") {
    return "i-fluent:error-circle-24-regular animate-heart-beat";
  }
});
const testConnection = () => {
  const database = config.value.database;
  const url = `postgres://${database.user}:${database.password}@${database.host}:${database.port}/${database.database}`;
  testStatus.value = "testing";
  Database.load(url)
    .then(() => {
      testStatus.value = "passed";
      message.success("连接成功");
    })
    .catch(() => {
      testStatus.value = "failed";
      message.error("连接失败，请检查配置！");
    });
};
const saveSettings = () => {
  if (changed.value) {
    if (testStatus.value === "failed") {
      message.error("连接失败，请检查配置！");
      return;
    } else if (testStatus.value === "pending") {
      message.warning("请先测试连接");
      return;
    } else if (testStatus.value === "testing") {
      message.warning("正在测试连接，请稍等...");
      return;
    }
    conf.save(config.value);
    message.success("保存设置成功");
  }

  show.value = false;
};
</script>

<style scoped></style>
