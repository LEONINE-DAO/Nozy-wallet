import { DOWNLOAD_URLS, REPO_RELEASES_LATEST } from "../lib/downloads";

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
          <strong>Production-ready today:</strong> the{" "}
          <strong className="text-zinc-800">CLI wallet</strong> (<code className="text-xs bg-zinc-100 px-1 rounded">nozy</code>
          ). Run it with your own <strong>zebrad</strong> and <strong>lightwalletd</strong>.
          Desktop, browser extension, and mobile apps are in active development — contributors can build from source on GitHub.
        </p>

        <div className={`${card} mb-6 border-yellow-200 bg-yellow-50/30`}>
          <h3 className="font-semibold text-lg text-zinc-900 mb-1">
            Command-line wallet (production)
          </h3>
          <p className="text-sm text-zinc-600 mb-4">
            Single binary per OS. On Linux / macOS run{" "}
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

        <div className="grid md:grid-cols-2 gap-6 mb-6">
          <div className={`${card} border-dashed border-zinc-300 bg-zinc-50/80`}>
            <h3 className="font-semibold text-lg text-zinc-900 mb-1">
              Desktop app (in development)
            </h3>
            <p className="text-sm text-zinc-600 mb-4">
              Tauri GUI under <code className="text-xs bg-zinc-100 px-1 rounded">desktop-client/</code>.
              Not promoted for end-user download until production-ready. Use the CLI for mainnet today.
            </p>
            <a
              href="https://github.com/LEONINE-DAO/Nozy-wallet/tree/master/desktop-client"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex w-full items-center justify-center rounded-xl border border-zinc-300 bg-white hover:bg-zinc-100 text-zinc-800 font-medium py-3 transition-colors"
            >
              Build from source on GitHub →
            </a>
          </div>

          <div className={`${card} border-dashed border-zinc-300 bg-zinc-50/80`}>
            <h3 className="font-semibold text-lg text-zinc-900 mb-1">
              Browser extension (in development)
            </h3>
            <p className="text-sm text-zinc-600 mb-4">
              MV3 + WASM wallet in <code className="text-xs bg-zinc-100 px-1 rounded">browser-extension/</code>.
              Requires companion API for compact sync — contributor preview only for now.
            </p>
            <a
              href="https://github.com/LEONINE-DAO/Nozy-wallet/tree/master/browser-extension"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex w-full items-center justify-center rounded-xl border border-zinc-300 bg-white hover:bg-zinc-100 text-zinc-800 font-medium py-3 transition-colors"
            >
              Extension docs on GitHub →
            </a>
          </div>
        </div>

        <div className={`${card} mb-6 border-dashed border-2 border-zinc-300 bg-zinc-50/80`}>
          <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-4">
            <div>
              <h3 className="font-semibold text-lg text-zinc-900 mb-1">
                iPhone &amp; Android (coming soon)
              </h3>
              <p className="text-sm text-zinc-600 max-w-xl">
                Native apps for <strong className="text-zinc-800">App Store</strong> and{" "}
                <strong className="text-zinc-800">Google Play</strong> are on the roadmap. Until then,
                use the <strong>CLI</strong> on a computer you control.
              </p>
            </div>
            <div className="flex flex-col sm:items-end gap-2 shrink-0">
              <span
                className="inline-flex items-center justify-center rounded-xl border border-zinc-300 bg-white px-4 py-2.5 text-sm font-medium text-zinc-400 cursor-not-allowed"
                title="Not published yet"
              >
                App Store — soon
              </span>
              <span
                className="inline-flex items-center justify-center rounded-xl border border-zinc-300 bg-white px-4 py-2.5 text-sm font-medium text-zinc-400 cursor-not-allowed"
                title="Not published yet"
              >
                Google Play — soon
              </span>
              <a
                href="https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/ENHANCEMENT_ROADMAP.md"
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm font-medium text-yellow-700 hover:underline text-center sm:text-right"
              >
                Mobile roadmap on GitHub →
              </a>
            </div>
          </div>
        </div>

        <p className="text-center text-sm text-zinc-500 mt-10">
          <a
            href={REPO_RELEASES_LATEST}
            target="_blank"
            rel="noopener noreferrer"
            className="text-yellow-700 font-medium hover:underline"
          >
            Browse CLI release assets &amp; checksums →
          </a>
        </p>
      </div>
    </section>
  );
};

export default DownloadSection;
