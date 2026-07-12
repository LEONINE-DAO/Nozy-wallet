# Nozy Lite benches — measured size & start

**Status:** Living measurements for **Nozy Lite (CLI)** vs desktop  
**Audience:** Operators and release notes  
**Related:** [`NOZY_LITE.md`](NOZY_LITE.md) · [`ZEBRA_DEV_CLI_POSITIONING.md`](../ZEBRA_DEV_CLI_POSITIONING.md)  
**Reproduce:** [`scripts/nozy-lite-bench.ps1`](../../scripts/nozy-lite-bench.ps1)

---

## Rules

1. **Publish only measured numbers** (or mark a cell `— not measured`).
2. Do **not** invent idle RSS or peak sync RSS without a tool run on that machine.
3. CLI size is the **`nozy` release binary** (full Orchard/Halo2 stack in the default artifact).
4. Record **OS, CPU, build profile, date**.

---

## Latest run

| Field | Value |
|-------|--------|
| Date | 2026-07-11 |
| Host | Windows 10 (build 26200), PowerShell |
| Build | `cargo build -p nozy --bin nozy --release` |
| CLI artifact | `nozy.exe` (Nozy Lite) |

### Install / binary size

| Surface | Artifact | Size (bytes) | Size (MiB) |
|---------|----------|--------------|------------|
| **Nozy Lite (CLI)** | `nozy.exe` release | 21 638 144 | **20.6** |
| Desktop (Tauri) | `NozyWallet` release | — | **not measured this run** (no release build present) |

### Cold start (`nozy --version`)

| Metric | ms | Notes |
|--------|-----|--------|
| First sample after build | 15 | Process exit; AV may inflate cold starts on other hosts |
| Immediate re-run | 12 | Warm OS cache |

### Ops check (`nozy health --json`)

| Metric | Value |
|--------|--------|
| Wall time | ~46 ms (this host; Zebra reachable, exit 0) |
| Exit code | 0 |

### Idle RSS / sync peak RSS

| Metric | Value |
|--------|--------|
| CLI idle RSS | — **not measured** (use Task Manager / `Get-Process` while `nozy tui` or a long `sync` runs) |
| CLI `sync` peak RSS | — **not measured** (honest: depends on tip gap + proving) |
| Desktop idle RSS | — **not measured** |

---

## How to re-measure

```powershell
# From repo root
.\scripts\nozy-lite-bench.ps1
# Optional desktop path after `npm run tauri build`:
.\scripts\nozy-lite-bench.ps1 -DesktopExe "desktop-client\src-tauri\target\release\NozyWallet.exe"
```

Paste the script’s markdown table into this file (replace “Latest run”).

---

## How to read these numbers

| Metric | Meaning |
|--------|---------|
| Binary size | On-disk `nozy` release artifact (~21 MiB on this host) — includes proving stack |
| Cold / warm start | `nozy --version` wall time |
| Health | `nozy health --json` for ops loops — see [`NOZY_LITE.md`](NOZY_LITE.md) |

---

## AI disclosure

Bench table drafted with Cursor Agent from a local release build on 2026-07-11. Re-run the script before quoting in forum posts.
