import config from "@/config";
import Database from "@tauri-apps/plugin-sql";

const databasePromise: Promise<Database> = config.database.then(Database.load);

export async function useDatabase(): Promise<Database> {
  return databasePromise;
}
