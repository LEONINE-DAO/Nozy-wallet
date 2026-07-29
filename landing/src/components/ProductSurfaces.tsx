import {
  ShieldCheck,
  Bolt,
  LockPassword,
  ShieldKeyholeMinimalistic,
  Download,
  QuestionCircle,
} from "@solar-icons/react";
import { PATHS, REPO_RELEASES } from "../lib/links";
import type { ComponentType, SVGProps } from "react";

type Status = "live" | "preview" | "soon";
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
  live: "text-emerald-300 border-emerald-500/30 bg-emerald-500/10",
  preview: "text-[#c8ccd4] border-[#c8ccd4]/30 bg-[#c8ccd4]/10",
  soon: "text-[#a39a88] border-[rgba(245,240,230,0.15)] bg-white/5",
};

const surfaces: Surface[] = [
  {
    icon: Bolt,
    title: "CLI Lite",
    tagline: "Production shielded ZEC on your own Zebrad + lightwalletd.",
    status: "live",
    statusLabel: "Mainnet",
    bullets: [
      "Orchard + Ironwood sync and send",
      "Ironwood plan / migrate / broadcast",
      "ZIP-317 fees · ops helpers",
    ],
    primary: { label: "Get CLI Lite", href: "#download" },
    secondary: { label: "Latest release", href: REPO_RELEASES },
  },
  {
    icon: LockPassword,
    title: "Desktop",
    tagline: "Native wallet UI — sync, send, history, settings. No in-app browser.",
    status: "preview",
    statusLabel: "Beta",
    bullets: [
      "Windows Tauri app (wallet-only)",
      "Ironwood readiness + migrate tools",
      "Same disk profile as CLI / companion",
    ],
    primary: { label: "Download desktop", href: "#download" },
    secondary: { label: "Desktop source", href: PATHS.desktop },
  },
  {
    icon: ShieldCheck,
    title: "Browser extension",
    tagline: "Shielded ZEC in Chrome, Brave, and Edge — for sites and dApps.",
    status: "preview",
    statusLabel: "Public beta",
    bullets: [
      "MV3 + WASM local keys",
      "Optional companion API sync",
      "Connect / sign / send approvals",
    ],
    primary: { label: "Get extension", href: PATHS.extensionRelease, external: true },
    secondary: { label: "Install docs", href: PATHS.extension },
  },
  {
    icon: ShieldKeyholeMinimalistic,
    title: "Web companion",
    tagline: "Optional dashboard against your local API — keys stay yours.",
    status: "preview",
    statusLabel: "Preview",
    bullets: [
      "Unlock against nozywallet-api",
      "Balance + sync status",
      "Not a custodial web wallet",
    ],
    primary: { label: "Web app docs", href: PATHS.webApp, external: true },
    secondary: { label: "Roadmap", href: PATHS.enhancementRoadmap },
  },
  {
    icon: Download,
    title: "Mobile",
    tagline: "Phone companion — wallet on device, sync via your API.",
    status: "soon",
    statusLabel: "Coming soon",
    bullets: [
      "Expo companion + API",
      "Store listings in progress",
      "Same Orchard stack",
    ],
    primary: { label: "Mobile page", href: PATHS.mobilePage },
    secondary: { label: "Mobile repo", href: PATHS.mobile },
  },
  {
    icon: QuestionCircle,
    title: "Companion API",
    tagline: "Localhost bridge for extension, mobile, and automation.",
    status: "preview",
    statusLabel: "Available",
    bullets: [
      "HTTP wrapper around Rust core",
      "LWD compact sync routes",
      "Run on your machine",
    ],
    primary: { label: "API docs", href: PATHS.apiServer, external: true },
    secondary: { label: "Deploy guide", href: PATHS.operatorDeploy },
  },
];

function SurfaceCard({ surface }: { surface: Surface }) {
  const Icon = surface.icon;
  return (
    <article className="nw-panel flex flex-col p-6 hover:border-[#c8ccd4]/35 transition-colors">
      <div className="flex items-start justify-between gap-3 mb-5">
        <div className="w-11 h-11 border border-[rgba(245,240,230,0.12)] bg-[#c8ccd4]/10 flex items-center justify-center shrink-0">
          <Icon className="text-[#c8ccd4]" size={22} />
        </div>
        <span
          className={`text-[10px] font-semibold uppercase tracking-[0.14em] px-2.5 py-1 border ${statusStyles[surface.status]}`}
        >
          {surface.statusLabel}
        </span>
      </div>

      <h3 className="font-display text-xl font-bold text-[#f5f0e6] mb-2">{surface.title}</h3>
      <p className="text-sm text-[#a39a88] mb-5 leading-relaxed">{surface.tagline}</p>

      <ul className="space-y-2.5 mb-6 flex-1">
        {surface.bullets.map((line) => (
          <li key={line} className="flex gap-2.5 text-sm text-[#a39a88]">
            <span className="w-1 h-1 rounded-full bg-[#c8ccd4] mt-2 shrink-0" />
            {line}
          </li>
        ))}
      </ul>

      <div className="flex flex-col gap-2 mt-auto">
        <a
          href={surface.primary.href}
          target={surface.primary.external ? "_blank" : undefined}
          rel={surface.primary.external ? "noopener noreferrer" : undefined}
          className="nw-btn nw-btn-primary !py-2.5 text-sm"
        >
          {surface.primary.label}
        </a>
        {surface.secondary && (
          <a
            href={surface.secondary.href}
            target="_blank"
            rel="noopener noreferrer"
            className="text-center text-sm font-medium text-[#c8ccd4] hover:underline"
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
    <section id="products" className="py-24 border-t border-[rgba(245,240,230,0.12)] scroll-mt-24">
      <div className="max-w-7xl mx-auto px-6">
        <div className="max-w-2xl mb-14">
          <p className="nw-kicker mb-4">Surfaces</p>
          <h2 className="font-display text-3xl lg:text-5xl font-bold text-[#f5f0e6] mb-4 tracking-tight">
            One core. Right tool for the job.
          </h2>
          <p className="text-[#a39a88] text-lg leading-relaxed">
            Desktop and CLI are wallet functions. The extension is for the open web.
            Mobile connects when you are away from the desk.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
          {surfaces.map((surface) => (
            <SurfaceCard key={surface.title} surface={surface} />
          ))}
        </div>
      </div>
    </section>
  );
};

export default ProductSurfaces;
