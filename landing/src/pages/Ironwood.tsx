import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import logoMarkUrl from "../assets/logo-mark-clean.png";
import {
  IRONWOOD_CIPHERSCAN,
  IRONWOOD_ZODL,
  PATHS,
  REPO_RELEASES,
  ZIP_318,
} from "../lib/links";
import {
  emptyIronwoodStats,
  fetchIronwoodNetworkStats,
  formatDuration,
  formatPct,
  formatZec,
  type IronwoodNetworkStats,
  NU6_3_MAINNET_ACTIVATION_HEIGHT,
  zip318WindowAtTip,
} from "../lib/ironwoodStats";

/** Matches desktop-client primary / dark shell. */
const DESKTOP = {
  bg: "#0a0a0a",
  primary: "#d4af37",
  primarySoft: "rgba(212, 175, 55, 0.14)",
} as const;

const STEPS = [
  "Sync to tip",
  "Plan ZIP 318 schedule",
  "Split if needed",
  "Migrate in bucket window",
  "Broadcast privately",
] as const;

function Atmosphere({ splash }: { splash?: boolean }) {
  return (
    <div
      className="pointer-events-none fixed inset-0 overflow-hidden"
      aria-hidden
    >
      {/* Base desktop dark */}
      <div className="absolute inset-0" style={{ background: DESKTOP.bg }} />

      {/* Desktop-style watermark: Nozy logo only (subtle, not a cyber scene). */}
      <img
        src={logoMarkUrl}
        alt=""
        className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-[42%] object-contain"
        style={{
          height: splash ? "min(78vh, 52rem)" : "min(70vh, 48rem)",
          width: "auto",
          opacity: splash ? 0.08 : 0.055,
        }}
      />

      {/* Gold wash (desktop primary) */}
      <div
        className="absolute left-1/2 top-[-10%] h-[36rem] w-[36rem] -translate-x-1/2 rounded-full blur-3xl"
        style={{ background: DESKTOP.primarySoft }}
      />
      <div className="absolute bottom-0 right-0 h-80 w-80 rounded-full bg-emerald-500/[0.04] blur-3xl" />

      {/* Readability veil */}
      <div
        className="absolute inset-0"
        style={{
          background: splash
            ? "radial-gradient(ellipse at center, transparent 0%, rgba(10,10,10,0.55) 55%, rgba(10,10,10,0.92) 100%)"
            : "linear-gradient(180deg, rgba(10,10,10,0.55) 0%, rgba(10,10,10,0.78) 40%, rgba(10,10,10,0.92) 100%)",
        }}
      />
    </div>
  );
}

function PoolBar({
  label,
  value,
  total,
  tone,
}: {
  label: string;
  value: number | null;
  total: number;
  tone: "orchard" | "ironwood";
}) {
  const pct =
    value != null && total > 0 ? Math.min(100, (value / total) * 100) : 0;
  return (
    <div className="space-y-2">
      <div className="flex items-baseline justify-between gap-3">
        <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500">
          {label}
        </span>
        <span className="text-lg font-semibold tabular-nums text-zinc-100">
          {formatZec(value)}{" "}
          <span className="text-xs font-normal text-zinc-500">ZEC</span>
        </span>
      </div>
      <div className="h-1.5 overflow-hidden rounded-full bg-zinc-800/80">
        <div
          className={`h-full rounded-full transition-[width] duration-1000 ease-out ${
            tone === "ironwood"
              ? "bg-gradient-to-r from-primary-600 via-primary to-yellow-300"
              : "bg-zinc-500"
          }`}
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  );
}

const Ironwood = () => {
  const [stats, setStats] = useState<IronwoodNetworkStats>(() =>
    emptyIronwoodStats()
  );
  const [phase, setPhase] = useState<"splash" | "ready">("splash");
  const [splashDone, setSplashDone] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const minSplash = window.setTimeout(() => setSplashDone(true), 1200);

    const load = async () => {
      const next = await fetchIronwoodNetworkStats();
      if (!cancelled) setStats(next);
    };
    void load();
    const id = window.setInterval(load, 60_000);
    return () => {
      cancelled = true;
      window.clearTimeout(minSplash);
      window.clearInterval(id);
    };
  }, []);

  useEffect(() => {
    if (splashDone) setPhase("ready");
  }, [splashDone]);

  const zipWindow = stats.tip != null ? zip318WindowAtTip(stats.tip) : null;
  const totalPools =
    (stats.orchardZec ?? 0) + (stats.ironwoodZec ?? 0) || 1;

  if (phase === "splash") {
    return (
      <div
        className="relative flex min-h-screen flex-col items-center justify-center px-6 text-center"
        style={{ background: DESKTOP.bg }}
      >
        <Atmosphere splash />
        <div className="relative z-10">
          <img
            src={logoMarkUrl}
            alt=""
            className="mx-auto mb-8 h-14 w-14 object-contain drop-shadow-[0_0_24px_rgba(212,175,55,0.35)]"
          />
          <h1 className="text-2xl font-semibold tracking-[0.12em] text-white sm:text-3xl">
            <span className="font-bold">IRONWOOD</span>{" "}
            <span className="font-normal text-zinc-400">MIGRATION</span>
          </h1>
          <p className="mt-4 animate-pulse text-sm text-zinc-400">
            Connecting to verified chain data…
          </p>
          <div
            className="mx-auto mt-8 h-px w-40"
            style={{ background: "rgba(212,175,55,0.35)" }}
          />
          <p className="mt-6 text-[11px] uppercase tracking-[0.2em] text-zinc-500">
            NozyWallet
          </p>
        </div>
      </div>
    );
  }

  return (
    <div
      className="relative min-h-screen text-zinc-100 selection:bg-primary/30 selection:text-primary-100"
      style={{ background: DESKTOP.bg }}
    >
      <Atmosphere />

      <header className="relative z-10 border-b border-white/5 bg-black/20 backdrop-blur-md">
        <div className="mx-auto flex max-w-6xl items-center justify-between px-6 py-5">
          <Link to="/" className="flex items-center gap-3">
            <img
              src={logoMarkUrl}
              alt="NozyWallet"
              className="h-8 w-8 object-contain"
            />
            <span className="text-sm font-semibold tracking-wide text-zinc-200">
              NozyWallet
            </span>
          </Link>
          <div className="flex items-center gap-6 text-xs font-medium uppercase tracking-[0.16em] text-zinc-500">
            <span
              className={
                stats.available ? "text-primary" : "text-zinc-600"
              }
            >
              {stats.available
                ? `Live · ${stats.tip?.toLocaleString()}`
                : "Offline"}
            </span>
            <a
              href={REPO_RELEASES}
              className="text-zinc-400 transition hover:text-primary"
            >
              CLI
            </a>
            <Link
              to="/"
              className="text-zinc-400 transition hover:text-primary"
            >
              Home
            </Link>
          </div>
        </div>
      </header>

      <main className="relative z-10 mx-auto max-w-6xl px-6 pb-24 pt-14">
        <div className="mb-14 text-center">
          <p
            className="mb-3 text-[11px] font-semibold uppercase tracking-[0.28em]"
            style={{ color: DESKTOP.primary }}
          >
            Ironwood migration
          </p>
          <h1 className="text-4xl font-bold tracking-tight text-white sm:text-5xl lg:text-6xl">
            Orchard → Ironwood
          </h1>
          <p className="mx-auto mt-4 max-w-xl text-base text-zinc-400">
            Network supply crossing the NU6.3 turnstile — and how to migrate with
            Nozy.
          </p>
        </div>

        <section className="mb-16 rounded-2xl border border-white/5 bg-gray-900/40 p-8 text-center shadow-xl shadow-black/40 backdrop-blur-md sm:p-10">
          <p className="text-[11px] font-semibold uppercase tracking-[0.22em] text-zinc-500">
            Migrated into Ironwood
          </p>
          <p className="mt-3 text-6xl font-bold tabular-nums tracking-tight text-white transition-all duration-700 sm:text-7xl lg:text-8xl">
            {formatPct(stats.migratedPct)}
          </p>
          <div className="mx-auto mt-8 h-2 max-w-md overflow-hidden rounded-full bg-zinc-800/90">
            <div
              className="h-full rounded-full bg-gradient-to-r from-primary-700 via-primary to-yellow-300 transition-[width] duration-1000 ease-out"
              style={{
                width: `${
                  stats.migratedPct != null
                    ? Math.min(100, Math.max(0, stats.migratedPct))
                    : 0
                }%`,
              }}
            />
          </div>
          {!stats.available ? (
            <p className="mx-auto mt-6 max-w-lg text-sm text-zinc-500">
              Live pool RPC not configured here. Set{" "}
              <code className="text-zinc-400">ZEBRA_RPC_URL</code> on Vercel (or
              local Vite) for chain values — or track{" "}
              <a
                href={IRONWOOD_ZODL}
                className="text-primary underline-offset-2 hover:underline"
                target="_blank"
                rel="noopener noreferrer"
              >
                ZODL
              </a>{" "}
              /{" "}
              <a
                href={IRONWOOD_CIPHERSCAN}
                className="text-primary underline-offset-2 hover:underline"
                target="_blank"
                rel="noopener noreferrer"
              >
                CipherScan
              </a>
              .
            </p>
          ) : null}
        </section>

        <section className="mb-16 grid gap-6 lg:grid-cols-2">
          <div className="space-y-8 rounded-2xl border border-white/5 bg-gray-900/35 p-8 backdrop-blur-md">
            <h2 className="text-[11px] font-semibold uppercase tracking-[0.22em] text-zinc-500">
              Pool supply
            </h2>
            <PoolBar
              label="Orchard (sealed)"
              value={stats.orchardZec}
              total={totalPools}
              tone="orchard"
            />
            <PoolBar
              label="Ironwood"
              value={stats.ironwoodZec}
              total={totalPools}
              tone="ironwood"
            />
          </div>

          <div className="grid grid-cols-2 gap-8 rounded-2xl border border-white/5 bg-gray-900/35 p-8 backdrop-blur-md sm:gap-10">
            <div>
              <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500">
                Activation
              </p>
              <p className="mt-2 text-2xl font-semibold tabular-nums text-white">
                {NU6_3_MAINNET_ACTIVATION_HEIGHT.toLocaleString()}
              </p>
              <p className="mt-1 text-sm text-zinc-500">
                {stats.ironwoodActive
                  ? `${(stats.blocksSinceActivation ?? 0).toLocaleString()} blocks since`
                  : "Pre-activation"}
              </p>
            </div>
            <div>
              <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500">
                Chain tip
              </p>
              <p className="mt-2 text-2xl font-semibold tabular-nums text-white">
                {stats.tip != null ? stats.tip.toLocaleString() : "—"}
              </p>
              <p className="mt-1 text-sm text-zinc-500">
                {stats.fetchedAt
                  ? new Date(stats.fetchedAt).toLocaleTimeString()
                  : "—"}
              </p>
            </div>
            <div>
              <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500">
                ZIP 318 bucket
              </p>
              <p className="mt-2 text-2xl font-semibold tabular-nums text-white">
                {zipWindow ? zipWindow.currentBucket.toLocaleString() : "—"}
              </p>
              <p className="mt-1 text-sm text-zinc-500">256-block windows</p>
            </div>
            <div>
              <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500">
                Next window
              </p>
              <p className="mt-2 text-2xl font-semibold tabular-nums text-white">
                {zipWindow ? zipWindow.nextBucket.toLocaleString() : "—"}
              </p>
              <p className="mt-1 text-sm text-zinc-500">
                {zipWindow
                  ? `${zipWindow.blocksUntilNext.toLocaleString()} blk · ${formatDuration(zipWindow.etaSeconds)}`
                  : "—"}
              </p>
            </div>
          </div>
        </section>

        <section className="rounded-2xl border border-white/5 bg-gray-900/35 p-8 backdrop-blur-md sm:p-10">
          <div className="mb-10 flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
            <div>
              <h2 className="text-2xl font-bold tracking-tight text-white">
                Migrate with Nozy
              </h2>
              <p className="mt-2 max-w-lg text-sm text-zinc-400">
                CLI-first turnstile path. Desktop and extension companion share
                the same profile on your local API.
              </p>
            </div>
            <div className="flex flex-wrap gap-3">
              <a
                href={REPO_RELEASES}
                className="rounded-full bg-primary px-6 py-2.5 text-sm font-semibold text-black shadow-lg shadow-primary/20 transition hover:bg-primary-300"
              >
                Download CLI
              </a>
              <a
                href={PATHS.ironwoodReadiness}
                target="_blank"
                rel="noopener noreferrer"
                className="rounded-full border border-white/10 bg-white/5 px-6 py-2.5 text-sm font-medium text-zinc-200 transition hover:border-primary/40 hover:text-primary"
              >
                Readiness guide
              </a>
            </div>
          </div>

          <ol className="mb-10 flex flex-wrap gap-3">
            {STEPS.map((step, i) => (
              <li
                key={step}
                className="flex items-center gap-2 text-sm text-zinc-400"
              >
                <span className="flex h-6 w-6 items-center justify-center rounded-full border border-primary/30 bg-primary/10 text-[11px] font-bold text-primary">
                  {i + 1}
                </span>
                {step}
                {i < STEPS.length - 1 ? (
                  <span className="ml-1 hidden text-zinc-700 sm:inline">→</span>
                ) : null}
              </li>
            ))}
          </ol>

          <pre className="overflow-x-auto rounded-xl border border-white/5 bg-black/50 px-5 py-4 text-left text-[13px] leading-relaxed text-primary-100/85">
{`$env:ZEBRA_RPC_URL = "http://<zebrad>:8232"
nozy sync --to-tip
nozy ironwood plan --save
nozy ironwood preflight
nozy ironwood migrate    # when tip >= next bucket
nozy ironwood broadcast  # local Zebrad / Tor / Nym`}
          </pre>
          <p className="mt-4 text-xs text-zinc-600">
            Turnstile amounts are public. Broadcast only over a privacy path —
            never clearnet lightwalletd for migration submit. Spec:{" "}
            <a
              href={ZIP_318}
              className="text-zinc-400 hover:text-primary"
              target="_blank"
              rel="noopener noreferrer"
            >
              ZIP 318
            </a>
            .
          </p>
        </section>
      </main>

      <footer className="relative z-10 border-t border-white/5 bg-black/30 py-8 backdrop-blur-sm">
        <div className="mx-auto flex max-w-6xl flex-col gap-4 px-6 text-xs text-zinc-600 sm:flex-row sm:items-center sm:justify-between">
          <p>NozyWallet · Ironwood supply dashboard</p>
          <div className="flex flex-wrap gap-5">
            <a
              href={IRONWOOD_ZODL}
              className="hover:text-primary"
              target="_blank"
              rel="noopener noreferrer"
            >
              ZODL
            </a>
            <a
              href={IRONWOOD_CIPHERSCAN}
              className="hover:text-primary"
              target="_blank"
              rel="noopener noreferrer"
            >
              CipherScan
            </a>
            <Link to="/mobile" className="hover:text-primary">
              Mobile
            </Link>
          </div>
        </div>
      </footer>
    </div>
  );
};

export default Ironwood;
