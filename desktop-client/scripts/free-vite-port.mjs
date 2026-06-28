/**
 * Free port 5173 before starting Vite (prior `tauri dev` / `npm run dev` often leaves node listening).
 */
import { execSync } from "node:child_process";
import net from "node:net";

const PORT = 5173;
const HOSTS = ["127.0.0.1", "::1", "localhost"];
const MAX_ATTEMPTS = 12;
const RETRY_MS = 250;

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function portFreeOn(host) {
  return new Promise((resolve) => {
    const server = net.createServer();
    server.once("error", () => resolve(false));
    server.once("listening", () => {
      server.close(() => resolve(true));
    });
    server.listen(PORT, host);
  });
}

async function isPortFree() {
  for (const host of HOSTS) {
    if (!(await portFreeOn(host))) {
      return false;
    }
  }
  return true;
}

function killPortWindows() {
  try {
    const output = execSync(
      `powershell -NoProfile -Command "(Get-NetTCPConnection -LocalPort ${PORT} -State Listen -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess -Unique) -join ' '"`,
      { encoding: "utf8" },
    );
    for (const pid of output.trim().split(/\s+/).filter(Boolean)) {
      try {
        execSync(`taskkill /F /PID ${pid}`, { stdio: "ignore" });
        console.log(`[free-vite-port] stopped PID ${pid} on port ${PORT}`);
      } catch {
        // Already exited.
      }
    }
  } catch {
    // Fallback when Get-NetTCPConnection is unavailable.
    try {
      const output = execSync(`netstat -ano | findstr :${PORT}`, { encoding: "utf8" });
      const pids = new Set();
      for (const line of output.split("\n")) {
        if (!line.includes("LISTENING")) continue;
        const pid = line.trim().split(/\s+/).pop();
        if (pid && pid !== "0") pids.add(pid);
      }
      for (const pid of pids) {
        execSync(`taskkill /F /PID ${pid}`, { stdio: "ignore" });
        console.log(`[free-vite-port] stopped PID ${pid} on port ${PORT}`);
      }
    } catch {
      // Port not in use.
    }
  }
}

function killPortUnix() {
  try {
    execSync(`lsof -ti :${PORT} | xargs kill -9 2>/dev/null || true`, {
      shell: true,
      stdio: "ignore",
    });
  } catch {
    // Port not in use.
  }
}

async function ensurePortFree() {
  for (let attempt = 1; attempt <= MAX_ATTEMPTS; attempt++) {
    if (process.platform === "win32") {
      killPortWindows();
    } else {
      killPortUnix();
    }

    if (await isPortFree()) {
      return;
    }

    if (attempt < MAX_ATTEMPTS) {
      await sleep(RETRY_MS);
    }
  }

  console.error(
    `[free-vite-port] port ${PORT} is still in use. Stop the other dev session (Ctrl+C in its terminal) or run:\n` +
      `  netstat -ano | findstr :${PORT}\n` +
      `  taskkill /F /PID <pid>`,
  );
  process.exit(1);
}

await ensurePortFree();
