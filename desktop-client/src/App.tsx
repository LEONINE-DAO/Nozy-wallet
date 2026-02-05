import { useQuery } from "@tanstack/react-query";
import { useEffect } from "react";
import toast, { Toaster } from "react-hot-toast";
import { formatErrorForDisplay } from "./utils/errors";
import { walletApi } from "./lib/api";
import { useWalletStore } from "./store/walletStore";
import { useSettingsStore } from "./store/settingsStore";
import { AuthenticatedLayout } from "./layout/AuthenticatedLayout";
import { WelcomePage } from "./pages/Welcome";

function App() {
  const { hasWallet, setHasWallet } = useWalletStore();
  const { darkMode } = useSettingsStore();

  useEffect(() => {
    if (darkMode) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }, [darkMode]);

  const { isLoading: isCheckingWallet } = useQuery({
    queryKey: ["walletStatus"],
    queryFn: async () => {
      const walletExistsToast = toast.loading("Checking wallet status...");
      try {
        const statusRes = await walletApi.getWalletStatus();
        // Wallet exists and is unlocked = show authenticated layout
        // Wallet exists but locked = show welcome page with unlock option
        // No wallet = show welcome page with create/restore options
        setHasWallet(statusRes.data.exists && statusRes.data.unlocked);
        toast.dismiss(walletExistsToast);
        return statusRes.data;
      } catch (e) {
        toast.error(formatErrorForDisplay(e, "Failed to check wallet status"), { id: walletExistsToast });
        setHasWallet(false);
        return { exists: false, unlocked: false };
      }
    },
  });

  if (isCheckingWallet) {
    return (
      <div className="h-screen w-full flex items-center justify-center">
        <div className="flex flex-col items-center gap-6 animate-fade-in">
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
          <div className="text-center space-y-2">
            <p className="text-gray-500 text-sm font-medium animate-pulse">
              Initializing Secure Environment...
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <>
      <Toaster
        position="top-right"
        toastOptions={{
          className:
            "bg-white/90 dark:bg-gray-800/90 backdrop-blur-md border border-white/50 dark:border-gray-700/50 shadow-xl rounded-2xl font-medium text-gray-900 dark:text-gray-100",
          duration: 3000,
          style: {
            padding: "12px 16px",
          },
          success: {
            iconTheme: {
              primary: "#f0a113",
              secondary: "white",
            },
          },
        }}
      />
      {hasWallet ? <AuthenticatedLayout /> : <WelcomePage />}
    </>
  );
}

export default App;
