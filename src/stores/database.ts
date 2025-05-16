import Database from "@tauri-apps/plugin-sql";
import { defineStore } from "pinia";
import { computed } from "vue";
import { useConfigStore } from "./config";

const useDatabaseStore = defineStore("database", () => {
  const config = useConfigStore();

  const databaseURL = computed(() =>
    config.promise.then((config) => {
      const { database } = config;
      return `postgres://${database.user}:${database.password}@${database.host}:${database.port}/${database.database}`;
    })
  );

  const promise = computed(() => databaseURL.value.then(Database.load));

  return {
    promise,
  };
});

export { useDatabaseStore };
