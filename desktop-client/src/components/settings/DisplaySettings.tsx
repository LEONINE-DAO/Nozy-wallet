import { Input } from "../Input";
import { Select } from "../Select";
import { useSettingsStore } from "../../store/settingsStore";
import { FIAT_CURRENCIES } from "../../lib/fiatCurrencies";
import { Toggle } from "../Toggle";
import { SettingsBackButton } from "./SettingsBackButton";
import toast from "react-hot-toast";

interface DisplaySettingsProps {
  onBack: () => void;
}

export function DisplaySettings({ onBack }: DisplaySettingsProps) {
  const {
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
    <div className="max-w-2xl mx-auto animate-fade-in pb-8">
      <SettingsBackButton onClick={onBack} />

      <h2 className="text-3xl font-bold text-white mb-2">Display</h2>
      <p className="text-base text-gray-200 mb-8">
        Choose how balances and prices appear in the wallet.
      </p>

      <div className="rounded-2xl bg-gray-800 border border-gray-600 p-6 shadow-lg">
        <h3 className="text-lg font-semibold text-white mb-2">Fiat equivalent</h3>
        <p className="text-sm text-gray-200 mb-5 leading-relaxed">
          Fiat values are shown on your balance and transaction history. Use a live
          CoinGecko price or set a custom rate.
        </p>
        <div className="space-y-5">
          <Select
            label="Currency"
            value={fiatCurrency}
            onChange={(e) => setFiatCurrency(e.target.value as typeof fiatCurrency)}
            className="max-w-xs bg-gray-900 border-gray-500 text-white"
          >
            {FIAT_CURRENCIES.map((c) => (
              <option key={c.code} value={c.code}>
                {c.code} — {c.name}
              </option>
            ))}
          </Select>
          <div className="rounded-xl bg-gray-900/70 border border-gray-600 px-4">
            <Toggle
              title="Use live price"
              description="Fetch ZEC price from CoinGecko (cached 5 min)"
              checked={useLiveFiatPrice}
              onChange={handleToggle(setUseLiveFiatPrice, "Live price")}
            />
          </div>
          {!useLiveFiatPrice && (
            <div>
              <label className="text-sm font-semibold text-white block mb-2">
                Custom rate ({fiatCurrency} per 1 ZEC)
              </label>
              <Input
                type="number"
                min={0}
                step={0.01}
                placeholder="e.g. 25.50"
                value={customFiatPerZec != null ? String(customFiatPerZec) : ""}
                onChange={handleCustomRateChange}
                className="bg-gray-900 border-gray-500 text-white placeholder:text-gray-400"
              />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
