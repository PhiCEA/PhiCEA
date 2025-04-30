interface DatabaseConfig {
  user: string;
  password: string;
  host: string;
  port: number;
  database: string;
}

interface Config {
  database: DatabaseConfig;
}

export default {
  database: fetch(new URL("../public/config.json", import.meta.url))
    .then((response) => {
      return response.json() as Promise<Config>;
    })
    .then(({ database }) => {
      return `postgres://${database.user}:${database.password}@${database.host}:${database.port}/${database.database}`;
    }),
};
