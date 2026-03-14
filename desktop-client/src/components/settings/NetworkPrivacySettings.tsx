import { Button } from "../Button";
import { Shield, AltArrowLeft } from "@solar-icons/react";

const NYM_VPN_URL = "https://nym.com/vpn";
const NYM_GITHUB = "https://github.com/nymtech/nym";
const NYM_VPN_GITHUB = "https://github.com/nymtech/nym-vpn-client";

interface NetworkPrivacySettingsProps {
  onBack: () => void;
}

export function NetworkPrivacySettings({ onBack }: NetworkPrivacySettingsProps) {
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
      <button
        onClick={onBack}
        className="flex items-center gap-2 text-gray-600 dark:text-gray-400 hover:text-primary-600 dark:hover:text-primary-400 mb-6 transition-colors"
      >
        <AltArrowLeft size={20} />
        Back to Settings
      </button>

      <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2 flex items-center gap-2">
        <Shield className="text-primary-600" />
        Network privacy (Nym)
      </h2>
      <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">
        Protect who you connect to and when—on top of NozyWallet’s on-chain privacy.
      </p>

      <div className="space-y-6">
        <div className="p-4 rounded-xl bg-white/60 dark:bg-gray-800/60 border border-gray-200 dark:border-gray-700">
          <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">Why network-level privacy?</h3>
          <p className="text-sm text-gray-600 dark:text-gray-300 mb-3">
            NozyWallet keeps every transaction shielded on-chain. Traffic between your device and the internet can still reveal <em>metadata</em>: which sites you visit, when you connect to nodes or dApps, and timing patterns. Nym’s mixnet and NymVPN add strong metadata protection against network observers.
          </p>
        </div>

        <div className="p-4 rounded-xl bg-primary-50/50 dark:bg-primary-900/20 border border-primary-200 dark:border-primary-800">
          <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">NymVPN — full privacy on the web</h3>
          <p className="text-sm text-gray-600 dark:text-gray-300 mb-4">
            <strong>NymVPN</strong> is a cross-platform VPN that routes your traffic through the Nym mixnet (or a fast 2-hop WireGuard mode). When NymVPN is running, <em>all</em> app traffic—including the in-app browser and Zebra connection—can go through Nym for metadata protection.
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
            Future versions of NozyWallet may offer a “Browse via Nym” option so that only the in-app browser’s traffic goes through the Nym mixnet (via a local SOCKS5 client). See the project docs for the integration plan.
          </p>
          <Button variant="outline" onClick={openNymRepo}>
            Nym project (GitHub)
          </Button>
        </div>
      </div>
    </div>
  );
}
