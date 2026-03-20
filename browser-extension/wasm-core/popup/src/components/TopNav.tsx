import type { PopupView } from "../store/uiStore";

type Props = {
  view: PopupView;
  onChange: (view: PopupView) => void;
};

const tabs: PopupView[] = ["dashboard", "send", "receive", "settings"];

export function TopNav({ view, onChange }: Props) {
  return (
    <div className="flex gap-2 border-b border-white/10 p-2">
      {tabs.map((tab) => (
        <button
          key={tab}
          onClick={() => onChange(tab)}
          className={`rounded-lg px-3 py-1 text-xs capitalize ${
            view === tab ? "bg-amber-500 text-black" : "bg-white/10 text-white"
          }`}
        >
          {tab}
        </button>
      ))}
    </div>
  );
}

