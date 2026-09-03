import type { ReactNode } from "react";
import zecMark from "../assets/zec.svg";
import { useZecFiatPrice } from "../lib/zecPrice";
import type { PopupView } from "../store/uiStore";
import { CurrencyPicker } from "./CurrencyPicker";
import { CopyButton, Icon, type IconName } from "./ui";

function shortAddr(addr: string): string {
  const a = addr.trim();
  if (!a) return "No address";
  if (a.length <= 20) return a;
  return `${a.slice(0, 10)}…${a.slice(-6)}`;
}

const WALLET_LINKS: Array<{ view: PopupView; label: string; icon: IconName }> = [
  { view: "send", label: "Send", icon: "send" },
  { view: "receive", label: "Receive", icon: "receive" },
  { view: "vote", label: "NU7 vote", icon: "vote" },
  { view: "companion", label: "Local API", icon: "grid" },
  { view: "settings", label: "Settings", icon: "settings" }
];

const PAGE_TITLES: Partial<Record<PopupView, string>> = {
  send: "Send",
  receive: "Receive",
  vote: "NU7 vote",
  companion: "Local API",
  settings: "Settings",
  browser: "Browser"
};

export function FullWalletShell({
  view,
  address,
  children,
  onNavigate,
  onLock
}: {
  view: PopupView;
  address: string | null;
  children: ReactNode;
  onNavigate: (view: PopupView) => void;
  onLock: () => void;
}) {
  const { currency, suggested, setCurrency } = useZecFiatPrice();
  const dashActive = view === "dashboard";
  const stakeActive = view === "crosslink";
  const browserActive = view === "browser";
  const title = PAGE_TITLES[view] ?? view;

  return (
    <div className="nw-keplr">
      <aside className="nw-keplr__side">
        <button type="button" className="nw-keplr__brand" onClick={() => onNavigate("dashboard")}>
          <img src="./logo.jpg" alt="Nozy Wallet" />
        </button>

        <input
          className="nw-keplr__search"
          placeholder="Search chains"
          disabled
          title="Chain search comes when we add more chains"
        />

        <nav className="nw-keplr__nav">
          <button
            type="button"
            className={`nw-keplr__link${dashActive ? " nw-keplr__link--active" : ""}`}
            onClick={() => onNavigate("dashboard")}
          >
            <Icon name="home" size={16} />
            Dashboard
          </button>

          <button
            type="button"
            className={`nw-keplr__link${browserActive ? " nw-keplr__link--active" : ""}`}
            onClick={() => onNavigate("browser")}
          >
            <Icon name="globe" size={16} />
            Browser
          </button>
          <p className="nw-keplr__hint">Sites via NymVPN Fast mode</p>

          <p className="nw-keplr__group">Stake</p>
          <button
            type="button"
            className={`nw-keplr__link nw-keplr__link--sub${stakeActive ? " nw-keplr__link--active" : ""}`}
            onClick={() => onNavigate("crosslink")}
          >
            <img className="nw-keplr__chain-icon" src={zecMark} alt="" />
            Zcash
            <span className="nw-keplr__muted">ZEC</span>
          </button>
          <p className="nw-keplr__hint">Opens Crosslink staking</p>

          <p className="nw-keplr__group">Chains</p>
          <div className="nw-keplr__soon">
            Coming when we add chains — Zcash is the only network today.
          </div>

          <p className="nw-keplr__group">Wallet</p>
          {WALLET_LINKS.map((item) => (
            <button
              key={item.view}
              type="button"
              className={`nw-keplr__link${view === item.view ? " nw-keplr__link--active" : ""}`}
              onClick={() => onNavigate(item.view)}
            >
              <Icon name={item.icon} size={16} />
              {item.label}
            </button>
          ))}
        </nav>

        <div className="nw-keplr__foot">
          <p className="nw-keplr__hint">Feature-net Crosslink · mainnet ZEC</p>
        </div>
      </aside>

      <div className="nw-keplr__main">
        <header className="nw-keplr__top">
          <div>
            <h1 className="nw-keplr__title">
              {dashActive ? (
                <>
                  <Icon name="home" size={18} /> Dashboard
                </>
              ) : stakeActive ? (
                <>
                  <Icon name="shield" size={18} /> Stake
                </>
              ) : browserActive ? (
                <>
                  <Icon name="globe" size={18} /> Browser
                </>
              ) : (
                title
              )}
            </h1>
            {dashActive ? (
              <div className="nw-keplr__tabs">
                <span className="nw-keplr__tab nw-keplr__tab--on">Overview</span>
                <button
                  type="button"
                  className="nw-keplr__tab"
                  onClick={() => onNavigate("crosslink")}
                >
                  Staking
                </button>
              </div>
            ) : stakeActive ? (
              <div className="nw-keplr__tabs">
                <button
                  type="button"
                  className="nw-keplr__tab"
                  onClick={() => onNavigate("dashboard")}
                >
                  Overview
                </button>
                <span className="nw-keplr__tab nw-keplr__tab--on">Staking</span>
              </div>
            ) : null}
          </div>
          <div className="nw-keplr__profile">
            <div className="min-w-0 text-right">
              <p className="text-[13px] font-semibold">Nozy Wallet</p>
              <p className="nw-mono truncate text-[11px]" style={{ color: "var(--nw-muted)" }}>
                {shortAddr(address || "")}
              </p>
            </div>
            {address ? <CopyButton value={address} label="Copy" /> : null}
            <CurrencyPicker currency={currency} suggested={suggested} onChange={setCurrency} />
            <button type="button" className="nw-iconbtn" title="Lock wallet" onClick={onLock}>
              <Icon name="lock" size={14} />
            </button>
          </div>
        </header>
        <div className="nw-keplr__body">{children}</div>
      </div>
    </div>
  );
}
