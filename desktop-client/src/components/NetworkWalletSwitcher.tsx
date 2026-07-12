import { useCallback, useEffect, useMemo, useState } from "react";
import toast from "react-hot-toast";
import { Button } from "./Button";
import { walletApi } from "../lib/api";
import type { NetworkWalletStatusResponse } from "../lib/types";
import { useWalletStore } from "../store/walletStore";
import { formatErrorForDisplay } from "../utils/errors";
import { selectClassName } from "./Select";
import { textareaClassName } from "./Textarea";
import { cn } from "../lib/cn";

const MAINNET_RPC = "http://127.0.0.1:8232";
const TESTNET_RPC = "http://127.0.0.1:18232";

const fieldClass =
  "mt-1 w-full rounded-xl border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50";

function shortProfile(profile: NetworkWalletStatusResponse["active_profile"]) {
  if (!profile) return "No active profile";
  return `${profile.name} (${profile.id.slice(0, 8)})`;
}

export function NetworkWalletSwitcher() {
  const { setAddress, setHasWallet, setBalance } = useWalletStore();
  const [status, setStatus] = useState<NetworkWalletStatusResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [selectedProfileId, setSelectedProfileId] = useState<string>("");
  const [mainnetRpc, setMainnetRpc] = useState(MAINNET_RPC);
  const [testnetRpc, setTestnetRpc] = useState(TESTNET_RPC);
  const [testnetName, setTestnetName] = useState("Ironwood Testnet");
  const [testnetPassword, setTestnetPassword] = useState("");
  const [restoreMnemonic, setRestoreMnemonic] = useState("");
  const [createdMnemonic, setCreatedMnemonic] = useState<string | null>(null);

  const walletProfiles = useMemo(
    () => status?.profiles.filter((profile) => profile.has_wallet) ?? [],
    [status?.profiles]
  );

  const loadStatus = useCallback(async () => {
    setLoading(true);
    try {
      const res = await walletApi.getNetworkWalletStatus();
      setStatus(res.data);
      setSelectedProfileId(
        res.data.suggested_testnet_profile_id ??
          res.data.active_profile?.id ??
          res.data.profiles.find((profile) => profile.has_wallet)?.id ??
          ""
      );
      const mainnetProfile = res.data.profiles.find((profile) => profile.network === "mainnet");
      const testnetProfile = res.data.profiles.find((profile) => profile.network === "testnet");
      setMainnetRpc(mainnetProfile?.zebra_url ?? MAINNET_RPC);
      setTestnetRpc(testnetProfile?.zebra_url ?? TESTNET_RPC);
    } catch (err) {
      toast.error(formatErrorForDisplay(err, "Could not load wallet network status."));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadStatus();
  }, [loadStatus]);

  const activateProfile = async (profileId: string) => {
    if (!profileId) {
      toast.error("Select a wallet profile first.");
      return;
    }
    setBusy(true);
    try {
      await walletApi.switchWalletProfile(profileId);
      const res = await walletApi.getNetworkWalletStatus();
      setStatus(res.data);
      setAddress("");
      setBalance(0);
      setHasWallet(false);
      const profile = res.data.profiles.find((item) => item.id === profileId);
      toast.success(
        `${profile?.name ?? "Wallet"} selected (${profile?.network ?? res.data.network}). Unlock to continue.`,
      );
    } catch (err) {
      toast.error(formatErrorForDisplay(err, "Could not switch wallet profile."));
    } finally {
      setBusy(false);
    }
  };

  const activateNetwork = async (network: "mainnet" | "testnet") => {
    if (!selectedProfileId) {
      toast.error("Select a wallet profile first.");
      return;
    }
    setBusy(true);
    try {
      const res = await walletApi.configureNetworkWallet({
        network,
        profile_id: selectedProfileId,
        zebra_url: network === "testnet" ? testnetRpc : mainnetRpc,
      });
      setStatus(res.data);
      setAddress("");
      setBalance(0);
      setHasWallet(false);
      toast.success(`${network === "testnet" ? "Testnet" : "Mainnet"} wallet selected. Unlock it to continue.`);
    } catch (err) {
      toast.error(formatErrorForDisplay(err, "Could not switch wallet network."));
    } finally {
      setBusy(false);
    }
  };

  const createOrRestoreTestnet = async () => {
    setBusy(true);
    setCreatedMnemonic(null);
    try {
      const res = await walletApi.createOrRestoreTestnetWallet({
        name: testnetName,
        password: testnetPassword,
        mnemonic: restoreMnemonic.trim() || undefined,
        rpc_url: testnetRpc,
      });
      setAddress(res.data.address);
      setHasWallet(true);
      if (res.data.mnemonic) {
        setCreatedMnemonic(res.data.mnemonic);
      }
      toast.success(restoreMnemonic.trim() ? "Testnet wallet restored." : "Testnet wallet created.");
      await loadStatus();
    } catch (err) {
      toast.error(formatErrorForDisplay(err, "Could not create or restore testnet wallet."));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="rounded-2xl border border-white/60 dark:border-gray-700/50 bg-white/70 dark:bg-gray-800/60 p-6 shadow-xl shadow-gray-900/5 backdrop-blur-sm">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <p className="text-xs font-bold uppercase tracking-[0.24em] text-primary">
            Wallet profiles
          </p>
          <h3 className="mt-2 text-2xl font-extrabold text-gray-950 dark:text-gray-50">
            Mainnet and testnet wallets
          </h3>
          <p className="mt-2 max-w-3xl text-sm leading-6 text-gray-600 dark:text-gray-300">
            Each wallet profile remembers its own network and Zebra RPC URL. Use{" "}
            <span className="font-medium">Switch profile</span> to restore a profile&apos;s saved
            settings, or <span className="font-medium">Use Mainnet / Use Testnet</span> to bind the
            selected profile to that network.
          </p>
        </div>
        <div className="rounded-2xl border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900/50 px-4 py-3 text-sm">
          <p className="font-semibold text-gray-900 dark:text-gray-100">
            {loading ? "Loading" : status?.network === "testnet" ? "Testnet active" : "Mainnet active"}
          </p>
          <p className="mt-1 text-gray-500 dark:text-gray-400">{shortProfile(status?.active_profile ?? null)}</p>
        </div>
      </div>

      <div className="mt-5 grid gap-4 lg:grid-cols-[1fr_1fr]">
        <div className="rounded-2xl border border-gray-200 dark:border-gray-700 bg-white/70 dark:bg-gray-900/40 p-4">
          <label className="text-sm font-semibold text-gray-800 dark:text-gray-200">Wallet profile</label>
          <select
            value={selectedProfileId}
            onChange={(event) => setSelectedProfileId(event.target.value)}
            className={cn(selectClassName, "mt-2")}
          >
            <option value="">Select wallet profile</option>
            {walletProfiles.map((profile) => (
              <option key={profile.id} value={profile.id}>
                {profile.name} ({profile.network}) {profile.is_active ? "(active)" : ""}
              </option>
            ))}
          </select>
          <Button
            className="mt-3 w-full"
            variant="outline"
            disabled={busy || !selectedProfileId}
            onClick={() => void activateProfile(selectedProfileId)}
          >
            Switch profile
          </Button>

          <div className="mt-4 grid gap-3 sm:grid-cols-2">
            <div>
              <label className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Mainnet RPC
              </label>
              <input
                value={mainnetRpc}
                onChange={(event) => setMainnetRpc(event.target.value)}
                className={fieldClass}
              />
              <Button
                className="mt-2 w-full"
                variant="outline"
                disabled={busy || !selectedProfileId}
                onClick={() => void activateNetwork("mainnet")}
              >
                Use Mainnet
              </Button>
            </div>
            <div>
              <label className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Testnet RPC
              </label>
              <input
                value={testnetRpc}
                onChange={(event) => setTestnetRpc(event.target.value)}
                className={fieldClass}
              />
              <Button
                className="mt-2 w-full"
                disabled={busy || !selectedProfileId}
                onClick={() => void activateNetwork("testnet")}
              >
                Use Testnet
              </Button>
            </div>
          </div>
        </div>

        <div className="rounded-2xl border border-amber-200 dark:border-amber-800/50 bg-amber-50/70 dark:bg-amber-950/30 p-4">
          <p className="text-sm font-bold text-gray-900 dark:text-gray-100">Add Ironwood Testnet Wallet</p>
          <p className="mt-1 text-xs leading-5 text-gray-600 dark:text-gray-300">
            Leave the mnemonic blank to create a new testnet wallet, or paste an existing testnet
            recovery phrase to restore it.
          </p>
          <div className="mt-3 grid gap-3">
            <input
              value={testnetName}
              onChange={(event) => setTestnetName(event.target.value)}
              placeholder="Profile name"
              className={cn(fieldClass, "border-amber-200 dark:border-amber-800/50")}
            />
            <input
              value={testnetPassword}
              onChange={(event) => setTestnetPassword(event.target.value)}
              placeholder="Optional password"
              type="password"
              className={cn(fieldClass, "border-amber-200 dark:border-amber-800/50")}
            />
            <textarea
              value={restoreMnemonic}
              onChange={(event) => setRestoreMnemonic(event.target.value)}
              placeholder="Optional testnet mnemonic"
              rows={3}
              className={cn(textareaClassName, "border-amber-200 dark:border-amber-800/50")}
            />
            <Button disabled={busy} onClick={() => void createOrRestoreTestnet()}>
              {restoreMnemonic.trim() ? "Restore Testnet Wallet" : "Create Testnet Wallet"}
            </Button>
          </div>
          {createdMnemonic && (
            <div className="mt-3 rounded-xl border border-amber-300 dark:border-amber-700 bg-white dark:bg-gray-900 p-3 text-xs text-gray-700 dark:text-gray-300">
              <p className="font-bold text-gray-900 dark:text-gray-100">Save this testnet recovery phrase:</p>
              <p className="mt-2 break-words font-mono">{createdMnemonic}</p>
            </div>
          )}
        </div>
      </div>
    </section>
  );
}
