import { useState, useEffect } from "react";
import { Button } from "../Button";
import { Input } from "../Input";
import { ArrowLeft, Lock, Shield, Moon, Sun } from "@solar-icons/react";
import { useSettingsStore } from "../../store/settingsStore";
import { Toggle } from "../Toggle";
import toast from "react-hot-toast";
import { formatErrorForDisplay } from "../../utils/errors";
import { walletApi } from "../../lib/api";

interface SecuritySettingsProps {
  onBack: () => void;
}

export function SecuritySettings({ onBack }: SecuritySettingsProps) {
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");

  const {
    showNavigationLabels,
    setShowNavigationLabels,
    hideBalance,
    setHideBalance,
    darkMode,
    setDarkMode,
    autoLockEnabled,
    setAutoLockEnabled,
    autoLockMinutes,
    setAutoLockMinutes,
    biometricsEnabled,
    setBiometricsEnabled,
    screenshotProtection,
    setScreenshotProtection,
  } = useSettingsStore();

  // Apply dark mode to document root
  useEffect(() => {
    if (darkMode) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }, [darkMode]);

  const handleToggle =
    (setter: (v: boolean) => void, label: string) => (value: boolean) => {
      setter(value);
      toast.success(`${label} ${value ? "enabled" : "disabled"}`, {
        id: `toggle-${label.toLowerCase().replace(/\s/g, "-")}`,
      });
    };

  const handleChangePassword = async () => {
    if (!currentPassword) {
      toast.error("Please enter your current password");
      return;
    }

    if (!newPassword) {
      toast.error("Please enter a new password");
      return;
    }

    if (newPassword !== confirmPassword) {
      toast.error("New passwords don't match!");
      return;
    }

    if (newPassword.length < 8) {
      toast.error("New password must be at least 8 characters long");
      return;
    }

    const passToast = toast.loading("Updating password...");
    try {
      await walletApi.changePassword({
        current_password: currentPassword,
        new_password: newPassword,
      });
      toast.success("Password updated successfully!", { id: passToast });
      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
    } catch (error: unknown) {
      toast.error(formatErrorForDisplay(error, "Failed to update password"), { id: passToast });
    }
  };

  return (
    <div className="max-w-2xl mx-auto animate-fade-in">
      <button
        onClick={onBack}
        className="flex items-center gap-2 text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 mb-6 transition-colors"
      >
        <ArrowLeft className="w-5 h-5" />
        <span className="font-medium">Back to Settings</span>
      </button>

      <h2 className="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-2">
        Security & Privacy
      </h2>
      <p className="text-gray-500 dark:text-gray-400 mb-8">
        Manage your wallet security and privacy settings.
      </p>

      <div className="space-y-6">
        <div className="bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm rounded-2xl p-6 border border-white/50 dark:border-gray-700/50 shadow-sm">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 rounded-full bg-[#f0a113]-100/50 flex items-center justify-center text-[#f0a113]">
              <Lock size={20} />
            </div>
            <div>
              <h3 className="font-semibold text-gray-900 dark:text-gray-100">Change Password</h3>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Update your wallet password
              </p>
            </div>
          </div>

          <div className="space-y-4">
            <Input
              type="password"
              label="Current Password"
              value={currentPassword}
              onChange={(e) => setCurrentPassword(e.target.value)}
              placeholder="Enter current password"
            />
            <Input
              type="password"
              label="New Password"
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
              placeholder="Enter new password"
            />
            <Input
              type="password"
              label="Confirm New Password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              placeholder="Confirm new password"
            />
            <Button
              onClick={handleChangePassword}
              disabled={!currentPassword || !newPassword || !confirmPassword}
              className="w-full"
            >
              Update Password
            </Button>
          </div>
        </div>

        <div className="bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm rounded-2xl p-6 border border-white/50 dark:border-gray-700/50 shadow-sm">
          <Toggle
            icon={<Shield size={20} />}
            title="Auto-Lock"
            description="Automatically lock wallet after inactivity"
            checked={autoLockEnabled}
            onChange={handleToggle(setAutoLockEnabled, "Auto-lock")}
          />

          {autoLockEnabled && (
            <div className="mt-4 animate-slide-up">
              <Input
                type="number"
                label="Auto-lock after (minutes)"
                value={autoLockMinutes}
                onChange={(e) => setAutoLockMinutes(e.target.value)}
                min="1"
                max="60"
              />
            </div>
          )}
        </div>

        <div className="bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm rounded-2xl p-6 border border-white/50 dark:border-gray-700/50 shadow-sm">
          <Toggle
            icon={<Shield size={20} />}
            title="Biometric Authentication"
            description="Use fingerprint or face ID"
            checked={biometricsEnabled}
            onChange={handleToggle(setBiometricsEnabled, "Biometrics")}
          />
        </div>

        <div className="bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm rounded-2xl p-6 border border-white/50 dark:border-gray-700/50 shadow-sm">
          <h3 className="font-semibold text-gray-900 dark:text-gray-100 mb-4">
            Privacy & Interface
          </h3>
          <div className="space-y-2">
            <Toggle
              icon={darkMode ? <Sun size={20} /> : <Moon size={20} />}
              title="Dark Mode"
              description="Switch between light and dark theme"
              checked={darkMode}
              onChange={handleToggle(setDarkMode, "Dark mode")}
            />
            <Toggle
              title="Hide Balance"
              description="Hide balance on home screen"
              checked={hideBalance}
              onChange={handleToggle(setHideBalance, "Privacy mode")}
            />
            <Toggle
              title="Show Navigation Labels"
              description="Display text labels in the top header"
              checked={showNavigationLabels}
              onChange={handleToggle(
                setShowNavigationLabels,
                "Navigation labels"
              )}
            />
            <Toggle
              title="Screenshot Protection"
              description="Prevent screenshots of sensitive data"
              checked={screenshotProtection}
              onChange={handleToggle(
                setScreenshotProtection,
                "Screenshot protection"
              )}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
