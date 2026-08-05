import { useState } from "react";
import { Link } from "react-router-dom";
import logoUrl from "../assets/logo-mark-clean.png";

const VALUES = [
  {
    title: "Shielded by default",
    body: "Orchard and Ironwood on-chain privacy — no transparent sends.",
  },
  {
    title: "Your keys, your node",
    body: "Companion mode talks to an API you run. Strongest path: Zebrad you control.",
  },
  {
    title: "Open source",
    body: "Auditable Rust core and mobile UI. No ads. No tracking SDKs in the app.",
  },
  {
    title: "Honest hosting",
    body: "Until Nozy funds its own Zebrad, phone-only users need a node — yours or someone you trust.",
  },
] as const;

const STEPS = [
  {
    n: "01",
    title: "Run your API",
    body: "Start nozywallet-api on your PC or VPS, pointed at a live Zebrad.",
  },
  {
    n: "02",
    title: "Connect the phone",
    body: "Enter your API URL (and key if required). Unlock or restore your wallet.",
  },
  {
    n: "03",
    title: "Sync & send",
    body: "Watch shielded balance update, receive with a unified address, send Orchard or Ironwood ZEC.",
  },
] as const;

const FAQS = [
  {
    q: "Is this a full node on my phone?",
    a: "No. The phone is a companion UI. Wallet sync and proving run on the NozyWallet API you configure, which talks to Zebrad. That keeps the 300 GB chain off your handset.",
  },
  {
    q: "Do I need my own computer?",
    a: "For maximum privacy and for sync today, yes — run Zebrad + the API yourself. If you have no PC, you can use another operator’s API only if you accept that they can see connection metadata and wallet data on their server.",
  },
  {
    q: "When will App Store / Play links work?",
    a: "We are finishing production builds, listing assets, and store accounts. This page will get the official badges when those builds ship. Privacy policy is already live for review.",
  },
  {
    q: "How does this relate to desktop?",
    a: "Same Orchard + Ironwood wallet family. Desktop and CLI are for operators at a keyboard; mobile is for the same stack when you are away from home — ideally still talking to your own API.",
  },
] as const;

function FaqItem({
  q,
  a,
  open,
  onToggle,
}: {
  q: string;
  a: string;
  open: boolean;
  onToggle: () => void;
}) {
  return (
    <div className="mobile-faq-item">
      <button
        type="button"
        className="mobile-faq-q"
        onClick={onToggle}
        aria-expanded={open}
      >
        <span>{q}</span>
        <span className={`mobile-faq-chevron ${open ? "open" : ""}`} aria-hidden>
          ⌄
        </span>
      </button>
      <div className={`mobile-faq-a ${open ? "open" : ""}`}>
        <p>{a}</p>
      </div>
    </div>
  );
}

const Mobile = () => {
  const [openFaq, setOpenFaq] = useState<number | null>(0);

  return (
    <div className="mobile-page">
      <section className="mobile-hero">
        <div className="mobile-hero-glow" aria-hidden />
        <div className="mobile-hero-grid" aria-hidden />
        <div className="mobile-hero-inner">
          <img
            src={logoUrl}
            alt="NozyWallet"
            className="mobile-hero-logo"
          />
          <h1 className="mobile-hero-title">
            Shielded ZEC
            <br />
            <span>in your pocket.</span>
          </h1>
          <p className="mobile-hero-sub">
            NozyWallet Mobile is a privacy companion — Orchard and Ironwood
            balance, receive, and send from your phone, synced through an API
            and Zebrad you control.
          </p>
          <div className="mobile-hero-ctas">
            <a href="#get-app" className="mobile-btn mobile-btn-primary">
              Get the app
            </a>
            <a href="#how" className="mobile-btn mobile-btn-ghost">
              How it works
            </a>
          </div>
        </div>
      </section>

      <section className="mobile-values" aria-label="What you get">
        <div className="mobile-values-inner">
          {VALUES.map((v) => (
            <div key={v.title} className="mobile-value">
              <h2>{v.title}</h2>
              <p>{v.body}</p>
            </div>
          ))}
        </div>
      </section>

      <section className="mobile-pair" id="pair">
        <div className="mobile-pair-inner">
          <p className="mobile-kicker">Desktop + phone</p>
          <h2 className="mobile-section-title">
            One wallet family.
            <br />
            Two places you live.
          </h2>
          <p className="mobile-section-lede">
            Keep operators at the keyboard on CLI or Desktop. Take the same
            shielded stack with you — phone talks HTTPS to your API, API talks
            JSON-RPC to Zebrad. Not a copy of someone else’s air-gap story —
            a Nozy companion built for Orchard and Ironwood.
          </p>
          <div className="mobile-pair-split">
            <div>
              <h3>On the phone</h3>
              <ul>
                <li>Unlock, restore, dashboard</li>
                <li>Receive QR &amp; send flow</li>
                <li>Settings for your API URL</li>
              </ul>
            </div>
            <div className="mobile-pair-plus" aria-hidden>
              +
            </div>
            <div>
              <h3>On your machine</h3>
              <ul>
                <li>nozywallet-api</li>
                <li>Zebrad you run or trust</li>
                <li>Keys &amp; scan state for companion mode</li>
              </ul>
            </div>
          </div>
        </div>
      </section>

      <section className="mobile-how" id="how">
        <div className="mobile-how-inner">
          <p className="mobile-kicker">Setup</p>
          <h2 className="mobile-section-title">Three steps to go.</h2>
          <ol className="mobile-steps">
            {STEPS.map((s) => (
              <li key={s.n}>
                <span className="mobile-step-n">{s.n}</span>
                <div>
                  <h3>{s.title}</h3>
                  <p>{s.body}</p>
                </div>
              </li>
            ))}
          </ol>
        </div>
      </section>

      <section className="mobile-faq" id="mobile-faq">
        <div className="mobile-faq-inner">
          <p className="mobile-kicker">Questions</p>
          <h2 className="mobile-section-title">Before you install</h2>
          <div className="mobile-faq-list">
            {FAQS.map((item, i) => (
              <FaqItem
                key={item.q}
                q={item.q}
                a={item.a}
                open={openFaq === i}
                onToggle={() => setOpenFaq(openFaq === i ? null : i)}
              />
            ))}
          </div>
        </div>
      </section>

      <section className="mobile-get" id="get-app">
        <div className="mobile-get-inner">
          <h2 className="mobile-section-title">Mobile beta coming soon</h2>
          <p className="mobile-section-lede">
            Store listing prep is underway — not on App Store or Google Play yet.
            Until then, use CLI Lite, Desktop, or the Sweet Chili browser extension.
            Privacy policy for store submission is already published.
          </p>
          <div className="mobile-store-row">
            <div className="mobile-store-badge" aria-disabled="true">
              App Store — beta soon
            </div>
            <div className="mobile-store-badge" aria-disabled="true">
              Google Play — beta soon
            </div>
          </div>
          <div className="mobile-get-links">
            <a
              href="https://leonine-dao.github.io/Nozy-wallet/privacy/"
              target="_blank"
              rel="noopener noreferrer"
            >
              Privacy policy
            </a>
            <a
              href="https://github.com/LEONINE-DAO/Nozy-wallet/tree/master/nozy-mobile"
              target="_blank"
              rel="noopener noreferrer"
            >
              Source on GitHub
            </a>
            <Link to="/">Back to NozyWallet home</Link>
          </div>
        </div>
      </section>
    </div>
  );
};

export default Mobile;
