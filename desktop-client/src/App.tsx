import { useQuery } from "@tanstack/react-query";
import toast, { Toaster } from "react-hot-toast";
import { walletApi } from "./lib/api";
import { useWalletStore } from "./store/walletStore";
import { AuthenticatedLayout } from "./layout/AuthenticatedLayout";
import { WelcomePage } from "./pages/Welcome";

function App() {
  const { hasWallet, setHasWallet } = useWalletStore();

  // Check if wallet exists on mount
  const { isLoading: isCheckingWallet } = useQuery({
    queryKey: ["walletExists"],
    queryFn: async () => {
      const walletExistsToast = toast.loading("Checking wallet status...");
      try {
        const res = await walletApi.checkWalletExists();
        setHasWallet(res.data.exists);
        toast.dismiss(walletExistsToast);
        return res.data;
      } catch (e) {
        // console.error("Failed to check wallet", e);
        toast.error("Failed to check wallet status", { id: walletExistsToast });
        return { exists: false };
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
            "bg-white/90 backdrop-blur-md border border-white/50 shadow-xl rounded-2xl font-medium text-gray-900",
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
      {/* {hasWallet ? <AuthenticatedLayout /> : <WelcomePage />} */}
      <AuthenticatedLayout />
    </>
  );
}

export default App;
