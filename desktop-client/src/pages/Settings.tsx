import { useState } from "react";
import { Button } from "../components/Button";
import toast from "react-hot-toast";
import {
  User,
  Shield,
  Bell,
  BoltCircle,
  AltArrowRight,
  Sun,
} from "@solar-icons/react";
import { NetworkSettings } from "../components/settings/NetworkSettings";
import { AccountSettings } from "../components/settings/AccountSettings";
import { SecuritySettings } from "../components/settings/SecuritySettings";
import { NotificationSettings } from "../components/settings/NotificationSettings";
import { DisplaySettings } from "../components/settings/DisplaySettings";
import { walletApi } from "../lib/api";
import { useWalletStore } from "../store/walletStore";

type SettingsSection =
  | "main"
  | "network"
  | "account"
  | "security"
  | "notifications"
  | "display";

export function SettingsPage() {
  const [activeSection, setActiveSection] = useState<SettingsSection>("main");
  const { setHasWallet, setBalance, setAddress } = useWalletStore();

  const handleLogout = async () => {
    const logoutToast = toast.loading("Logging out...");
    try {
      await walletApi.lockWallet();
      setHasWallet(false);
      setBalance(0);
      setAddress("");
      toast.success("Logged out successfully", { id: logoutToast });
    } catch (error) {
      toast.error("Failed to log out", { id: logoutToast });
    }
  };

  if (activeSection === "network") {
    return <NetworkSettings onBack={() => setActiveSection("main")} />;
  }

  if (activeSection === "account") {
    return <AccountSettings onBack={() => setActiveSection("main")} />;
  }

  if (activeSection === "security") {
    return <SecuritySettings onBack={() => setActiveSection("main")} />;
  }

  if (activeSection === "notifications") {
    return <NotificationSettings onBack={() => setActiveSection("main")} />;
  }

  if (activeSection === "display") {
    return <DisplaySettings onBack={() => setActiveSection("main")} />;
  }

  return (
    <div className="max-w-2xl mx-auto animate-fade-in">
      <h2 className="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-8">Settings</h2>

      <div className="space-y-4">
        <SettingsItem
          icon={<User className="text-primary-600" />}
          title="Account Information"
          description="Manage your keys and seeds"
          onClick={() => setActiveSection("account")}
        />
        <SettingsItem
          icon={<BoltCircle className="text-primary-600" />}
          title="Network & Node"
          description="Configure API backend connection"
          onClick={() => setActiveSection("network")}
        />
        <SettingsItem
          icon={<Shield className="text-primary-600" />}
          title="Security & Privacy"
          description="PIN, Password, and Privacy settings"
          onClick={() => setActiveSection("security")}
        />
        <SettingsItem
          icon={<Bell className="text-primary-600" />}
          title="Notifications"
          description="Manage alerts and push notifications"
          onClick={() => setActiveSection("notifications")}
        />
        <SettingsItem
          icon={<Sun className="text-primary-600" />}
          title="Display"
          description="Fiat equivalent and display options"
          onClick={() => setActiveSection("display")}
        />
      </div>

      <div className="mt-12 pt-8 border-t border-gray-100 dark:border-gray-800">
        <Button
          variant="outline"
          className="w-full bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 border border-gray-200 dark:border-gray-700 shadow-sm"
          onClick={handleLogout}
        >
          Log Out
        </Button>
        <p className="text-center text-xs text-gray-400 dark:text-gray-500 mt-4">
          Version 1.0.0 (Beta)
        </p>
      </div>
    </div>
  );
}

function SettingsItem({
  icon,
  title,
  description,
  onClick,
}: {
  icon: React.ReactNode;
  title: string;
  description: string;
  onClick: () => void;
}) {
  return (
    <div
      onClick={onClick}
      className="p-4 rounded-xl bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm border border-white/50 dark:border-gray-700/50 hover:border-primary/30 dark:hover:border-primary/40 hover:shadow-md hover:shadow-primary/5 cursor-pointer transition-all duration-200 flex items-center gap-4 group"
    >
      <div className="w-10 h-10 rounded-full bg-primary-100/50 dark:bg-primary-900/30 flex items-center justify-center group-hover:bg-primary-50 dark:group-hover:bg-primary-900/50 transition-colors">
        {icon}
      </div>
      <div className="flex-1">
        <h3 className="font-medium text-gray-900 dark:text-gray-100">{title}</h3>
        <p className="text-sm text-gray-500 dark:text-gray-400">{description}</p>
      </div>
      <div className="text-gray-400 dark:text-gray-500 group-hover:text-primary transition-colors">
        <AltArrowRight size={20} />
      </div>
    </div>
  );
}
