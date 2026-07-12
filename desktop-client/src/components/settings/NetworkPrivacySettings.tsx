import { Button } from "../Button";
import { Shield } from "@solar-icons/react";
import { SettingsBackButton } from "./SettingsBackButton";
import { useSettingsStore } from "../../store/settingsStore";

const NYM_VPN_URL = "https://nym.com/vpn";
const NYM_GITHUB = "https://github.com/nymtech/nym";
const NYM_VPN_GITHUB = "https://github.com/nymtech/nym-vpn-client";

interface NetworkPrivacySettingsProps {
  onBack: () => void;
}

export function NetworkPrivacySettings({ onBack }: NetworkPrivacySettingsProps) {
  const attestPrivateNetworkForMigration = useSettingsStore(
    (s) => s.attestPrivateNetworkForMigration
  );
  const setAttestPrivateNetworkForMigration = useSettingsStore(
    (s) => s.setAttestPrivateNetworkForMigration
  );

  const openNymVpn = () => {
    window.open(NYM_VPN_URL, "_blank", "noopener,noreferrer");
  };

  const openNymRepo = () => {
    window.open(NYM_GITHUB, "_blank", "noopener,noreferrer");
  };

  const openNymVpnRepo = () => {
    window.open(NYM_VPN_GITHUB, "_blank", "noopener,noreferrer");
  };

  return (
    <div className="max-w-2xl mx-auto animate-fade-in">
      <SettingsBackButton onClick={onBack} />

      <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2 flex items-center gap-2">
        <Shield className="text-primary-600" />
        Network privacy
      </h2>
      <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">
        Safer migration defaults to a local Zebrad. Nym/Tor options live under Advanced.
      </p>

      <div className="space-y-6">
        <div className="p-4 rounded-xl bg-primary-50/50 dark:bg-primary-900/20 border border-primary-200 dark:border-primary-800">
          <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">
            Default — local Zebrad
          </h3>
          <p className="text-sm text-gray-600 dark:text-gray-300 mb-3">
            For Ironwood migration IP protection (Priority 1), Nozy expects a{" "}
            <strong>local Zebrad</strong> in Settings → Network &amp; Node (for example{" "}
            <code className="text-xs">http://127.0.0.1:8232</code> /{" "}
            <code className="text-xs">:18232</code>). That path is allowed automatically — no
            attestation required.
          </p>
          <p className="text-sm text-gray-600 dark:text-gray-300">
            Remote clearnet RPC is blocked for safer migration until you use a local node, a
            detected Tor/I2P SOCKS proxy, or the Advanced attestation below.
          </p>
        </div>

        <div className="p-4 rounded-xl bg-white/60 dark:bg-gray-800/60 border border-gray-200 dark:border-gray-700">
          <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">Why network-level privacy?</h3>
          <p className="text-sm text-gray-600 dark:text-gray-300 mb-3">
            NozyWallet keeps every transaction shielded on-chain. Traffic between your device and
            the internet can still reveal <em>metadata</em>: which sites you visit, when you
            connect to nodes or dApps, and timing patterns. A local node plus optional Nym/Tor
            reduces IP linkage during sync and migration broadcast.
          </p>
        </div>

        <div className="p-4 rounded-xl bg-white/60 dark:bg-gray-800/60 border border-gray-200 dark:border-gray-700">
          <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">NymVPN — full privacy on the web</h3>
          <p className="text-sm text-gray-600 dark:text-gray-300 mb-4">
            <strong>NymVPN</strong> routes traffic through the Nym mixnet (or a fast 2-hop
            WireGuard mode). Useful for general metadata protection; for migration, prefer local
            Zebrad first.
          </p>
          <div className="flex flex-wrap gap-3">
            <Button variant="primary" onClick={openNymVpn}>
              Get NymVPN (nym.com)
            </Button>
            <Button variant="outline" onClick={openNymVpnRepo}>
              NymVPN source (GitHub)
            </Button>
          </div>
        </div>

        <div className="p-4 rounded-xl bg-white/60 dark:bg-gray-800/60 border border-gray-200 dark:border-gray-700">
          <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">Nym mixnet (browser integration)</h3>
          <p className="text-sm text-gray-600 dark:text-gray-300 mb-3">
            Future versions of NozyWallet may offer a “Browse via Nym” option so that only the
            in-app browser’s traffic goes through the Nym mixnet (via a local SOCKS5 client).
          </p>
          <Button variant="outline" onClick={openNymRepo}>
            Nym project (GitHub)
          </Button>
        </div>

        <div className="p-4 rounded-xl bg-white/60 dark:bg-gray-800/60 border border-gray-200 dark:border-gray-700">
          <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">
            Nym smolmix — remote tx broadcast (CLI / advanced)
          </h3>
          <p className="text-sm text-gray-600 dark:text-gray-300 mb-3">
            Biggest IP↔tx win: route <strong>remote</strong>{" "}
            <code className="text-xs">sendrawtransaction</code> through the Nym mixnet helper
            (not attestation alone). Local/LAN Zebrad stays direct.
          </p>
          <ol className="list-decimal list-inside text-sm text-gray-600 dark:text-gray-300 space-y-1 mb-3">
            <li>
              Build helper:{" "}
              <code className="text-xs">
                cd tools/nym-smolmix-broadcast-spike && cargo build --release
              </code>
            </li>
            <li>
              Set{" "}
              <code className="text-xs">NOZY_NYM_SMOLMIX_BIN</code> to that binary, and{" "}
              <code className="text-xs">NOZY_BROADCAST_VIA_NYM_MIXNET=1</code> (or config{" "}
              <code className="text-xs">privacy_network.broadcast_via_nym_mixnet</code>).
            </li>
            <li>
              Remote Zebrad URL must be reachable from a Nym exit (not WSL LAN /{" "}
              <code className="text-xs">172.20…</code>).
            </li>
          </ol>
          <p className="text-xs text-gray-500 dark:text-gray-400">
            Tracking: issue #147 · docs/reference/NYM_IP_PRIVACY_CASE_BREAKDOWN.md
          </p>
        </div>

        <div className="p-4 rounded-xl border border-amber-200 dark:border-amber-800/50 bg-amber-50/60 dark:bg-amber-950/20">
          <p className="text-xs font-bold uppercase tracking-[0.18em] text-amber-700 dark:text-amber-400 mb-2">
            Advanced
          </p>
          <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">
            Attest NymVPN / Tor for remote Zebrad
          </h3>
          <p className="text-sm text-gray-600 dark:text-gray-300 mb-4">
            Only if you must use a remote Zebra RPC and already run NymVPN or Tor system-wide.
            Nozy cannot detect system VPN paths automatically. Prefer a local node instead.
            This attestation is used by Ironwood Broadcast on the Ironwood tab.
          </p>
          <label className="flex cursor-pointer items-start gap-3">
            <input
              type="checkbox"
              className="mt-1 h-4 w-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500"
              checked={attestPrivateNetworkForMigration}
              onChange={(e) => setAttestPrivateNetworkForMigration(e.target.checked)}
            />
            <span>
              <span className="block text-sm font-semibold text-gray-900 dark:text-gray-100">
                I am on NymVPN / Tor (or equivalent)
              </span>
              <span className="mt-1 block text-xs leading-5 text-gray-600 dark:text-gray-400">
                Allows safer-migration Priority 1 when Zebrad is remote and no Tor/I2P SOCKS is
                detected. Turn this off when you switch back to a local node.
              </span>
            </span>
          </label>
        </div>
      </div>
    </div>
  );
}
