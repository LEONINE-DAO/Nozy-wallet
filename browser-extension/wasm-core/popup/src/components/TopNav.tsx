import type { PopupView } from "../store/uiStore";

type Props = {
  view: PopupView;
  onChange: (view: PopupView) => void;
};

const tabs: PopupView[] = ["dashboard", "send", "receive", "companion", "settings"];

const tabLabel: Record<PopupView, string> = {
  welcome: "Welcome",
  unlock: "Unlock",
  dashboard: "Home",
  send: "Send",
  receive: "Receive",
  companion: "API",
  settings: "Settings"
};

export function TopNav({ view, onChange }: Props) {
  return (
    <div className="flex flex-wrap gap-1 border-b border-white/10 p-2">
      {tabs.map((tab) => (
        <button
          key={tab}
          onClick={() => onChange(tab)}
          className={`rounded-lg px-2.5 py-1 text-[11px] font-medium ${
            view === tab ? "bg-amber-500 text-black" : "bg-white/10 text-white"
          }`}
        >
          {tabLabel[tab]}
        </button>
      ))}
    </div>
  );
}

