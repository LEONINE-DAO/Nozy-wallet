import { useState, useEffect } from "react";
import { Button } from "../Button";
import { Input } from "../Input";
import { ArrowLeft, Shield } from "@solar-icons/react";
import QRCode from "react-qr-code";
import toast from "react-hot-toast";
import { walletApi } from "../../lib/api";
import type { KeystoneStatusResponse } from "../../lib/types";
import { formatErrorForDisplay } from "../../utils/errors";

interface KeystoneSettingsProps {
  onBack: () => void;
}

export function KeystoneSettings({ onBack }: KeystoneSettingsProps) {
  const [keystoneStatus, setKeystoneStatus] = useState<KeystoneStatusResponse | null>(null);
  const [deviceLabel, setDeviceLabel] = useState("My Keystone");
  const [ufvkPassword, setUfvkPassword] = useState("");
  const [exportedUfvk, setExportedUfvk] = useState("");
  const [keystoneLoading, setKeystoneLoading] = useState(false);

  const refreshKeystoneStatus = async () => {
    try {
      const { data } = await walletApi.getKeystoneStatus();
      setKeystoneStatus(data);
      if (data.device_label) setDeviceLabel(data.device_label);
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Failed to load Keystone status"));
    }
  };

  useEffect(() => {
    void refreshKeystoneStatus();
  }, []);

  return (
    <div className="max-w-2xl mx-auto animate-fade-in">
      <button
        onClick={onBack}
        className="flex items-center gap-2 text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 mb-6 transition-colors"
      >
        <ArrowLeft className="w-5 h-5" />
        <span className="font-medium">Back to Settings</span>
      </button>

      <div className="flex items-center gap-3 mb-6">
        <div className="w-12 h-12 rounded-full bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center">
          <Shield className="w-6 h-6 text-amber-600 dark:text-amber-400" />
        </div>
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100">Keystone</h2>
          <p className="text-sm text-amber-600 dark:text-amber-400 font-medium">
            Zcash mainnet · air-gapped hardware signing via PCZT + QR
          </p>
        </div>
      </div>

      <div className="space-y-4">
        <div className="p-4 rounded-xl bg-white/60 dark:bg-gray-800/60 border border-white/50 dark:border-gray-700/50 space-y-3">
          <h3 className="font-semibold text-gray-900 dark:text-gray-100">How it works</h3>
          <ol className="text-sm text-gray-600 dark:text-gray-400 list-decimal list-inside space-y-2">
            <li>Set your Keystone device to <strong>Zcash mainnet</strong>.</li>
            <li>Export your Orchard UFVK from Nozy (read-only pairing).</li>
            <li>Import the UFVK on Keystone and confirm the same unified address.</li>
            <li>Enable Keystone below — sends on the Send tab will use PCZT signing.</li>
            <li>Nozy builds a PCZT → scan QR on Keystone → sign → paste signed PCZT back → broadcast.</li>
          </ol>
        </div>

        {keystoneStatus?.network === "testnet" && (
          <div className="p-4 rounded-xl border border-red-300/80 bg-red-50/80 dark:bg-red-950/30 dark:border-red-800/60">
            <p className="text-sm text-red-800 dark:text-red-200 font-medium">
              Wallet is on testnet — Keystone requires mainnet
            </p>
            <p className="text-xs text-red-700 dark:text-red-300 mt-1">
              Set <code className="text-xs">network</code> to <code className="text-xs">mainnet</code> in
              your wallet config before pairing or sending with Keystone.
            </p>
          </div>
        )}

        <div className="p-4 rounded-xl bg-white/60 dark:bg-gray-800/60 border border-white/50 dark:border-gray-700/50 space-y-4">
          <div className="flex items-center justify-between gap-2">
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Status:{" "}
              {keystoneStatus
                ? `${keystoneStatus.enabled ? "Enabled" : "Disabled"} · ${
                    keystoneStatus.network === "testnet" ? "testnet" : "mainnet"
                  } · UFVK ${
                    keystoneStatus.has_ufvk ? "paired" : "not paired"
                  }${keystoneStatus.pending_send ? " · pending send" : ""}`
                : "Loading…"}
            </p>
            <Button variant="ghost" size="sm" onClick={() => void refreshKeystoneStatus()}>
              Refresh
            </Button>
          </div>

          <Input
            label="Device label"
            value={deviceLabel}
            onChange={(e) => setDeviceLabel(e.target.value)}
            placeholder="My Keystone"
          />

          <div className="flex flex-wrap gap-2">
            <Button
              variant="secondary"
              size="sm"
              disabled={keystoneLoading || (keystoneStatus?.network === "testnet" && !keystoneStatus?.enabled)}
              onClick={async () => {
                setKeystoneLoading(true);
                try {
                  const enabled = !keystoneStatus?.enabled;
                  await walletApi.setKeystoneEnabled(enabled, deviceLabel);
                  await refreshKeystoneStatus();
                  toast.success(
                    enabled
                      ? "Keystone enabled — use Send to prepare PCZT transactions"
                      : "Keystone disabled — sends use local signing"
                  );
                } catch (e) {
                  toast.error(formatErrorForDisplay(e, "Failed to update Keystone setting"));
                } finally {
                  setKeystoneLoading(false);
                }
              }}
            >
              {keystoneStatus?.enabled ? "Disable Keystone" : "Enable Keystone"}
            </Button>
          </div>
        </div>

        <div className="p-4 rounded-xl bg-white/60 dark:bg-gray-800/60 border border-white/50 dark:border-gray-700/50 space-y-3">
          <h3 className="font-semibold text-gray-900 dark:text-gray-100">Export UFVK for pairing</h3>
          <p className="text-xs text-gray-500">
            Mainnet UFVK (read-only). Import on your Keystone device set to Zcash mainnet. Scan the
            QR or copy the text — it should start with <code className="text-xs">uview1</code>.
          </p>
          <Input
            type="password"
            label="Wallet password (if locked)"
            value={ufvkPassword}
            onChange={(e) => setUfvkPassword(e.target.value)}
            placeholder="Optional if already unlocked"
          />
          <Button
            size="sm"
            className="gap-2"
            disabled={keystoneLoading || keystoneStatus?.network === "testnet"}
            onClick={async () => {
              setKeystoneLoading(true);
              try {
                const { data } = await walletApi.exportKeystoneUfvk(
                  ufvkPassword.trim() || undefined
                );
                setExportedUfvk(data.ufvk);
                await refreshKeystoneStatus();
                toast.success("UFVK exported — pair on your Keystone device");
              } catch (e) {
                toast.error(formatErrorForDisplay(e, "UFVK export failed"));
              } finally {
                setKeystoneLoading(false);
              }
            }}
          >
            Export UFVK
          </Button>

          {exportedUfvk && (
            <div className="space-y-3 pt-2">
              <div className="flex justify-center p-4 bg-white rounded-xl border border-gray-200">
                <QRCode value={exportedUfvk} size={180} level="M" />
              </div>
              <textarea
                readOnly
                value={exportedUfvk}
                rows={3}
                className="w-full rounded-lg border border-gray-200 dark:border-gray-600 bg-gray-50/50 dark:bg-gray-900/30 px-3 py-2 text-xs font-mono"
              />
              <Button
                variant="outline"
                size="sm"
                onClick={async () => {
                  try {
                    await navigator.clipboard.writeText(exportedUfvk);
                    toast.success("UFVK copied");
                  } catch {
                    toast.error("Could not copy to clipboard");
                  }
                }}
              >
                Copy UFVK
              </Button>
            </div>
          )}
        </div>

        <p className="text-xs text-amber-700 dark:text-amber-300 px-1">
          When Keystone is enabled, go to <strong>Send</strong> to build and broadcast mainnet PCZT
          transactions. Recipients must use mainnet Orchard unified addresses (<code className="text-xs">u1…</code>).
        </p>
      </div>
    </div>
  );
}
