import {
  DOWNLOAD_URLS,
  DESKTOP_DOWNLOAD_URLS,
  REPO_RELEASES_LATEST,
} from "../lib/downloads";
import { PATHS } from "../lib/links";

const DownloadSection = () => {
  return (
    <section
      id="download"
      className="py-24 border-t border-[rgba(245,240,230,0.12)] scroll-mt-24"
    >
      <div className="max-w-7xl mx-auto px-6">
        <div className="max-w-2xl mb-12">
          <p className="nw-kicker mb-4">Download</p>
          <h2 className="font-display text-3xl lg:text-5xl font-bold text-[#f5f0e6] mb-4 tracking-tight">
            Get NozyWallet
          </h2>
          <p className="text-[#a39a88] text-lg leading-relaxed">
            CLI Lite and the localhost companion API are production for same-machine use.
            Extension and desktop are in public beta. Mobile is coming next.
          </p>
        </div>

        <div className="nw-panel p-6 mb-5 border-[#c8ccd4]/25">
          <div className="flex flex-wrap items-center gap-3 mb-3">
            <h3 className="font-display text-xl font-bold text-[#f5f0e6]">CLI Lite</h3>
            <span className="text-[10px] font-semibold uppercase tracking-[0.14em] px-2 py-1 border border-emerald-500/30 text-emerald-300 bg-emerald-500/10">
              Mainnet
            </span>
          </div>
          <p className="text-sm text-[#a39a88] mb-5 leading-relaxed max-w-3xl">
            Production <code className="text-[#c8ccd4]">nozy</code> binary — Orchard + Ironwood.
            Pair with your own Zebrad + lightwalletd. On Linux / macOS run{" "}
            <code className="text-[#c8ccd4]">chmod +x</code> after download.
          </p>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 mb-4">
            {[
              { label: "Windows", href: DOWNLOAD_URLS.cliWindows },
              { label: "Linux", href: DOWNLOAD_URLS.cliLinux },
              { label: "macOS ARM", href: DOWNLOAD_URLS.cliMacArm },
              { label: "macOS Intel", href: DOWNLOAD_URLS.cliMacIntel },
            ].map((item) => (
              <a key={item.label} href={item.href} className="nw-btn nw-btn-primary !py-2.5 text-sm">
                {item.label}
              </a>
            ))}
          </div>
          <a href={DOWNLOAD_URLS.hashes} className="text-xs text-[#c8ccd4] hover:underline">
            Verify with HASHES.txt →
          </a>
        </div>

        <div className="nw-panel p-6 mb-5 border-emerald-500/20">
          <div className="flex flex-wrap items-center gap-3 mb-3">
            <h3 className="font-display text-xl font-bold text-[#f5f0e6]">Companion API</h3>
            <span className="text-[10px] font-semibold uppercase tracking-[0.14em] px-2 py-1 border border-emerald-500/30 text-emerald-300 bg-emerald-500/10">
              Production · localhost
            </span>
          </div>
          <p className="text-sm text-[#a39a88] mb-5 leading-relaxed max-w-3xl">
            <code className="text-[#c8ccd4]">nozywallet-api</code> — same wallet core over HTTP for
            the extension and local apps. Default bind{" "}
            <code className="text-[#c8ccd4]">http://127.0.0.1:3000</code>. Not a public hosted
            wallet. Ships on the latest CLI release (e.g. Mango Habanero).
          </p>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 mb-4">
            {[
              { label: "Windows", href: DOWNLOAD_URLS.apiWindows },
              { label: "Linux", href: DOWNLOAD_URLS.apiLinux },
              { label: "macOS ARM", href: DOWNLOAD_URLS.apiMacArm },
              { label: "macOS Intel", href: DOWNLOAD_URLS.apiMacIntel },
            ].map((item) => (
              <a key={item.label} href={item.href} className="nw-btn nw-btn-primary !py-2.5 text-sm">
                {item.label}
              </a>
            ))}
          </div>
          <a
            href={PATHS.apiServer}
            target="_blank"
            rel="noopener noreferrer"
            className="text-xs text-[#c8ccd4] hover:underline"
          >
            Companion API docs →
          </a>
        </div>

        <div className="nw-panel p-6 mb-5">
          <div className="flex flex-wrap items-center gap-3 mb-3">
            <h3 className="font-display text-xl font-bold text-[#f5f0e6]">Desktop</h3>
            <span className="text-[10px] font-semibold uppercase tracking-[0.14em] px-2 py-1 border border-[#c8ccd4]/30 text-[#c8ccd4] bg-[#c8ccd4]/10">
              Beta · wallet-only
            </span>
          </div>
          <p className="text-sm text-[#a39a88] mb-5 leading-relaxed max-w-3xl">
            Native sync, send, history, and settings. No in-app browser — use the extension for
            sites and dApps. Prefer CLI Lite for production mainnet today.
          </p>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 mb-4">
            {[
              { label: "Windows", href: DESKTOP_DOWNLOAD_URLS.windows },
              { label: "macOS ARM", href: DESKTOP_DOWNLOAD_URLS.macArm },
              { label: "Linux", href: DESKTOP_DOWNLOAD_URLS.linux },
            ].map((item) => (
              <a key={item.label} href={item.href} className="nw-btn nw-btn-ghost !py-2.5 text-sm">
                {item.label}
              </a>
            ))}
          </div>
          <a
            href={DESKTOP_DOWNLOAD_URLS.releasePage}
            target="_blank"
            rel="noopener noreferrer"
            className="text-xs text-[#c8ccd4] hover:underline"
          >
            Desktop release notes &amp; checksums →
          </a>
        </div>

        <div className="grid md:grid-cols-2 gap-5">
          <div className="nw-panel p-6">
            <div className="flex flex-wrap items-center gap-3 mb-3">
              <h3 className="font-display text-xl font-bold text-[#f5f0e6]">Extension</h3>
              <span className="text-[10px] font-semibold uppercase tracking-[0.14em] px-2 py-1 border border-[#c8ccd4]/30 text-[#c8ccd4] bg-[#c8ccd4]/10">
                Beta · 0.1.7
              </span>
            </div>
            <p className="text-sm text-[#a39a88] mb-5 leading-relaxed">
              Chrome, Brave, Edge — load unpacked. Optional local companion for sync. Store listing
              in progress.
            </p>
            <a
              href={PATHS.extensionRelease}
              target="_blank"
              rel="noopener noreferrer"
              className="nw-btn nw-btn-primary w-full !py-2.5 text-sm mb-2"
            >
              Download extension zip
            </a>
            <a
              href={PATHS.extension}
              target="_blank"
              rel="noopener noreferrer"
              className="block text-center text-sm text-[#c8ccd4] hover:underline"
            >
              Install guide →
            </a>
          </div>

          <div className="nw-panel p-6 opacity-90">
            <div className="flex flex-wrap items-center gap-3 mb-3">
              <h3 className="font-display text-xl font-bold text-[#f5f0e6]">Mobile</h3>
              <span className="text-[10px] font-semibold uppercase tracking-[0.14em] px-2 py-1 border border-[rgba(245,240,230,0.15)] text-[#a39a88]">
                Coming soon
              </span>
            </div>
            <p className="text-sm text-[#a39a88] mb-5 leading-relaxed">
              Expo companion for App Store / Play is in prep. Until then use CLI, desktop, or
              extension.
            </p>
            <a href={PATHS.mobilePage} className="nw-btn nw-btn-ghost w-full !py-2.5 text-sm">
              Mobile preview →
            </a>
          </div>
        </div>

        <p className="text-center text-sm text-[#a39a88] mt-12">
          <a
            href={REPO_RELEASES_LATEST}
            target="_blank"
            rel="noopener noreferrer"
            className="text-[#c8ccd4] font-medium hover:underline"
          >
            Browse all release assets →
          </a>
        </p>
      </div>
    </section>
  );
};

export default DownloadSection;
