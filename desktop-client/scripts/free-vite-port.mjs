/**
 * Free port 5173 before starting Vite (common when a prior `tauri dev` or `npm run dev` was not stopped).
 */
import { execSync } from "node:child_process";

const PORT = 5173;

function freePortWindows() {
  try {
    const output = execSync(`netstat -ano | findstr :${PORT}`, { encoding: "utf8" });
    const pids = new Set();
    for (const line of output.split("\n")) {
      if (!line.includes("LISTENING")) continue;
      const pid = line.trim().split(/\s+/).pop();
      if (pid && pid !== "0") pids.add(pid);
    }
    for (const pid of pids) {
      try {
        execSync(`taskkill /F /PID ${pid}`, { stdio: "ignore" });
        console.log(`[free-vite-port] stopped PID ${pid} on port ${PORT}`);
      } catch {
        // Process may have already exited.
      }
    }
  } catch {
    // Port not in use.
  }
}

function freePortUnix() {
  try {
    execSync(`lsof -ti :${PORT} | xargs kill -9 2>/dev/null || true`, {
      shell: true,
      stdio: "ignore",
    });
  } catch {
    // Port not in use.
  }
}

if (process.platform === "win32") {
  freePortWindows();
} else {
  freePortUnix();
}
