import config from "@/config";
import Database from "@tauri-apps/plugin-sql";
import { h } from "vue";
import type { Component } from "vue";
import { NIcon } from "naive-ui";

const databasePromise: Promise<Database> = config.database.then(Database.load);

export async function useDatabase(): Promise<Database> {
  return databasePromise;
}

export function renderIcon(icon: Component) {
  return () => {
    return h(NIcon, null, {
      default: () => h(icon),
    });
  };
}
