CREATE TABLE "job_info"(
    "id" BIGINT NOT NULL,
    "name" TEXT NOT NULL,
    "queue" TEXT NOT NULL,
    "num_cpu" INTEGER NOT NULL,
    "nodes" TEXT[] NOT NULL,
    "paramters" jsonb NOT NULL
);
ALTER TABLE
    "job_info" ADD PRIMARY KEY("id");
CREATE INDEX "job_info_id_index" ON
    "job_info"("id");

-----------------------------------------------------------

CREATE TABLE "error_log"(
    "job_id" BIGINT NOT NULL,
    "timestamp" TIMESTAMP(3) WITHOUT TIME ZONE NOT NULL,
    "load" DOUBLE PRECISION NOT NULL,
    "iter" INTEGER NOT NULL,
    "error_u" DOUBLE PRECISION NOT NULL,
    "error_phi" DOUBLE PRECISION NOT NULL
);
CREATE INDEX "error_log_job_id_index" ON
    "error_log"("job_id");
CREATE INDEX "error_log_job_id_iter_index" ON
    "error_log"("job_id", "iter");
-- CREATE INDEX error_log_job_id_load_iter_index ON
--     "error_log"("job_id", "load", "iter");

CREATE VIEW error_log_summary AS
WITH 
"duration" AS (
  -- 获取每个 job_id + load 在 iter=1 时的 timestamp
  SELECT 
    "job_id", 
    "load", 
    "timestamp" 
  FROM "error_log" 
  WHERE "iter" = 1
),
"iterations" AS (
  -- 计算每个 job_id + load 的最大迭代次数
  SELECT 
    "job_id", 
    "load", 
    MAX("iter") AS "iters"
  FROM "error_log" 
  GROUP BY "job_id", "load"
)
SELECT
  "duration"."job_id",
  "duration"."load",
  "duration"."timestamp",
  "iterations"."iters"
FROM "duration"
JOIN "iterations" 
  ON "duration"."job_id" = "iterations"."job_id"  -- 按 job_id 和 load 双重关联
  AND "duration"."load" = "iterations"."load"
ORDER BY 
  "duration"."job_id" ASC, 
  "duration"."load" ASC;

-----------------------------------------------------------

CREATE TABLE "modeling"(
    "id" INTEGER NOT NULL,
    "desc" TEXT
);
ALTER TABLE
    "modeling" ADD PRIMARY KEY("id");

-----------------------------------------------------------

CREATE TABLE "modeling_jobs"(
    "modeling_id" INTEGER NOT NULL,
    "job_id" BIGINT NOT NULL
);

-----------------------------------------------------------
-- Relations
ALTER TABLE
    "modeling_jobs" ADD CONSTRAINT "modeling_jobs_modeling_id_foreign" FOREIGN KEY("modeling_id") REFERENCES "modeling"("id") 
    ON UPDATE CASCADE
    ON DELETE CASCADE;
ALTER TABLE
    "modeling_jobs" ADD CONSTRAINT "modeling_jobs_job_id_foreign" FOREIGN KEY("job_id") REFERENCES "job_info"("id")
    ON UPDATE CASCADE
    ON DELETE CASCADE;
ALTER TABLE
    "error_log" ADD CONSTRAINT "error_log_job_id_foreign" FOREIGN KEY("job_id") REFERENCES "job_info"("id")
    ON UPDATE CASCADE
    ON DELETE CASCADE;