import type { ReactNode } from "react";

type TileAccent = "default" | "gold" | "emerald" | "amber";
type SurfaceTone = "dark" | "platinum";

/** Shared Vote / Crosslink-style metric tile. */
export function MetricTile({
  label,
  value,
  hint,
  accent = "default",
  tone = "dark",
}: {
  label: string;
  value: string;
  hint?: string;
  accent?: TileAccent;
  tone?: SurfaceTone;
}) {
  if (tone === "platinum") {
    const accentCls =
      accent === "gold"
        ? "border-lime-400/40 bg-lime-500/10"
        : accent === "emerald"
          ? "border-emerald-400/45 bg-emerald-500/15"
          : accent === "amber"
            ? "border-amber-400/40 bg-amber-500/10"
            : "border-emerald-500/25 bg-black/35";
    return (
      <div className={`rounded-2xl border p-4 shadow-lg shadow-emerald-950/30 backdrop-blur-md ${accentCls}`}>
        <p className="text-[0.65rem] font-bold uppercase tracking-[0.2em] text-emerald-300/90">{label}</p>
        <p className="mt-2 text-xl font-extrabold tracking-tight text-primary-100">{value}</p>
        {hint ? <p className="mt-1.5 text-xs leading-relaxed text-emerald-200/60">{hint}</p> : null}
      </div>
    );
  }

  const accentCls =
    accent === "gold"
      ? "border-primary/35 bg-gradient-to-br from-primary/20 to-amber-950/20"
      : accent === "emerald"
        ? "border-emerald-400/30 bg-gradient-to-br from-emerald-500/15 to-emerald-950/20"
        : accent === "amber"
          ? "border-amber-400/30 bg-gradient-to-br from-amber-500/15 to-amber-950/20"
          : "border-white/10 bg-black/25";
  const labelCls =
    accent === "gold"
      ? "text-primary/90"
      : accent === "emerald"
        ? "text-emerald-200/90"
        : accent === "amber"
          ? "text-amber-200/90"
          : "text-gray-400";
  return (
    <div className={`rounded-2xl border p-4 backdrop-blur-sm ${accentCls}`}>
      <p className={`text-[0.65rem] font-bold uppercase tracking-[0.2em] ${labelCls}`}>{label}</p>
      <p className="mt-2 text-xl font-extrabold tracking-tight text-primary-100">{value}</p>
      {hint ? <p className="mt-1.5 text-xs leading-relaxed text-gray-400">{hint}</p> : null}
    </div>
  );
}

export function Panel({
  title,
  subtitle,
  children,
  className = "",
  tone = "dark",
}: {
  title: string;
  subtitle?: string;
  children: ReactNode;
  className?: string;
  tone?: SurfaceTone;
}) {
  if (tone === "platinum") {
    return (
      <div
        className={`rounded-2xl border border-emerald-500/25 bg-black/40 p-4 shadow-lg shadow-emerald-950/25 backdrop-blur-md ${className}`}
      >
        <p className="text-[0.65rem] font-bold uppercase tracking-[0.18em] text-emerald-300/85">{title}</p>
        {subtitle ? <p className="mt-1 text-xs text-emerald-200/55">{subtitle}</p> : null}
        <div className="mt-3">{children}</div>
      </div>
    );
  }

  return (
    <div
      className={`rounded-2xl border border-white/10 bg-black/20 p-4 backdrop-blur-sm ${className}`}
    >
      <p className="text-[0.65rem] font-bold uppercase tracking-[0.18em] text-gray-400">{title}</p>
      {subtitle ? <p className="mt-1 text-xs text-gray-500">{subtitle}</p> : null}
      <div className="mt-3">{children}</div>
    </div>
  );
}

/** Outer gradient shell used by Vote / Home / History feature cards. */
export function FeatureShell({
  children,
  className = "",
  tone = "dark",
}: {
  children: ReactNode;
  className?: string;
  tone?: SurfaceTone;
}) {
  if (tone === "platinum") {
    return (
      <section
        className={`relative overflow-hidden rounded-2xl border border-emerald-400/35 bg-gradient-to-br from-[#031910]/95 via-[#06281c]/92 to-[#0a3a28]/90 p-6 text-primary-100 shadow-2xl shadow-emerald-950/40 sm:p-8 ${className}`}
      >
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_top_left,_rgba(0,255,140,0.18),_transparent_50%),radial-gradient(ellipse_at_bottom_right,_rgba(0,200,100,0.12),_transparent_45%)]"
        />
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 opacity-[0.07] [background-image:linear-gradient(rgba(0,255,140,0.35)_1px,transparent_1px),linear-gradient(90deg,rgba(0,255,140,0.35)_1px,transparent_1px)] [background-size:28px_28px]"
        />
        <div className="relative z-10">{children}</div>
      </section>
    );
  }

  return (
    <section
      className={`relative overflow-hidden rounded-2xl border border-primary/25 bg-gradient-to-br from-amber-950/35 via-[#0c0b09] to-emerald-950/25 p-6 shadow-2xl shadow-amber-950/20 ${className}`}
    >
      <div
        aria-hidden
        className="pointer-events-none absolute -right-24 -top-24 h-64 w-64 rounded-full bg-primary/15 blur-3xl"
      />
      <div
        aria-hidden
        className="pointer-events-none absolute -bottom-20 -left-16 h-52 w-52 rounded-full bg-emerald-500/10 blur-3xl"
      />
      <div className="relative">{children}</div>
    </section>
  );
}
