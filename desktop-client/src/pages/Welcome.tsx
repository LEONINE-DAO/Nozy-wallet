import { useState, useEffect } from "react";
import { Eye, EyeClosed, Shield, Lock, Copy, Refresh } from "@solar-icons/react";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { Textarea } from "../components/Textarea";
import { Card } from "../components/Card";
import { useWalletStore } from "../store/walletStore";
import { walletApi } from "../lib/api";
import toast from "react-hot-toast";
import { formatErrorForDisplay } from "../utils/errors";

type ViewState = "initial" | "create" | "restore" | "securityTips";

export function WelcomePage() {
  const [view, setView] = useState<ViewState>("initial");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [walletExistsOnDisk, setWalletExistsOnDisk] = useState(false);
  const [requiresPassword, setRequiresPassword] = useState(false);
  const { setHasWallet, setAddress } = useWalletStore();

  const [createPassword, setCreatePassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showCreatePass, setShowCreatePass] = useState(false);
  const [showConfirmPass, setShowConfirmPass] = useState(false);
  const [generatedMnemonic, setGeneratedMnemonic] = useState<string | null>(null);
  const [mnemonicCopied, setMnemonicCopied] = useState(false);

  const [mnemonic, setMnemonic] = useState("");
  const [restorePassword, setRestorePassword] = useState("");
  const [showRestorePass, setShowRestorePass] = useState(false);
  const [unlockPassword, setUnlockPassword] = useState("");
  const [showUnlockPass, setShowUnlockPass] = useState(false);

  useEffect(() => {
    walletApi
      .checkWalletExists()
      .then((res) => {
        if (res.data?.exists) {
          setWalletExistsOnDisk(true);
          setRequiresPassword(res.data.has_password ?? false);
        }
      })
      .catch(() => undefined);
  }, []);

  const handleUnlockWallet = async (e: React.FormEvent) => {
    e.preventDefault();
    const unlockToast = toast.loading("Unlocking wallet...");
    setIsLoading(true);
    setError(null);
    try {
      const res = await walletApi.unlockWallet({
        password: requiresPassword ? unlockPassword : "",
      });
      if (res.data.address) {
        setAddress(res.data.address);
      }
      toast.success("Wallet unlocked", { id: unlockToast });
      setHasWallet(true);
    } catch (err: unknown) {
      const errMsg = formatErrorForDisplay(err, "Failed to unlock wallet. Check your password.");
      setError(errMsg);
      toast.error(errMsg, { id: unlockToast });
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreateWallet = async (e: React.FormEvent) => {
    e.preventDefault();
    if (createPassword !== confirmPassword) {
      toast.error("Passwords do not match");
      setError("Passwords do not match");
      return;
    }

    const createToast = toast.loading("Creating secure wallet...");
    setIsLoading(true);
    setError(null);
    try {
      const response = await walletApi.createWallet({ password: createPassword });
      const mnemonic = response?.data;
      if (mnemonic) {
        setGeneratedMnemonic(mnemonic);
        setWalletExistsOnDisk(true);
        setRequiresPassword(Boolean(createPassword));
        toast.success("Wallet created! Please save your recovery phrase.", { id: createToast });
      } else {
        toast.success("Wallet created successfully!", { id: createToast });
        setHasWallet(true);
      }
    } catch (err: unknown) {
      const errMsg = formatErrorForDisplay(err, "Failed to create wallet. Please try again.");
      setError(errMsg);
      toast.error(errMsg, { id: createToast });
    } finally {
      setIsLoading(false);
    }
  };

  const handleRestoreWallet = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!mnemonic.trim()) {
      toast.error("Mnemonic is required");
      setError("Mnemonic is required");
      return;
    }

    const restoreToast = toast.loading("Restoring wallet from seed...");
    setIsLoading(true);
    setError(null);
    try {
      await walletApi.restoreWallet({
        mnemonic: mnemonic.trim(),
        password: restorePassword,
      });
      setWalletExistsOnDisk(true);
      setRequiresPassword(Boolean(restorePassword));
      toast.success("Wallet restored successfully!", { id: restoreToast });
      setView("securityTips");
    } catch (err: unknown) {
      const errMsg = formatErrorForDisplay(err, "Failed to restore wallet. Please check your mnemonic.");
      setError(errMsg);
      toast.error(errMsg, { id: restoreToast });
    } finally {
      setIsLoading(false);
    }
  };

  const handleBack = () => {
    setView("initial");
    setError(null);
    setCreatePassword("");
    setConfirmPassword("");
    setMnemonic("");
    setRestorePassword("");
    setGeneratedMnemonic(null);
  };

  const handleMnemonicCopied = () => {
    navigator.clipboard.writeText(generatedMnemonic || "");
    setMnemonicCopied(true);
    toast.success("Recovery phrase copied to clipboard!");
    setTimeout(() => setMnemonicCopied(false), 2000);
  };

  const handleMnemonicConfirmed = () => {
    setGeneratedMnemonic(null);
    setView("securityTips");
  };

  const handleGetStarted = async () => {
    try {
      const statusRes = await walletApi.getWalletStatus();
      if (statusRes.data.address) {
        setAddress(statusRes.data.address);
      }
    } catch {
      // Best-effort; wallet session should already be open after create/restore.
    }
    setView("initial");
    setHasWallet(true);
  };

  const PasswordToggle = ({
    show,
    onToggle,
  }: {
    show: boolean;
    onToggle: () => void;
  }) => (
    <button
      type="button"
      onClick={onToggle}
      className="focus:outline-none hover:text-primary transition-colors text-gray-500 dark:text-gray-400"
    >
      {show ? <EyeClosed size={20} /> : <Eye size={20} />}
    </button>
  );

  return (
    <div className="h-screen w-full bg-gray-950 text-gray-100 font-sans flex flex-col justify-center items-center overflow-auto relative">
      <div className="absolute top-0 left-0 w-full h-96 bg-linear-to-b from-primary-900/20 to-transparent pointer-events-none" />

      <div className="container mx-auto px-8 py-12 relative z-10 flex flex-col items-center">
        <div className="max-w-md w-full text-center space-y-8 animate-fade-in">
          <div className="flex justify-center mb-4">
            <div className="aspect-square w-48 rounded-2xl flex items-center justify-center">
              <img
                src="/logo.png"
                alt="Nozy Wallet"
                className="aspect-square w-48 object-contain"
                onError={(e) => {
                  e.currentTarget.style.display = "none";
                }}
              />
            </div>
          </div>

          <h1 className="text-3xl md:text-4xl font-extrabold tracking-tight text-gray-700 dark:text-gray-200 animate-slide-up animation-delay-200">
            Privacy by Default.
          </h1>

          {view === "initial" && (
            <div className="space-y-8 animate-slide-up animation-delay-200">
              {!walletExistsOnDisk && (
                <p className="text-xl text-gray-600 dark:text-gray-400 leading-relaxed max-w-2xl mx-auto">
                  Experience the future of private finance. Nozy Wallet combines
                  speed and anonymity in a beautiful, easy-to-use interface.
                </p>
              )}

              {walletExistsOnDisk && (
                <form onSubmit={handleUnlockWallet} className="text-left space-y-6 max-w-md mx-auto">
                  <div className="text-center mb-2">
                    <h2 className="text-2xl font-bold text-gray-800 dark:text-gray-200">Welcome back</h2>
                    <p className="text-gray-500 dark:text-gray-400">
                      {requiresPassword
                        ? "Enter your password to unlock your wallet"
                        : "Nozy people will be Nozy shield up"}
                    </p>
                  </div>

                  {requiresPassword && (
                    <Input
                      type={showUnlockPass ? "text" : "password"}
                      label="Password"
                      placeholder="Enter your wallet password"
                      value={unlockPassword}
                      onChange={(e) => setUnlockPassword(e.target.value)}
                      required
                      error={error || undefined}
                      suffix={
                        <PasswordToggle
                          show={showUnlockPass}
                          onToggle={() => setShowUnlockPass(!showUnlockPass)}
                        />
                      }
                    />
                  )}

                  {error && !requiresPassword && (
                    <p className="text-sm text-red-600">{error}</p>
                  )}

                  <Button type="submit" disabled={isLoading} className="w-full py-6 text-lg">
                    {isLoading ? "Unlocking..." : "Unlock Wallet"}
                  </Button>
                </form>
              )}

              {walletExistsOnDisk && (
                <p className="text-sm text-gray-500 dark:text-gray-400 text-center">
                  or set up a different wallet
                </p>
              )}

              <div className="flex flex-col sm:flex-row gap-4 justify-center">
                <Button
                  size="md"
                  onClick={() => {
                    setError(null);
                    setView("create");
                  }}
                  className="rounded-xl px-10 py-4 text-lg bg-white dark:bg-gray-800 hover:bg-primary shadow-lg hover:shadow-none transition-all duration-300 text-gray-900 dark:text-gray-100"
                >
                  Create New Wallet
                </Button>
                <Button
                  size="md"
                  onClick={() => {
                    setError(null);
                    setView("restore");
                  }}
                  className="rounded-xl px-10 py-4 text-lg bg-white/60 dark:bg-gray-800/60 hover:bg-white dark:hover:bg-gray-800 shadow-sm hover:shadow-md transition-all duration-300 text-gray-900 dark:text-gray-100 border border-transparent hover:border-gray-200 dark:hover:border-gray-700"
                  variant="secondary"
                >
                  Restore Wallet
                </Button>
              </div>
            </div>
          )}

          {view === "create" && (
            <form onSubmit={handleCreateWallet} className="text-left">
              <Card variant="elevated" padding="lg" className="space-y-6">
              <div className="text-center mb-2">
                <h2 className="text-2xl font-bold text-gray-800 dark:text-gray-200">
                  Create New Wallet
                </h2>
                <p className="text-gray-500 dark:text-gray-400">
                  Set a password to secure your wallet. You can create as many wallets as you need.
                </p>
              </div>

              <div className="space-y-4">
                <Input
                  type={showCreatePass ? "text" : "password"}
                  label="Password"
                  placeholder="Enter a strong password"
                  value={createPassword}
                  onChange={(e) => setCreatePassword(e.target.value)}
                  required
                  suffix={
                    <PasswordToggle
                      show={showCreatePass}
                      onToggle={() => setShowCreatePass(!showCreatePass)}
                    />
                  }
                />
                <Input
                  type={showConfirmPass ? "text" : "password"}
                  label="Confirm Password"
                  placeholder="Confirm your password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  required
                  error={error || undefined}
                  suffix={
                    <PasswordToggle
                      show={showConfirmPass}
                      onToggle={() => setShowConfirmPass(!showConfirmPass)}
                    />
                  }
                />
              </div>

              <div className="flex flex-col gap-3 pt-2">
                <Button
                  type="submit"
                  disabled={isLoading}
                  className="w-full py-6 text-lg"
                >
                  {isLoading ? "Creating..." : "Create Wallet"}
                </Button>
                <Button
                  type="button"
                  variant="ghost"
                  onClick={handleBack}
                  className="w-full"
                >
                  Back
                </Button>
              </div>
              </Card>
            </form>
          )}

          {view === "restore" && (
            <form onSubmit={handleRestoreWallet} className="text-left">
              <Card variant="elevated" padding="lg" className="space-y-6">
              <div className="text-center mb-2">
                <h2 className="text-2xl font-bold text-gray-800 dark:text-gray-200">
                  Restore Wallet
                </h2>
                <p className="text-gray-500 dark:text-gray-400">
                  Enter your seed phrase to recover your wallet
                </p>
              </div>

              <div className="space-y-4">
                <Textarea
                  label="Seed Phrase (Mnemonic)"
                  placeholder="Enter your 24-word seed phrase..."
                  value={mnemonic}
                  onChange={(e) => setMnemonic(e.target.value)}
                  required
                  rows={4}
                />
                <Input
                  type={showRestorePass ? "text" : "password"}
                  label="New Password"
                  placeholder="Set a password for this device"
                  value={restorePassword}
                  onChange={(e) => setRestorePassword(e.target.value)}
                  suffix={
                    <PasswordToggle
                      show={showRestorePass}
                      onToggle={() => setShowRestorePass(!showRestorePass)}
                    />
                  }
                />
                {error && (
                  <p className="text-sm text-red-500 text-center">{error}</p>
                )}
              </div>

              <div className="flex flex-col gap-3 pt-2">
                <Button
                  type="submit"
                  disabled={isLoading}
                  className="w-full py-6 text-lg"
                >
                  {isLoading ? "Restoring..." : "Restore Wallet"}
                </Button>
                <Button
                  type="button"
                  variant="ghost"
                  onClick={handleBack}
                  className="w-full"
                >
                  Back
                </Button>
              </div>
              </Card>
            </form>
          )}

          {generatedMnemonic && (
            <div className="text-center space-y-6 max-w-2xl mx-auto">
              <div className="space-y-4">
                <h2 className="text-2xl font-bold text-gray-800 dark:text-gray-200">
                  Save Your Recovery Phrase
                </h2>
                <p className="text-gray-600 dark:text-gray-400">
                  Write down these 24 words in order. Store them in a safe place. Away from nozy people.
                  You'll need this to recover your wallet.
                </p>
                <div className="bg-yellow-50 dark:bg-yellow-900/20 border-2 border-yellow-200 dark:border-yellow-800/50 rounded-xl p-4 text-left">
                  <p className="text-sm font-semibold text-yellow-800 dark:text-yellow-200 mb-2">
                    ⚠️ Important: Never share your recovery phrase with anyone!
                  </p>
                  <p className="text-xs text-yellow-700 dark:text-yellow-300">
                    Anyone with access to these words can control your wallet.
                  </p>
                </div>
              </div>

              <Card variant="elevated" padding="lg">
                <div className="grid grid-cols-3 gap-3 text-left">
                  {generatedMnemonic.split(" ").map((word, index) => (
                    <div
                      key={index}
                      className="flex items-center gap-2 p-2 rounded-lg bg-gray-50 dark:bg-gray-800/60 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
                    >
                      <span className="text-xs text-gray-500 dark:text-gray-400 font-mono w-6">
                        {index + 1}.
                      </span>
                      <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
                        {word}
                      </span>
                    </div>
                  ))}
                </div>
              </Card>

              <div className="flex flex-col gap-3">
                <Button
                  onClick={handleMnemonicCopied}
                  className="w-full py-6 text-lg"
                  variant={mnemonicCopied ? "secondary" : "primary"}
                >
                  {mnemonicCopied ? "✓ Copied!" : "Copy Recovery Phrase"}
                </Button>
                <Button
                  onClick={handleMnemonicConfirmed}
                  className="w-full py-6 text-lg"
                  disabled={!mnemonicCopied}
                >
                  I've Saved My Recovery Phrase
                </Button>
              </div>
            </div>
          )}

          {view === "securityTips" && (
            <div className="text-left space-y-8 max-w-xl mx-auto animate-fade-in">
              <div className="text-center">
                <h2 className="text-2xl font-bold text-gray-800 dark:text-gray-200">
                  A few security tips
                </h2>
                <p className="text-gray-500 dark:text-gray-400 mt-1">
                  Keep your wallet and funds safe
                </p>
              </div>

              <ul className="space-y-4">
                <li className="flex gap-4 p-4 rounded-2xl bg-white/80 dark:bg-gray-800/60 border border-white/50 dark:border-gray-700/50 shadow-sm">
                  <div className="w-10 h-10 rounded-xl bg-primary/10 flex items-center justify-center shrink-0">
                    <Lock size={20} className="text-primary" />
                  </div>
                  <div>
                    <p className="font-semibold text-gray-900 dark:text-gray-100">Never share your recovery phrase</p>
                    <p className="text-sm text-gray-600 dark:text-gray-400 mt-0.5">
                      Nozy Wallet and no one else will ever ask for it. Anyone with these words can control your funds.
                    </p>
                  </div>
                </li>
                <li className="flex gap-4 p-4 rounded-2xl bg-white/80 dark:bg-gray-800/60 border border-white/50 dark:border-gray-700/50 shadow-sm">
                  <div className="w-10 h-10 rounded-xl bg-primary/10 flex items-center justify-center shrink-0">
                    <Copy size={20} className="text-primary" />
                  </div>
                  <div>
                    <p className="font-semibold text-gray-900 dark:text-gray-100">Verify addresses before sending</p>
                    <p className="text-sm text-gray-600 dark:text-gray-400 mt-0.5">
                      Always double-check the recipient address. Transactions cannot be reversed.
                    </p>
                  </div>
                </li>
                <li className="flex gap-4 p-4 rounded-2xl bg-white/80 dark:bg-gray-800/60 border border-white/50 dark:border-gray-700/50 shadow-sm">
                  <div className="w-10 h-10 rounded-xl bg-primary/10 flex items-center justify-center shrink-0">
                    <Refresh size={20} className="text-primary" />
                  </div>
                  <div>
                    <p className="font-semibold text-gray-900 dark:text-gray-100">Sync your wallet</p>
                    <p className="text-sm text-gray-600 dark:text-gray-400 mt-0.5">
                      After unlocking, sync to load your balance and transaction history from the network.
                    </p>
                  </div>
                </li>
                <li className="flex gap-4 p-4 rounded-2xl bg-white/80 dark:bg-gray-800/60 border border-white/50 dark:border-gray-700/50 shadow-sm">
                  <div className="w-10 h-10 rounded-xl bg-primary/10 flex items-center justify-center shrink-0">
                    <Shield size={20} className="text-primary" />
                  </div>
                  <div>
                    <p className="font-semibold text-gray-900 dark:text-gray-100">Keep your app updated</p>
                    <p className="text-sm text-gray-600 dark:text-gray-400 mt-0.5">
                      Updates include security fixes. Download only from official sources.
                    </p>
                  </div>
                </li>
              </ul>

              <Button
                onClick={handleGetStarted}
                className="w-full py-6 text-lg"
              >
                Get started
              </Button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
