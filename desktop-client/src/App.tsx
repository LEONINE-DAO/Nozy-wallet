import { useQuery } from "@tanstack/react-query";
import { useEffect } from "react";
import toast, { Toaster, ToastBar } from "react-hot-toast";
import { formatErrorForDisplay } from "./utils/errors";
import { walletApi } from "./lib/api";
import { useWalletStore } from "./store/walletStore";
import { AuthenticatedLayout } from "./layout/AuthenticatedLayout";
import { WelcomePage } from "./pages/Welcome";

const BOOT_STATUS_TIMEOUT_MS = 12_000;

function withTimeout<T>(promise: Promise<T>, ms: number, label: string): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = window.setTimeout(() => {
      reject(new Error(`${label} timed out after ${ms}ms`));
    }, ms);
    promise.then(
      (value) => {
        window.clearTimeout(timer);
        resolve(value);
      },
      (err) => {
        window.clearTimeout(timer);
        reject(err);
      }
    );
  });
}

function toastToneClass(type: string): string {
  if (type === "success") return "nw-toast nw-toast--success";
  if (type === "error") return "nw-toast nw-toast--error";
  if (type === "loading") return "nw-toast nw-toast--loading";
  return "nw-toast";
}

function AppToaster() {
  return (
    <Toaster
      position="top-right"
      gutter={12}
      containerStyle={{ top: 16, right: 16, bottom: undefined, left: undefined }}
      toastOptions={{
        duration: 4000,
        success: {
          duration: 4500,
          iconTheme: { primary: "#d4af37", secondary: "#14110c" },
        },
        error: {
          duration: 6000,
          iconTheme: { primary: "#f87171", secondary: "#1a1010" },
        },
        loading: {
          iconTheme: { primary: "#c8ccd4", secondary: "#14110c" },
        },
      }}
    >
      {(t) => (
        <ToastBar
          toast={t}
          style={{
            background: "transparent",
            boxShadow: "none",
            padding: 0,
            margin: 0,
            maxWidth: "none",
            width: "auto",
          }}
        >
          {({ icon, message }) => (
            <div className={toastToneClass(t.type)} role="status">
              <div className="nw-toast__icon">{icon}</div>
              <div className="nw-toast__message">{message}</div>
            </div>
          )}
        </ToastBar>
      )}
    </Toaster>
  );
}

function App() {
  const { hasWallet, setHasWallet } = useWalletStore();

  // Desktop app is dark-only.
  useEffect(() => {
    document.documentElement.classList.add("dark");
  }, []);

  const { isLoading: isCheckingWallet } = useQuery({
    queryKey: ["walletStatus"],
    retry: false,
    queryFn: async () => {
      const walletExistsToast = toast.loading("Checking wallet status...");
      try {
        const statusRes = await withTimeout(
          walletApi.getWalletStatus(),
          BOOT_STATUS_TIMEOUT_MS,
          "Wallet status check"
        );
        // Wallet exists and is unlocked = show authenticated layout
        // Wallet exists but locked = show welcome page with unlock option
        // No wallet = show welcome page with create/restore options
        setHasWallet(statusRes.data.exists && statusRes.data.unlocked);
        toast.dismiss(walletExistsToast);
        return statusRes.data;
      } catch (e) {
        toast.error(formatErrorForDisplay(e, "Failed to check wallet status"), {
          id: walletExistsToast,
        });
        setHasWallet(false);
        return { exists: false, unlocked: false, has_password: false, address: null };
      }
    },
  });

  if (isCheckingWallet) {
    return (
      <>
        <AppToaster />
        <div className="h-screen w-full flex items-center justify-center bg-[#020b07]">
          <div className="flex flex-col items-center gap-6 animate-fade-in">
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
            <div className="text-center space-y-2">
              <p className="text-gray-400 text-sm font-medium animate-pulse">
                Initializing Secure Environment...
              </p>
            </div>
          </div>
        </div>
      </>
    );
  }

  return (
    <>
      <AppToaster />
      {hasWallet ? <AuthenticatedLayout /> : <WelcomePage />}
    </>
  );
}

export default App;
