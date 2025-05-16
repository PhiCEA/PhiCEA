export interface DatabaseConfig {
  user: string;
  password: string;
  host: string;
  port: number;
  database: string;

  [key: string] : any;
}

export interface Config {
  database: DatabaseConfig;

  [key: string] : any;
}
