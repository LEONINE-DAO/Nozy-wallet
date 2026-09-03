import type { PopupView } from "../store/uiStore";
import { Icon, type IconName } from "./ui";

/** Views reachable from the "More" sheet rather than a primary tab. */
export const MORE_VIEWS: PopupView[] = ["vote", "crosslink", "companion", "browser"];

const PRIMARY_TABS: Array<{ view: PopupView; label: string; icon: IconName }> = [
  { view: "dashboard", label: "Home", icon: "home" },
  { view: "send", label: "Send", icon: "send" },
  { view: "receive", label: "Receive", icon: "receive" },
  { view: "settings", label: "Settings", icon: "settings" }
];

const MORE_ITEMS: Array<{
  view: PopupView;
  title: string;
  description: string;
  icon: IconName;
}> = [
  {
    view: "vote",
    title: "NU7 coinholder vote",
    description: "Export notes, delegate or cast a ballot",
    icon: "vote"
  },
  {
    view: "crosslink",
    title: "Crosslink guardian",
    description: "Opens the full-page staking GUI (pick a finalizer, stake)",
    icon: "shield"
  },
  {
    view: "browser",
    title: "Browser",
    description: "Open sites from Full wallet — use NymVPN Fast mode",
    icon: "globe"
  },
  {
    view: "companion",
    title: "Local API",
    description: "Companion URLs, API key, compact sync",
    icon: "grid"
  }
];

export function BottomNav({
  view,
  onChange,
  onOpenMore,
  moreOpen
}: {
  view: PopupView;
  onChange: (view: PopupView) => void;
  onOpenMore: () => void;
  moreOpen: boolean;
}) {
  const moreActive = moreOpen || MORE_VIEWS.includes(view);
  return (
    <nav className="nw-bottomnav flex shrink-0 items-stretch">
      {PRIMARY_TABS.map((tab) => (
        <button
          key={tab.view}
          type="button"
          className={`nw-navbtn${view === tab.view && !moreOpen ? " nw-navbtn--active" : ""}`}
          onClick={() => onChange(tab.view)}
        >
          <Icon name={tab.icon} size={18} />
          {tab.label}
        </button>
      ))}
      <button
        type="button"
        className={`nw-navbtn${moreActive ? " nw-navbtn--active" : ""}`}
        onClick={onOpenMore}
      >
        <Icon name="grid" size={18} />
        More
      </button>
    </nav>
  );
}

export function MoreSheet({
  onSelect,
  onClose,
  onLogout,
  onOpenFull
}: {
  onSelect: (view: PopupView) => void;
  onClose: () => void;
  onLogout: () => Promise<void>;
  onOpenFull?: () => void;
}) {
  return (
    <div className="absolute inset-0 z-20 flex flex-col justify-end">
      <button
        type="button"
        aria-label="Close"
        className="absolute inset-0 cursor-default"
        style={{ background: "rgba(0,0,0,0.6)" }}
        onClick={onClose}
      />
      <div
        className="relative rounded-t-2xl p-3"
        style={{
          background: "var(--nw-surface)",
          borderTop: "1px solid var(--nw-border)"
        }}
      >
        <div className="mx-auto mb-3 h-1 w-10 rounded-full" style={{ background: "var(--nw-border-strong)" }} />
        <div className="nw-card nw-card--flush">
          {onOpenFull && (
            <button
              type="button"
              className="nw-row"
              onClick={() => {
                onClose();
                onOpenFull();
              }}
            >
              <span className="flex min-w-0 items-center gap-3">
                <span style={{ color: "var(--nw-platinum)" }}>
                  <Icon name="expand" size={17} />
                </span>
                <span className="min-w-0">
                  <span className="block text-[13px] font-semibold">Full wallet</span>
                  <span className="nw-hint mt-0.5 block">
                    Open in a browser tab like Keplr or Brave
                  </span>
                </span>
              </span>
              <span style={{ color: "var(--nw-faint)" }}>
                <Icon name="chevron" size={14} />
              </span>
            </button>
          )}
          {MORE_ITEMS.map((item) => (
            <button
              key={item.view}
              type="button"
              className="nw-row"
              onClick={() => onSelect(item.view)}
            >
              <span className="flex min-w-0 items-center gap-3">
                <span style={{ color: "var(--nw-platinum)" }}>
                  <Icon name={item.icon} size={17} />
                </span>
                <span className="min-w-0">
                  <span className="block text-[13px] font-semibold">{item.title}</span>
                  <span className="nw-hint mt-0.5 block">{item.description}</span>
                </span>
              </span>
              <span style={{ color: "var(--nw-faint)" }}>
                <Icon name="chevron" size={14} />
              </span>
            </button>
          ))}
          <button
            type="button"
            className="nw-row"
            onClick={() => {
              onClose();
              void onLogout();
            }}
          >
            <span className="flex min-w-0 items-center gap-3">
              <span style={{ color: "var(--nw-danger)" }}>
                <Icon name="lock" size={17} />
              </span>
              <span className="min-w-0">
                <span className="block text-[13px] font-semibold" style={{ color: "var(--nw-danger)" }}>
                  Log out
                </span>
                <span className="nw-hint mt-0.5 block">
                  Lock this wallet without deleting its data
                </span>
              </span>
            </span>
          </button>
        </div>
      </div>
    </div>
  );
}
