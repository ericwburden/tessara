import { execFileSync } from "node:child_process";
import { resolve } from "node:path";

export function runPlaywrightSql(sql: string) {
  const container = process.env.PLAYWRIGHT_POSTGRES_CONTAINER;
  const database = process.env.PLAYWRIGHT_POSTGRES_DATABASE ?? "tessara";
  const user = process.env.PLAYWRIGHT_POSTGRES_USER ?? "tessara";
  if (process.env.TESSARA_PLAYWRIGHT_ACCEPTANCE === "1") {
    if (
      !container ||
      !process.env.PLAYWRIGHT_POSTGRES_DATABASE ||
      !process.env.PLAYWRIGHT_POSTGRES_USER
    ) {
      throw new Error(
        "Playwright acceptance requires an exact PostgreSQL container, database, and user binding.",
      );
    }
    if (
      !/^[0-9a-f]{64}$/.test(container) ||
      !/^[A-Za-z_][A-Za-z0-9_-]*$/.test(database) ||
      !/^[A-Za-z_][A-Za-z0-9_-]*$/.test(user)
    ) {
      throw new Error("Playwright acceptance PostgreSQL binding is malformed.");
    }
  }
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
