import { useState } from "react";
import { Button } from "../Button";
import { Input } from "../Input";
import { ArrowLeft, Lock, Shield } from "@solar-icons/react";
import { useSettingsStore } from "../../store/settingsStore";
import { Toggle } from "../Toggle";
import toast from "react-hot-toast";

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
    autoLockEnabled,
    setAutoLockEnabled,
    autoLockMinutes,
    setAutoLockMinutes,
    biometricsEnabled,
    setBiometricsEnabled,
    screenshotProtection,
    setScreenshotProtection,
  } = useSettingsStore();

  const handleToggle =
    (setter: (v: boolean) => void, label: string) => (value: boolean) => {
      setter(value);
      toast.success(`${label} ${value ? "enabled" : "disabled"}`, {
        id: `toggle-${label.toLowerCase().replace(/\s/g, "-")}`,
      });
    };

  const handleChangePassword = async () => {
    if (newPassword !== confirmPassword) {
      toast.error("Passwords don't match!");
      return;
    }

    const passToast = toast.loading("Updating password...");
    try {
      // Implement password change logic
      // await walletApi.changePassword({ currentPassword, newPassword });
      await new Promise((resolve) => setTimeout(resolve, 1000)); // Simulate
      toast.success("Password updated successfully!", { id: passToast });
      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
    } catch (error: any) {
      // console.error("Failed to change password:", error);
      toast.error(error?.message || "Failed to update password", {
        id: passToast,
      });
    }
  };

  return (
    <div className="max-w-2xl mx-auto animate-fade-in">
      <button
        onClick={onBack}
        className="flex items-center gap-2 text-gray-500 hover:text-gray-900 mb-6 transition-colors"
      >
        <ArrowLeft className="w-5 h-5" />
        <span className="font-medium">Back to Settings</span>
      </button>

      <h2 className="text-3xl font-bold text-gray-900 mb-2">
        Security & Privacy
      </h2>
      <p className="text-gray-500 mb-8">
        Manage your wallet security and privacy settings.
      </p>

      <div className="space-y-6">
        <div className="bg-white/60 backdrop-blur-sm rounded-2xl p-6 border border-white/50 shadow-sm">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 rounded-full bg-[#f0a113]-100/50 flex items-center justify-center text-[#f0a113]">
              <Lock size={20} />
            </div>
            <div>
              <h3 className="font-semibold text-gray-900">Change Password</h3>
              <p className="text-sm text-gray-500">
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

        <div className="bg-white/60 backdrop-blur-sm rounded-2xl p-6 border border-white/50 shadow-sm">
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

        <div className="bg-white/60 backdrop-blur-sm rounded-2xl p-6 border border-white/50 shadow-sm">
          <Toggle
            icon={<Shield size={20} />}
            title="Biometric Authentication"
            description="Use fingerprint or face ID"
            checked={biometricsEnabled}
            onChange={handleToggle(setBiometricsEnabled, "Biometrics")}
          />
        </div>

        <div className="bg-white/60 backdrop-blur-sm rounded-2xl p-6 border border-white/50 shadow-sm">
          <h3 className="font-semibold text-gray-900 mb-4">
            Privacy & Interface
          </h3>
          <div className="space-y-2">
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
