import { execFileSync } from "node:child_process";
import { resolve } from "node:path";

export function runPlaywrightSql(sql: string) {
  const container = process.env.PLAYWRIGHT_POSTGRES_CONTAINER;
  const database = process.env.PLAYWRIGHT_POSTGRES_DATABASE ?? "tessara";
  const user = process.env.PLAYWRIGHT_POSTGRES_USER ?? "tessara";
  const args = container
    ? ["exec", "-i", container, "psql", "-v", "ON_ERROR_STOP=1", "-U", user, "-d", database]
    : [
        "compose",
        "exec",
        "-T",
        "postgres",
        "psql",
        "-v",
        "ON_ERROR_STOP=1",
        "-U",
        user,
        "-d",
        database,
      ];

  execFileSync("docker", args, {
    cwd: resolve(process.cwd(), ".."),
    input: sql,
    stdio: ["pipe", "pipe", "pipe"],
  });
}
