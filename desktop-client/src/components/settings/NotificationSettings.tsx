import { ArrowLeft, Bell, BellOff } from "@solar-icons/react";
import { Toggle } from "../Toggle";
import { useSettingsStore } from "../../store/settingsStore";
import toast from "react-hot-toast";

interface NotificationSettingsProps {
  onBack: () => void;
}

export function NotificationSettings({ onBack }: NotificationSettingsProps) {
  const {
    transactionNotifs,
    setTransactionNotifs,
    soundEnabled,
    setSoundEnabled,
    dndFrom,
    setDndFrom,
    dndTo,
    setDndTo,
    dndEnabled,
    setDndEnabled,
  } = useSettingsStore();

  const handleToggle =
    (setter: (v: boolean) => void, label: string) => (value: boolean) => {
      setter(value);
      toast.success(`${label} ${value ? "enabled" : "disabled"}`, {
        id: `toggle-${label.toLowerCase().replace(/\s/g, "-")}`,
      });
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

      <h2 className="text-3xl font-bold text-gray-900 mb-2">Notifications</h2>
      <p className="text-gray-500 mb-8">
        Manage your notification preferences and alerts.
      </p>

      <div className="space-y-6">
        <div className="bg-white/60 backdrop-blur-sm rounded-2xl p-6 border border-white/50 shadow-sm">
          <h3 className="font-semibold text-gray-900 mb-4">
            Notification Types
          </h3>
          <div className="space-y-4">
            <Toggle
              icon={<Bell size={20} />}
              title="Transaction Notifications"
              description="Get notified when you send or receive funds"
              checked={transactionNotifs}
              onChange={handleToggle(
                setTransactionNotifs,
                "Transaction notifications"
              )}
            />
          </div>
        </div>

        <div className="bg-white/60 backdrop-blur-sm rounded-2xl p-6 border border-white/50 shadow-sm">
          <h3 className="font-semibold text-gray-900 mb-4">
            Notification Preferences
          </h3>
          <div className="space-y-4">
            <Toggle
              icon={soundEnabled ? <Bell size={20} /> : <BellOff size={20} />}
              title="Sound"
              description="Play sound for notifications"
              checked={soundEnabled}
              onChange={handleToggle(setSoundEnabled, "Notification sounds")}
            />
          </div>
        </div>

        <div className="bg-white/60 backdrop-blur-sm rounded-2xl p-6 border border-white/50 shadow-sm">
          <Toggle
            icon={<BellOff size={20} />}
            title="Do Not Disturb"
            description="Mute all notifications during specific hours"
            checked={dndEnabled}
            onChange={handleToggle(setDndEnabled, "Do not disturb")}
          />

          {dndEnabled && (
            <div className="grid grid-cols-2 gap-4 mt-4 animate-slide-up">
              <div>
                <label className="text-sm font-medium text-gray-700 mb-2 block">
                  From
                </label>
                <input
                  type="time"
                  className="w-full px-4 py-2 rounded-xl border border-gray-200 focus:border-[#f0a113] focus:ring-4 focus:ring-[#f0a113]/10 transition-all font-medium"
                  value={dndFrom}
                  onChange={(e) => setDndFrom(e.target.value)}
                />
              </div>
              <div>
                <label className="text-sm font-medium text-gray-700 mb-2 block">
                  To
                </label>
                <input
                  type="time"
                  className="w-full px-4 py-2 rounded-xl border border-gray-200 focus:border-[#f0a113] focus:ring-4 focus:ring-[#f0a113]/10 transition-all font-medium"
                  value={dndTo}
                  onChange={(e) => setDndTo(e.target.value)}
                />
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
