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
          <strong>Desktop and extension</strong> are published on the same GitHub release as the CLI, but
          installers and zips are uploaded by <strong>separate CI jobs</strong> — they are not always
          present under one-click <code className="text-xs bg-zinc-100 px-1 rounded">/latest/download/…</code>{" "}
          URLs. Use the buttons below to open the <strong>release page</strong> and pick the file from{" "}
          <strong>Assets</strong>. <strong>CLI</strong> binaries use direct links and are attached with every
          tag.
        </p>

        <div className="grid md:grid-cols-2 gap-6 mb-6">
          <div className={card}>
            <h3 className="font-semibold text-lg text-zinc-900 mb-1">
              Windows desktop app
            </h3>
            <p className="text-sm text-zinc-600 mb-3">
              Full GUI (Tauri).{" "}
              <strong className="text-zinc-800">Most people should use the setup wizard (.exe)</strong> — it
              works on normal Windows PCs and does not require MSI or special admin tooling.
            </p>
            <p className="text-xs text-zinc-500 mb-4">
              The <code className="bg-zinc-100 px-1 rounded">.msi</code> is optional: use it for IT /
              silent deployment (Intune, GPO). If your PC blocks MSI or you prefer a classic installer, use
              the <code className="bg-zinc-100 px-1 rounded">.exe</code> only.
            </p>
            <div className="flex flex-col sm:flex-row gap-3">
              <a
                href={REPO_RELEASES_LATEST}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex flex-1 items-center justify-center rounded-xl bg-yellow-500 hover:bg-yellow-400 text-white font-semibold py-3 px-4 transition-colors text-center"
              >
                Get .exe on GitHub (Assets)
              </a>
              <a
                href={REPO_RELEASES_LATEST}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex flex-1 items-center justify-center rounded-xl bg-zinc-900 hover:bg-zinc-800 text-white font-semibold py-3 px-4 transition-colors text-center"
              >
                Get .msi on GitHub (Assets)
              </a>
            </div>
            <p className="text-xs text-zinc-500 mt-3">
              On the release page, download{" "}
              <code className="bg-zinc-100 px-1 rounded">nozy-desktop-windows-x86_64-installer.exe</code>{" "}
              (or <code className="bg-zinc-100 px-1 rounded">.msi</code>). If they are missing, wait for the{" "}
              <strong>Build Desktop Release</strong> workflow or pick an older tag that lists them.
            </p>
          </div>

          <div className={card}>
            <h3 className="font-semibold text-lg text-zinc-900 mb-1">
              Browser extension (wallet)
            </h3>
            <p className="text-sm text-zinc-600 mb-4">
              Chromium zip — unzip, then{" "}
              <strong className="text-zinc-800">Load unpacked</strong> in{" "}
              <code className="text-xs bg-zinc-100 px-1 rounded">chrome://extensions</code>{" "}
              (Developer mode on).
            </p>
            <a
              href={REPO_RELEASES_LATEST}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex w-full items-center justify-center rounded-xl bg-zinc-900 hover:bg-zinc-800 text-white font-semibold py-3 transition-colors"
            >
              Get extension .zip on GitHub (Assets)
            </a>
            <a
              href={REPO_RELEASES_LATEST}
              target="_blank"
              rel="noopener noreferrer"
              className="mt-2 inline-flex w-full items-center justify-center rounded-lg border border-zinc-200 py-2 text-sm font-medium text-zinc-700 hover:bg-zinc-100"
            >
              Firefox bundle — same Assets list
            </a>
            <p className="text-xs text-zinc-500 mt-3">
              Look for{" "}
              <code className="bg-zinc-100 px-1 rounded">nozy-extension-chromium.zip</code> or{" "}
              <code className="bg-zinc-100 px-1 rounded">nozy-extension-firefox.zip</code>. If missing, wait for{" "}
              <strong>extension-release-bundles</strong> after publish.
            </p>
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
                use the <strong>desktop app</strong> or <strong>browser extension</strong> on a computer,
                or follow progress in the repo.
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

        <div className={card}>
          <h3 className="font-semibold text-lg text-zinc-900 mb-1">
            Command-line wallet (advanced)
          </h3>
          <p className="text-sm text-zinc-600 mb-4">
            Single binary per OS. On Linux / macOS run{" "}
            <code className="text-xs bg-zinc-100 px-1 rounded">chmod +x</code> after
            download.
          </p>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
            <a
              href={DOWNLOAD_URLS.cliWindows}
              className="text-center rounded-lg bg-zinc-100 hover:bg-zinc-200 py-2.5 text-sm font-medium text-zinc-900"
            >
              Windows
            </a>
            <a
              href={DOWNLOAD_URLS.cliLinux}
              className="text-center rounded-lg bg-zinc-100 hover:bg-zinc-200 py-2.5 text-sm font-medium text-zinc-900"
            >
              Linux
            </a>
            <a
              href={DOWNLOAD_URLS.cliMacArm}
              className="text-center rounded-lg bg-zinc-100 hover:bg-zinc-200 py-2.5 text-sm font-medium text-zinc-900"
            >
              macOS ARM
            </a>
            <a
              href={DOWNLOAD_URLS.cliMacIntel}
              className="text-center rounded-lg bg-zinc-100 hover:bg-zinc-200 py-2.5 text-sm font-medium text-zinc-900"
            >
              macOS Intel
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
