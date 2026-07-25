import {
  DOWNLOAD_URLS,
  DESKTOP_DOWNLOAD_URLS,
  REPO_RELEASES_LATEST,
} from "../lib/downloads";
import { PATHS } from "../lib/links";

const card =
  "rounded-2xl border border-zinc-200 bg-white p-6 shadow-sm hover:border-yellow-300/70 transition-colors text-left";

const DownloadSection = () => {
  return (
    <section
      id="download"
      className="py-20 bg-zinc-50 border-t border-zinc-200 scroll-mt-24"
    >
      <div className="max-w-7xl mx-auto px-6">
        <h2 className="text-3xl font-bold text-zinc-900 mb-3 text-center">
          Download NozyWallet
        </h2>
        <p className="text-zinc-600 text-center max-w-2xl mx-auto mb-12">
          <strong>CLI Lite</strong> is production-ready for mainnet.{" "}
          <strong>Browser extension</strong> (Sweet chili) is in public beta.{" "}
          <strong>Desktop beta</strong> is available for early testers.{" "}
          <strong>Mobile beta</strong> is coming soon.
        </p>

        <div className={`${card} mb-6 border-yellow-200 bg-yellow-50/30`}>
          <div className="flex flex-wrap items-center gap-2 mb-2">
            <h3 className="font-semibold text-lg text-zinc-900">
              CLI Lite — Teriyaki Hot
            </h3>
            <span className="text-xs font-semibold uppercase tracking-wide px-2 py-0.5 rounded-full border bg-emerald-500/10 text-emerald-800 border-emerald-500/25">
              Mainnet ready
            </span>
          </div>
          <p className="text-sm text-zinc-600 mb-4">
            Orchard-first <code className="text-xs bg-zinc-100 px-1 rounded">nozy</code> binary.
            Pair with your own <strong>zebrad</strong> or <strong>Zakura</strong> + <strong>lightwalletd</strong>.
            On Linux / macOS run{" "}
            <code className="text-xs bg-zinc-100 px-1 rounded">chmod +x</code> after download.
          </p>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 mb-3">
            <a
              href={DOWNLOAD_URLS.cliWindows}
              className="text-center rounded-lg bg-yellow-500 hover:bg-yellow-400 text-white font-semibold py-2.5 text-sm"
            >
              Windows
            </a>
            <a
              href={DOWNLOAD_URLS.cliLinux}
              className="text-center rounded-lg bg-yellow-500 hover:bg-yellow-400 text-white font-semibold py-2.5 text-sm"
            >
              Linux
            </a>
            <a
              href={DOWNLOAD_URLS.cliMacArm}
              className="text-center rounded-lg bg-yellow-500 hover:bg-yellow-400 text-white font-semibold py-2.5 text-sm"
            >
              macOS ARM
            </a>
            <a
              href={DOWNLOAD_URLS.cliMacIntel}
              className="text-center rounded-lg bg-yellow-500 hover:bg-yellow-400 text-white font-semibold py-2.5 text-sm"
            >
              macOS Intel
            </a>
          </div>
          <p className="text-xs text-zinc-500">
            Verify with{" "}
            <a href={DOWNLOAD_URLS.hashes} className="text-yellow-700 hover:underline font-medium">
              HASHES.txt
            </a>{" "}
            on the release page.
          </p>
        </div>

        <div className={`${card} mb-6 border-amber-200 bg-amber-50/40`}>
          <div className="flex flex-wrap items-center gap-2 mb-2">
            <h3 className="font-semibold text-lg text-zinc-900">
              Desktop — Hot Lemon beta.2
            </h3>
            <span className="text-xs font-semibold uppercase tracking-wide px-2 py-0.5 rounded-full border bg-amber-500/10 text-amber-900 border-amber-500/25">
              Beta · Ironwood WIP
            </span>
          </div>
          <p className="text-sm text-zinc-600 mb-4">
            Tauri GUI for operators and early testers. GA waits until Ironwood is official.
            Prefer CLI Lite for production mainnet today.
          </p>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 mb-3">
            <a
              href={DESKTOP_DOWNLOAD_URLS.windows}
              className="text-center rounded-lg bg-zinc-900 hover:bg-zinc-800 text-white font-semibold py-2.5 text-sm"
            >
              Windows installer
            </a>
            <a
              href={DESKTOP_DOWNLOAD_URLS.macArm}
              className="text-center rounded-lg bg-zinc-900 hover:bg-zinc-800 text-white font-semibold py-2.5 text-sm"
            >
              macOS ARM
            </a>
            <a
              href={DESKTOP_DOWNLOAD_URLS.linux}
              className="text-center rounded-lg bg-zinc-900 hover:bg-zinc-800 text-white font-semibold py-2.5 text-sm"
            >
              Linux x86_64
            </a>
          </div>
          <a
            href={DESKTOP_DOWNLOAD_URLS.releasePage}
            target="_blank"
            rel="noopener noreferrer"
            className="text-xs font-medium text-yellow-700 hover:underline"
          >
            Desktop beta release notes &amp; checksums →
          </a>
        </div>

        <div className="grid md:grid-cols-2 gap-6 mb-6">
          <div className={`${card} border-amber-200 bg-amber-50/40`}>
            <div className="flex flex-wrap items-center gap-2 mb-2">
              <h3 className="font-semibold text-lg text-zinc-900">
                Browser extension — Sweet chili
              </h3>
              <span className="text-xs font-semibold uppercase tracking-wide px-2 py-0.5 rounded-full border bg-amber-500/10 text-amber-900 border-amber-500/25">
                Public beta · 0.1.7
              </span>
            </div>
            <p className="text-sm text-zinc-600 mb-4">
              Chrome, Brave, and Edge (load unpacked). Optional local{" "}
              <code className="text-xs bg-zinc-100 px-1 rounded">nozywallet-api</code>{" "}
              companion for lightwalletd sync. Store listing in progress.
            </p>
            <a
              href={PATHS.extensionRelease}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex w-full items-center justify-center rounded-xl bg-zinc-900 hover:bg-zinc-800 text-white font-semibold py-3 transition-colors mb-2"
            >
              Download extension zip →
            </a>
            <a
              href={PATHS.extension}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex w-full items-center justify-center text-sm font-medium text-yellow-700 hover:underline"
            >
              Install guide &amp; companion docs
            </a>
          </div>

          <div className={`${card} border-dashed border-zinc-300 bg-zinc-50/80`}>
            <div className="flex flex-wrap items-center gap-2 mb-2">
              <h3 className="font-semibold text-lg text-zinc-900">
                iPhone &amp; Android
              </h3>
              <span className="text-xs font-semibold uppercase tracking-wide px-2 py-0.5 rounded-full border bg-zinc-100 text-zinc-600 border-zinc-200">
                Beta coming soon
              </span>
            </div>
            <p className="text-sm text-zinc-600 mb-4">
              Expo companion for App Store / Play is in store prep — not listed yet.
              Until then, use CLI Lite, Desktop beta, or the browser extension.
            </p>
            <a
              href={PATHS.mobilePage}
              className="inline-flex w-full items-center justify-center rounded-xl border border-zinc-300 bg-white hover:bg-zinc-100 text-zinc-800 font-medium py-3 transition-colors"
            >
              Mobile preview page →
            </a>
          </div>
        </div>

        <p className="text-center text-sm text-zinc-500 mt-10">
          <a
            href={REPO_RELEASES_LATEST}
            target="_blank"
            rel="noopener noreferrer"
            className="text-yellow-700 font-medium hover:underline"
          >
            Browse all release assets &amp; checksums →
          </a>
        </p>
      </div>
    </section>
  );
};

export default DownloadSection;
