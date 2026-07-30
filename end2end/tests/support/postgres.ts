import { execFileSync } from "node:child_process";
import { resolve } from "node:path";

export function runPlaywrightSql(sql: string) {
  const configuredContainer = process.env.PLAYWRIGHT_POSTGRES_CONTAINER;
  const configuredDatabase = process.env.PLAYWRIGHT_POSTGRES_DATABASE;
  const configuredUser = process.env.PLAYWRIGHT_POSTGRES_USER;
  if (process.env.TESSARA_PLAYWRIGHT_ACCEPTANCE === "1") {
    if (
      !configuredContainer ||
      !process.env.PLAYWRIGHT_POSTGRES_DATABASE ||
      !process.env.PLAYWRIGHT_POSTGRES_USER
    ) {
      throw new Error(
        "Playwright acceptance requires an exact PostgreSQL container, database, and user binding.",
      );
    }
    if (
      !/^[0-9a-f]{64}$/.test(configuredContainer) ||
      !/^[A-Za-z_][A-Za-z0-9_-]*$/.test(configuredDatabase) ||
      !/^[A-Za-z_][A-Za-z0-9_-]*$/.test(configuredUser)
    ) {
      throw new Error("Playwright acceptance PostgreSQL binding is malformed.");
    }
  }
  const container = configuredContainer ?? discoverSinglePlaywrightPostgres();
  const user =
    configuredUser ??
    (configuredContainer
      ? "tessara"
      : discoverPostgresBootstrapUser(container));
  const database =
    configuredDatabase ??
    (configuredContainer
      ? "tessara"
      : discoverCoreDatabase(container, user));
  const args = [
    "exec",
    "-i",
    container,
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

function discoverSinglePlaywrightPostgres() {
  const output = execFileSync(
    "docker",
    [
      "ps",
      "--filter",
      "label=com.docker.compose.service=postgres",
      "--format",
      "{{.ID}}",
    ],
    {
      cwd: resolve(process.cwd(), ".."),
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  const containers = output
    .split(/\r?\n/)
    .map((value) => value.trim())
    .filter(Boolean);
  if (containers.length !== 1 || !/^[0-9a-f]{12,64}$/.test(containers[0])) {
    throw new Error(
      "A direct Playwright run requires exactly one running Compose PostgreSQL service. " +
        "Set PLAYWRIGHT_POSTGRES_CONTAINER, PLAYWRIGHT_POSTGRES_DATABASE, and " +
        "PLAYWRIGHT_POSTGRES_USER when multiple stacks are running.",
    );
  }
  return containers[0];
}

function discoverPostgresBootstrapUser(container: string) {
  const output = execFileSync(
    "docker",
    [
      "inspect",
      "--format",
      "{{range .Config.Env}}{{println .}}{{end}}",
      container,
    ],
    {
      cwd: resolve(process.cwd(), ".."),
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  const user = output
    .split(/\r?\n/)
    .find((value) => value.startsWith("POSTGRES_USER="))
    ?.slice("POSTGRES_USER=".length)
    .trim();
  if (!user || !/^[A-Za-z_][A-Za-z0-9_-]*$/.test(user)) {
    throw new Error(
      "The discovered Compose PostgreSQL service does not declare a valid POSTGRES_USER. " +
        "Set PLAYWRIGHT_POSTGRES_USER explicitly.",
    );
  }
  return user;
}

function discoverCoreDatabase(container: string, user: string) {
  const output = execFileSync(
    "docker",
    [
      "exec",
      "-i",
      container,
      "psql",
      "-At",
      "-U",
      user,
      "-d",
      "postgres",
      "-c",
      "SELECT datname FROM pg_database WHERE datistemplate = false AND datname ~ '(^|_)core$' ORDER BY datname",
    ],
    {
      cwd: resolve(process.cwd(), ".."),
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  const databases = output
    .split(/\r?\n/)
    .map((value) => value.trim())
    .filter(Boolean);
  if (
    databases.length !== 1 ||
    !/^[A-Za-z_][A-Za-z0-9_-]*$/.test(databases[0])
  ) {
    throw new Error(
      "The discovered Compose PostgreSQL service does not expose exactly one Core database. " +
        "Set PLAYWRIGHT_POSTGRES_DATABASE explicitly.",
    );
  }
  return databases[0];
}
