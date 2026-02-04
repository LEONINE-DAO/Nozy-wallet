import { Button } from "../Button";
import { Input } from "../Input";
import { ArrowLeft } from "@solar-icons/react";
import { useSettingsStore, type FiatCurrency } from "../../store/settingsStore";
import { Toggle } from "../Toggle";
import toast from "react-hot-toast";

interface DisplaySettingsProps {
  onBack: () => void;
}

export function DisplaySettings({ onBack }: DisplaySettingsProps) {
  const {
    showFiatEquivalent,
    setShowFiatEquivalent,
    fiatCurrency,
    setFiatCurrency,
    useLiveFiatPrice,
    setUseLiveFiatPrice,
    customFiatPerZec,
    setCustomFiatPerZec,
  } = useSettingsStore();

  const handleToggle =
    (setter: (v: boolean) => void, label: string) => (value: boolean) => {
      setter(value);
      toast.success(`${label} ${value ? "enabled" : "disabled"}`, {
        id: `toggle-${label.toLowerCase().replace(/\s/g, "-")}`,
      });
    };

  const handleCustomRateChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const raw = e.target.value.trim();
    if (raw === "") {
      setCustomFiatPerZec(null);
      return;
    }
    const n = parseFloat(raw);
    if (!Number.isNaN(n) && n >= 0) setCustomFiatPerZec(n);
  };

  return (
    <div className="max-w-2xl mx-auto animate-fade-in">
      <Button
        variant="ghost"
        onClick={onBack}
        className="mb-6 gap-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100"
      >
        <ArrowLeft size={20} />
        Back
      </Button>

      <h2 className="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-8">
        Display
      </h2>

      <div className="space-y-6">
        <div className="p-4 rounded-xl bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm border border-white/50 dark:border-gray-700/50">
          <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-4">
            Fiat equivalent
          </h3>
          <p className="text-sm text-gray-500 dark:text-gray-400 mb-4">
            Show transaction amounts in fiat (e.g. USD) in History. Uses live
            price from CoinGecko or a custom rate.
          </p>
          <Toggle
            title="Show fiat equivalent"
            description="Display fiat value next to ZEC in transaction history"
            checked={showFiatEquivalent}
            onChange={handleToggle(setShowFiatEquivalent, "Fiat equivalent")}
          />
          {showFiatEquivalent && (
            <div className="mt-4 space-y-4 pl-0">
              <div>
                <label className="text-sm font-medium text-gray-700 dark:text-gray-300 block mb-2">
                  Currency
                </label>
                <select
                  value={fiatCurrency}
                  onChange={(e) =>
                    setFiatCurrency(e.target.value as FiatCurrency)
                  }
                  className="h-11 w-full max-w-[140px] rounded-lg border border-gray-200/60 dark:border-gray-600 bg-white/60 dark:bg-gray-800/60 px-3 py-2 text-sm text-gray-700 dark:text-gray-300 focus:outline-none focus:ring-2 focus:ring-primary"
                >
                  <option value="USD">USD</option>
                  <option value="EUR">EUR</option>
                </select>
              </div>
              <Toggle
                title="Use live price"
                description="Fetch ZEC price from CoinGecko (cached 5 min)"
                checked={useLiveFiatPrice}
                onChange={handleToggle(setUseLiveFiatPrice, "Live price")}
              />
              {!useLiveFiatPrice && (
                <div>
                  <label className="text-sm font-medium text-gray-700 dark:text-gray-300 block mb-2">
                    Custom rate ({fiatCurrency} per 1 ZEC)
                  </label>
                  <Input
                    type="number"
                    min={0}
                    step={0.01}
                    placeholder="e.g. 25.50"
                    value={
                      customFiatPerZec != null ? String(customFiatPerZec) : ""
                    }
                    onChange={handleCustomRateChange}
                  />
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
