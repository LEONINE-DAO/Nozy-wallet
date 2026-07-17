import { useState, useEffect } from "react";
import { Button } from "../Button";
import { Input } from "../Input";
import { CheckCircle, Danger, InfoCircle } from "@solar-icons/react";
import toast from "react-hot-toast";
import { formatErrorForDisplay } from "../../utils/errors";
import { walletApi } from "../../lib/api";
import { SettingsBackButton } from "./SettingsBackButton";

interface NetworkSettingsProps {
  onBack: () => void;
}

export function NetworkSettings({ onBack }: NetworkSettingsProps) {
  const [url, setUrl] = useState("");
  const [network, setNetwork] = useState<"mainnet" | "testnet">("mainnet");
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [testStatus, setTestStatus] = useState<"idle" | "success" | "error">(
    "idle"
  );

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      setIsLoading(true);
      const res = await walletApi.getConfig();
      if (res.data) {
        setUrl(res.data.zebra_url);
        const cfg = res.data as { network?: string };
        setNetwork(cfg.network === "testnet" ? "testnet" : "mainnet");
      }
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Failed to load network configuration"));
    } finally {
      setIsLoading(false);
    }
  };

  const handleSave = async () => {
    try {
      setIsSaving(true);
      await walletApi.setZebraUrl({ url });
      toast.success("Network configuration saved");
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Failed to save configuration"));
    } finally {
      setIsSaving(false);
    }
  };

  const handleTestConnection = async () => {
    const testToast = toast.loading("Testing connection to node...");
    setTestStatus("idle");
    try {
      const res = await walletApi.testZebraConnection();
      if (res?.data) {
        setTestStatus("success");
        toast.success("Node connection successful!", { id: testToast });
      } else {
        setTestStatus("error");
        toast.error("Node connection failed", { id: testToast });
      }
    } catch (e) {
      setTestStatus("error");
      toast.error(formatErrorForDisplay(e, "Failed to connect to node. Check your URL."), {
        id: testToast,
      });
    }
  };

  return (
    <div className="max-w-2xl mx-auto animate-fade-in pb-8">
      <SettingsBackButton onClick={onBack} />

      <h2 className="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-2">Network & Node</h2>
      <p className="text-gray-500 dark:text-gray-400 mb-8">
        Configure your connection to a full node for syncing and sending. Use{" "}
        <a
          href="https://github.com/ZcashFoundation/zebra"
          className="text-emerald-600 dark:text-emerald-400 underline"
          target="_blank"
          rel="noreferrer"
        >
          Zebrad
        </a>{" "}
        or{" "}
        <a
          href="https://zakura.com/"
          className="text-emerald-600 dark:text-emerald-400 underline"
          target="_blank"
          rel="noreferrer"
        >
          Zakura
        </a>{" "}
        JSON-RPC (same port and URL field).
      </p>

      <div className="space-y-6">
        <div className="bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm rounded-2xl p-6 border border-white/50 dark:border-gray-700/50 shadow-sm space-y-4">
          <div className="flex items-center justify-between gap-3">
            <p className="text-sm font-semibold text-gray-700 dark:text-gray-300">Active wallet network</p>
            <span
              className={`rounded-full px-3 py-1 text-xs font-semibold ${
                network === "testnet"
                  ? "bg-amber-100 dark:bg-amber-900/30 text-amber-800 dark:text-amber-200"
                  : "bg-emerald-100 dark:bg-emerald-900/30 text-emerald-800 dark:text-emerald-200"
              }`}
            >
              {network === "testnet" ? "Testnet (Ironwood)" : "Mainnet"}
            </span>
          </div>
          {network === "testnet" && (
            <p className="text-xs text-amber-800 dark:text-amber-200 bg-amber-50 dark:bg-amber-900/20 border border-amber-100 dark:border-amber-800/50 rounded-xl px-3 py-2">
              Testnet wallets use port <strong>18232</strong>, not 8232. Switch network in
              Settings → Wallets & Accounts if this still shows mainnet.
            </p>
          )}
          <Input
            label="Full node RPC URL (Zebrad or Zakura)"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder={
              network === "testnet" ? "http://127.0.0.1:18232" : "http://127.0.0.1:8232"
            }
            disabled={isLoading}
          />
          <div className="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
            <InfoCircle size={14} />
            <span>
              {network === "testnet" ? (
                <>
                  Local testnet:{" "}
                  <code className="bg-black/5 dark:bg-white/10 px-1 rounded">http://127.0.0.1:18232</code>. WSL
                  node: use the WSL IP, e.g.{" "}
                  <code className="bg-black/5 dark:bg-white/10 px-1 rounded">http://172.x.x.x:18232</code>.
                </>
              ) : (
                <>
                  Local mainnet:{" "}
                  <code className="bg-black/5 dark:bg-white/10 px-1 rounded">http://127.0.0.1:8232</code>. Remote
                  node: use that host&apos;s IP on the same port.
                </>
              )}
            </span>
          </div>

          <div className="pt-2 flex gap-3">
            <Button
              variant="secondary"
              onClick={handleTestConnection}
              className="flex-1"
            >
              Test Connection
            </Button>
            <Button
              onClick={handleSave}
              disabled={isSaving}
              className="flex-1"
            >
              {isSaving ? "Saving..." : "Save Configuration"}
            </Button>
          </div>

          {testStatus !== "idle" && (
            <div
              className={`p-3 rounded-xl flex items-center gap-2 text-sm font-medium ${
                testStatus === "success"
                  ? "bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-300 border border-green-100 dark:border-green-800/50"
                  : "bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-300 border border-red-100 dark:border-red-800/50"
              }`}
            >
              {testStatus === "success" ? (
                <>
                  <CheckCircle size={18} /> Connection Successful
                </>
              ) : (
                <>
                  <Danger size={18} /> Connection Failed
                </>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
