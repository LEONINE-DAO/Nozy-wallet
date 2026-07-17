import {
  ShieldCheck,
  Bolt,
  LockPassword,
  ShieldKeyholeMinimalistic,
  Download,
  QuestionCircle,
} from "@solar-icons/react";
import { PATHS, REPO_RELEASES } from "../lib/links";

type Status = "live" | "preview" | "soon";

import type { ComponentType, SVGProps } from "react";

type IconComponent = ComponentType<SVGProps<SVGSVGElement> & { size?: number }>;

type Surface = {
  icon: IconComponent;
  title: string;
  tagline: string;
  status: Status;
  statusLabel: string;
  bullets: string[];
  primary: { label: string; href: string; external?: boolean };
  secondary?: { label: string; href: string };
};

const statusStyles: Record<Status, string> = {
  live: "bg-emerald-500/10 text-emerald-800 border-emerald-500/25",
  preview: "bg-amber-500/10 text-amber-900 border-amber-500/25",
  soon: "bg-zinc-100 text-zinc-600 border-zinc-200",
};

const surfaces: Surface[] = [
  {
    icon: Bolt,
    title: "CLI Lite",
    tagline: "Production-ready shielded ZEC on your own Zebrad or Zakura + lightwalletd.",
    status: "live",
    statusLabel: "Mainnet ready",
    bullets: [
      "Orchard-first sends and sync (Nozy Lite)",
      "ZIP-317 fees, NU6.2 mainnet",
      "Ops helpers: health, status --json, optional Nym broadcast",
    ],
    primary: { label: "Get CLI Lite", href: "#download" },
    secondary: { label: "Latest release", href: REPO_RELEASES },
  },
  {
    icon: LockPassword,
    title: "Desktop",
    tagline: "Tauri GUI — Hot Lemon beta with Ironwood readiness tooling.",
    status: "preview",
    statusLabel: "Beta.2",
    bullets: [
      "Windows / macOS / Linux builds published",
      "Ironwood migrate / split / broadcast UI",
      "GA deferred until Ironwood is official",
    ],
    primary: { label: "Download desktop beta", href: "#download" },
    secondary: { label: "Desktop source", href: PATHS.desktop },
  },
  {
    icon: ShieldCheck,
    title: "Browser extension",
    tagline: "The community-shaped path — privacy-first, in your browser.",
    status: "preview",
    statusLabel: "Contributor preview",
    bullets: [
      "MV3 + WASM Orchard wallet",
      "Compact sync via local companion API",
      "Zips ship alongside desktop beta releases",
    ],
    primary: { label: "Extension docs", href: PATHS.extension, external: true },
    secondary: { label: "Companion setup", href: PATHS.extensionCompanion },
  },
  {
    icon: ShieldKeyholeMinimalistic,
    title: "Web app",
    tagline: "Full dashboard in the browser — extension + companion, keys stay yours.",
    status: "soon",
    statusLabel: "Coming soon",
    bullets: [
      "Send, receive, and sync without installing desktop",
      "Pairs with the extension and nozywallet-api",
      "Privacy chains added as modules, not an everything-wallet",
    ],
    primary: { label: "Web app plan", href: PATHS.webApp, external: true },
    secondary: { label: "Enhancement roadmap", href: PATHS.enhancementRoadmap },
  },
  {
    icon: Download,
    title: "Mobile",
    tagline: "Expo companion — wallet on phone, sync via your API.",
    status: "soon",
    statusLabel: "In development",
    bullets: [
      "Connects to nozywallet-api on PC or VPS + Zebrad/Zakura",
      "Business / Sell mode on the roadmap",
      "App Store and Play when ready",
    ],
    primary: { label: "Mobile repo", href: PATHS.mobile, external: true },
    secondary: { label: "VPS deploy guide", href: PATHS.operatorDeploy },
  },
  {
    icon: QuestionCircle,
    title: "Operator API",
    tagline: "Localhost companion for extension, mobile, and automation.",
    status: "preview",
    statusLabel: "In development",
    bullets: [
      "HTTP wrapper around the Rust wallet core",
      "LWD compact sync routes for the extension",
      "Run on your machine or a VPS you control",
    ],
    primary: { label: "api-server docs", href: PATHS.apiServer, external: true },
    secondary: { label: "VPS deploy", href: PATHS.operatorDeploy },
  },
];

function SurfaceCard({ surface }: { surface: Surface }) {
  const Icon = surface.icon;
  return (
    <article className="flex flex-col rounded-2xl border border-zinc-200 bg-white p-6 shadow-sm hover:border-yellow-300/60 hover:shadow-md transition-all">
      <div className="flex items-start justify-between gap-3 mb-4">
        <div className="w-11 h-11 rounded-xl bg-yellow-500/10 border border-yellow-500/20 flex items-center justify-center shrink-0">
          <Icon className="text-yellow-700" size={22} />
        </div>
        <span
          className={`text-xs font-semibold uppercase tracking-wide px-2.5 py-1 rounded-full border ${statusStyles[surface.status]}`}
        >
          {surface.statusLabel}
        </span>
      </div>

      <h3 className="text-lg font-bold text-zinc-900 mb-1">{surface.title}</h3>
      <p className="text-sm text-zinc-600 mb-4 leading-relaxed">{surface.tagline}</p>

      <ul className="space-y-2 mb-6 flex-1">
        {surface.bullets.map((line) => (
          <li key={line} className="flex gap-2 text-sm text-zinc-600">
            <span className="w-1.5 h-1.5 rounded-full bg-yellow-500 mt-2 shrink-0" />
            {line}
          </li>
        ))}
      </ul>

      <div className="flex flex-col gap-2 mt-auto">
        <a
          href={surface.primary.href}
          target={surface.primary.external ? "_blank" : undefined}
          rel={surface.primary.external ? "noopener noreferrer" : undefined}
          className="text-center rounded-xl bg-zinc-900 hover:bg-zinc-800 text-white font-semibold py-2.5 text-sm transition-colors"
        >
          {surface.primary.label}
        </a>
        {surface.secondary && (
          <a
            href={surface.secondary.href}
            target="_blank"
            rel="noopener noreferrer"
            className="text-center text-sm font-medium text-yellow-700 hover:text-yellow-800 hover:underline"
          >
            {surface.secondary.label} →
          </a>
        )}
      </div>
    </article>
  );
}

const ProductSurfaces = () => {
  return (
    <section id="products" className="py-24 bg-white border-t border-zinc-100 scroll-mt-24">
      <div className="max-w-7xl mx-auto px-6">
        <div className="text-center max-w-3xl mx-auto mb-6">
          <p className="text-xs font-semibold uppercase tracking-widest text-yellow-700 mb-3">
            ZEC today · privacy multichain later
          </p>
          <h2 className="text-3xl lg:text-5xl font-bold text-zinc-900 mb-4">
            One super wallet,{" "}
            <span className="text-gradient-primary">many surfaces</span>
          </h2>
          <p className="text-zinc-600 text-lg leading-relaxed">
            NozyWallet is community-shaped for privacy-native daily use: extension and web app for
            daily flows, CLI and desktop for operators, mobile when you are on the go.{" "}
            <strong className="text-zinc-800 font-semibold">CLI Lite is live for mainnet</strong>;
            Desktop beta.2 is out for early testers. Other privacy chains ship as modules when ready.
          </p>
        </div>

        <div className="flex flex-wrap justify-center gap-2 mb-12">
          <span className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-yellow-500/10 border border-yellow-500/25 text-sm font-medium text-yellow-900">
            <span className="w-2 h-2 rounded-full bg-yellow-500" />
            Zcash (Orchard) — supported
          </span>
          <span className="inline-flex items-center px-4 py-2 rounded-full bg-zinc-100 border border-zinc-200 text-sm text-zinc-500">
            Namada · Penumbra — planned
          </span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {surfaces.map((surface) => (
            <SurfaceCard key={surface.title} surface={surface} />
          ))}
        </div>
      </div>
    </section>
  );
};

export default ProductSurfaces;
