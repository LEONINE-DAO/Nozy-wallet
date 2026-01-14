import { useState } from "react";
import { Eye, EyeClosed } from "@solar-icons/react";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { useWalletStore } from "../store/walletStore";
import { walletApi } from "../lib/api";
import toast from "react-hot-toast";

type ViewState = "initial" | "create" | "restore";

export function WelcomePage() {
  const [view, setView] = useState<ViewState>("initial");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { setHasWallet } = useWalletStore();

  const [createPassword, setCreatePassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showCreatePass, setShowCreatePass] = useState(false);
  const [showConfirmPass, setShowConfirmPass] = useState(false);
  const [generatedMnemonic, setGeneratedMnemonic] = useState<string | null>(null);
  const [mnemonicCopied, setMnemonicCopied] = useState(false);

  const [mnemonic, setMnemonic] = useState("");
  const [restorePassword, setRestorePassword] = useState("");
  const [showRestorePass, setShowRestorePass] = useState(false);

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
        toast.success("Wallet created! Please save your recovery phrase.", { id: createToast });
      } else {
        toast.success("Wallet created successfully!", { id: createToast });
        setHasWallet(true);
      }
    } catch (err: any) {
      const errMsg =
        err?.message || "Failed to create wallet. Please try again.";
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
      toast.success("Wallet restored successfully!", { id: restoreToast });
      setHasWallet(true);
    } catch (err: any) {
      const errMsg =
        err?.message || "Failed to restore wallet. Please check your mnemonic.";
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
      className="focus:outline-none hover:text-primary transition-colors text-gray-500"
    >
      {show ? <EyeClosed size={20} /> : <Eye size={20} />}
    </button>
  );

  return (
    <div className="h-screen w-full bg-background-subtle text-gray-900 font-sans flex flex-col justify-center items-center overflow-auto relative">
      <div className="absolute top-0 left-0 w-full h-96 bg-linear-to-b from-primary-50 to-transparent pointer-events-none" />

      <div className="container mx-auto px-6 py-12 relative z-10 flex flex-col items-center">
        <div className="max-w-xl w-full text-center space-y-8 animate-fade-in">
          <div className="flex justify-center mb-6">
            <div className="aspect-square w-64 rounded-2xl flex items-center justify-center">
              <img
                src="/logo.png"
                alt="Nozy Wallet"
                className="aspect-square w-64 object-contain"
                onError={(e) => {
                  e.currentTarget.style.display = "none";
                }}
              />
            </div>
          </div>

          <h1 className="text-3xl md:text-5xl font-extrabold tracking-tight text-gray-600 animate-slide-up animation-delay-200">
            Privacy by Default.
          </h1>

          {view === "initial" && (
            <div className="space-y-8 animate-slide-up animation-delay-200">
              <p className="text-xl text-gray-600 leading-relaxed max-w-2xl mx-auto">
                Experience the future of private finance. Nozy Wallet combines
                speed and anonymity in a beautiful, easy-to-use interface.
              </p>
              <div className="mt-8 flex flex-col sm:flex-row gap-4 justify-center">
                <Button
                  size="md"
                  onClick={() => setView("create")}
                  className="rounded-xl px-10 py-4 text-lg bg-white hover:bg-primary shadow-lg hover:shadow-none transition-all duration-300 text-black"
                >
                  Create New Wallet
                </Button>
                <Button
                  size="md"
                  onClick={() => setView("restore")}
                  className="rounded-xl px-10 py-4 text-lg bg-white/60 hover:bg-white shadow-sm hover:shadow-md transition-all duration-300 text-black border border-transparent hover:border-gray-200"
                  variant="secondary"
                >
                  Restore Wallet
                </Button>
              </div>
            </div>
          )}

          {view === "create" && (
            <form
              onSubmit={handleCreateWallet}
              className="text-left space-y-6"
            >
              <div className="text-center mb-6">
                <h2 className="text-2xl font-bold text-gray-800">
                  Create New Wallet
                </h2>
                <p className="text-gray-500">
                  Set a password to secure your wallet
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
            </form>
          )}

          {view === "restore" && (
            <form
              onSubmit={handleRestoreWallet}
              className="p-8 text-left space-y-6"
            >
              <div className="text-center mb-6">
                <h2 className="text-2xl font-bold text-gray-800">
                  Restore Wallet
                </h2>
                <p className="text-gray-500">
                  Enter your seed phrase to recover your wallet
                </p>
              </div>

              <div className="space-y-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium text-gray-700">
                    Seed Phrase (Mnemonic)
                  </label>
                  <textarea
                    className="w-full rounded-lg border border-gray-200/60 bg-white/60 px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary resize-none placeholder:text-gray-400"
                    placeholder="Enter your 24-word seed phrase..."
                    value={mnemonic}
                    onChange={(e) => setMnemonic(e.target.value)}
                    required
                  />
                </div>
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
            </form>
          )}

          {generatedMnemonic && (
            <div className="p-8 text-center space-y-6 max-w-2xl mx-auto">
              <div className="space-y-4">
                <h2 className="text-2xl font-bold text-gray-800">
                  Save Your Recovery Phrase
                </h2>
                <p className="text-gray-600">
                  Write down these 24 words in order. Store them in a safe place.
                  You'll need this to recover your wallet.
                </p>
                <div className="bg-yellow-50 border-2 border-yellow-200 rounded-xl p-4 text-left">
                  <p className="text-sm font-semibold text-yellow-800 mb-2">
                    ⚠️ Important: Never share your recovery phrase with anyone!
                  </p>
                  <p className="text-xs text-yellow-700">
                    Anyone with access to these words can control your wallet.
                  </p>
                </div>
              </div>

              <div className="bg-white/80 backdrop-blur-sm rounded-xl border-2 border-gray-200 p-6">
                <div className="grid grid-cols-3 gap-3 text-left">
                  {generatedMnemonic.split(" ").map((word, index) => (
                    <div
                      key={index}
                      className="flex items-center gap-2 p-2 rounded-lg bg-gray-50 hover:bg-gray-100 transition-colors"
                    >
                      <span className="text-xs text-gray-500 font-mono w-6">
                        {index + 1}.
                      </span>
                      <span className="text-sm font-medium text-gray-900">
                        {word}
                      </span>
                    </div>
                  ))}
                </div>
              </div>

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
        </div>
      </div>
    </div>
  );
}
