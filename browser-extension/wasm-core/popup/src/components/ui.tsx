/**
 * Shared popup primitives. Visual language: dark shell aligned with desktop silver/platinum,
 * inset fields (never plain white). Tokens live in styles.css.
 */
import { useState } from "react";
import type {
  ButtonHTMLAttributes,
  CSSProperties,
  InputHTMLAttributes,
  ReactNode,
  SelectHTMLAttributes,
  TextareaHTMLAttributes
} from "react";

type Tone = "neutral" | "accent" | "success" | "danger" | "warn";

function cx(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(" ");
}

/* ── Icons ──────────────────────────────────────────────── */

export type IconName =
  | "home"
  | "send"
  | "receive"
  | "grid"
  | "settings"
  | "copy"
  | "check"
  | "chevron"
  | "refresh"
  | "lock"
  | "shield"
  | "vote"
  | "expand"
  | "globe";

const ICON_PATHS: Record<IconName, ReactNode> = {
  home: <path d="M3 10.5 12 3l9 7.5V20a1 1 0 0 1-1 1h-5v-6H10v6H4a1 1 0 0 1-1-1z" />,
  send: <path d="M12 20V5m0 0-6 6m6-6 6 6" />,
  receive: <path d="M12 4v15m0 0 6-6m-6 6-6-6" />,
  grid: (
    <>
      <rect x="3" y="3" width="7" height="7" rx="1.5" />
      <rect x="14" y="3" width="7" height="7" rx="1.5" />
      <rect x="3" y="14" width="7" height="7" rx="1.5" />
      <rect x="14" y="14" width="7" height="7" rx="1.5" />
    </>
  ),
  settings: (
    <>
      <circle cx="12" cy="12" r="3" />
      <path d="M12 2v3m0 14v3M2 12h3m14 0h3M4.9 4.9l2.1 2.1m10 10 2.1 2.1m0-14.2-2.1 2.1m-10 10-2.1 2.1" />
    </>
  ),
  copy: (
    <>
      <rect x="9" y="9" width="11" height="11" rx="2" />
      <path d="M15 5.5A1.5 1.5 0 0 0 13.5 4H5.5A1.5 1.5 0 0 0 4 5.5v8A1.5 1.5 0 0 0 5.5 15" />
    </>
  ),
  check: <path d="m4 12.5 5 5 11-11" />,
  chevron: <path d="m9 6 6 6-6 6" />,
  refresh: <path d="M20 12a8 8 0 1 1-2.34-5.66M20 4v4h-4" />,
  lock: (
    <>
      <rect x="4" y="10" width="16" height="11" rx="2" />
      <path d="M8 10V7a4 4 0 0 1 8 0v3" />
    </>
  ),
  shield: <path d="M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6z" />,
  vote: (
    <>
      <path d="M4 20h16" />
      <path d="m6 15 3-9 9 3-3 9z" />
    </>
  ),
  expand: (
    <>
      <path d="M14 4h6v6" />
      <path d="M10 20H4v-6" />
      <path d="M20 4 13 11" />
      <path d="m4 20 7-7" />
    </>
  ),
  globe: (
    <>
      <circle cx="12" cy="12" r="9" />
      <path d="M3 12h18" />
      <path d="M12 3a15 15 0 0 1 0 18" />
      <path d="M12 3a15 15 0 0 0 0 18" />
    </>
  )
};

export function Icon({ name, size = 16 }: { name: IconName; size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.8}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {ICON_PATHS[name]}
    </svg>
  );
}

/* ── Layout & typography ────────────────────────────────── */

export function Screen({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cx("space-y-3 px-4 pt-4 pb-2", className)}>{children}</div>;
}

export function PageHeader({
  title,
  description,
  trailing
}: {
  title: string;
  description?: ReactNode;
  trailing?: ReactNode;
}) {
  return (
    <div className="flex items-start justify-between gap-3">
      <div className="min-w-0">
        <h1 className="nw-title">{title}</h1>
        {description && <p className="nw-hint mt-1">{description}</p>}
      </div>
      {trailing && <div className="shrink-0">{trailing}</div>}
    </div>
  );
}

export function SectionTitle({ children, trailing }: { children: ReactNode; trailing?: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-2">
      <h2 className="nw-section-title">{children}</h2>
      {trailing}
    </div>
  );
}

export function Eyebrow({ children }: { children: ReactNode }) {
  return <div className="nw-eyebrow">{children}</div>;
}

export function Hint({
  children,
  className,
  style
}: {
  children: ReactNode;
  className?: string;
  style?: CSSProperties;
}) {
  return (
    <p className={cx("nw-hint", className)} style={style}>
      {children}
    </p>
  );
}

export function Card({
  children,
  tone,
  className,
  flush
}: {
  children: ReactNode;
  tone?: "accent";
  className?: string;
  flush?: boolean;
}) {
  return (
    <div
      className={cx(
        "nw-card",
        tone === "accent" && "nw-card--accent",
        flush && "nw-card--flush",
        className
      )}
    >
      {children}
    </div>
  );
}

/* ── Controls ───────────────────────────────────────────── */

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: "primary" | "secondary" | "ghost" | "success" | "danger";
  size?: "md" | "sm";
  fullWidth?: boolean;
  icon?: IconName;
};

export function Button({
  variant = "secondary",
  size = "md",
  fullWidth,
  icon,
  className,
  children,
  type = "button",
  ...rest
}: ButtonProps) {
  return (
    <button
      type={type}
      className={cx(
        "nw-btn",
        `nw-btn--${variant}`,
        size === "sm" && "nw-btn--sm",
        fullWidth && "w-full",
        className
      )}
      {...rest}
    >
      {icon && <Icon name={icon} size={size === "sm" ? 13 : 15} />}
      {children}
    </button>
  );
}

type FieldProps = {
  label?: ReactNode;
  hint?: ReactNode;
  error?: string | null;
  mono?: boolean;
};

export function Input({
  label,
  hint,
  error,
  mono,
  className,
  ...rest
}: InputHTMLAttributes<HTMLInputElement> & FieldProps) {
  return (
    <div>
      {label && <label className="nw-label">{label}</label>}
      {hint && <Hint className="mb-1.5">{hint}</Hint>}
      <input
        className={cx("nw-input", mono && "nw-mono", error && "nw-input--invalid", className)}
        {...rest}
      />
      {error && <p className="nw-hint mt-1" style={{ color: "var(--nw-danger)" }}>{error}</p>}
    </div>
  );
}

export function Textarea({
  label,
  hint,
  error,
  mono,
  className,
  ...rest
}: TextareaHTMLAttributes<HTMLTextAreaElement> & FieldProps) {
  return (
    <div>
      {label && <label className="nw-label">{label}</label>}
      {hint && <Hint className="mb-1.5">{hint}</Hint>}
      <textarea
        className={cx("nw-input resize-none", mono && "nw-mono", error && "nw-input--invalid", className)}
        {...rest}
      />
      {error && <p className="nw-hint mt-1" style={{ color: "var(--nw-danger)" }}>{error}</p>}
    </div>
  );
}

export function Select({
  label,
  hint,
  className,
  children,
  ...rest
}: SelectHTMLAttributes<HTMLSelectElement> & FieldProps) {
  return (
    <div>
      {label && <label className="nw-label">{label}</label>}
      {hint && <Hint className="mb-1.5">{hint}</Hint>}
      <select className={cx("nw-input", className)} {...rest}>
        {children}
      </select>
    </div>
  );
}

export function SegmentedControl<T extends string>({
  options,
  value,
  onChange
}: {
  options: Array<{ value: T; label: string }>;
  value: T;
  onChange: (next: T) => void;
}) {
  return (
    <div className="nw-segment">
      {options.map((option) => {
        const active = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            onClick={() => onChange(option.value)}
            className={cx("nw-segment__btn", active && "nw-segment__btn--active")}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}

/* ── Feedback ───────────────────────────────────────────── */

export function Pill({ children, tone = "neutral" }: { children: ReactNode; tone?: Tone }) {
  return <span className={`nw-pill nw-pill--${tone}`}>{children}</span>;
}

export function Callout({
  children,
  tone = "info"
}: {
  children: ReactNode;
  tone?: "info" | "accent" | "success" | "danger" | "warn";
}) {
  return <div className={`nw-callout nw-callout--${tone}`}>{children}</div>;
}

export function ProgressBar({
  percent,
  variant = "default"
}: {
  percent: number;
  variant?: "default" | "cyberpunk";
}) {
  const clamped = Math.min(100, Math.max(0, percent));
  if (variant === "cyberpunk") {
    return (
      <div
        className="nw-sync-bar h-2 overflow-hidden rounded-full"
        role="progressbar"
        aria-valuenow={clamped}
        aria-valuemin={0}
        aria-valuemax={100}
      >
        <div
          className="nw-sync-bar-fill h-full rounded-full"
          style={{ width: `${clamped}%` }}
        />
      </div>
    );
  }
  return (
    <div className="nw-progress">
      <div className="nw-progress__bar" style={{ width: `${clamped}%` }} />
    </div>
  );
}

export function SyncGlitchText({
  children,
  className
}: {
  children: string;
  className?: string;
}) {
  return (
    <span className={cx("nw-sync-glitch", className)} data-text={children}>
      {children}
    </span>
  );
}

export function CyberpunkSyncPanel({
  headline,
  detail,
  percent,
  tone = "syncing"
}: {
  headline: string;
  detail?: string;
  percent?: number | null;
  tone?: "ok" | "warn" | "offline" | "syncing";
}) {
  const panelClass =
    tone === "warn"
      ? "nw-sync-panel nw-sync-panel--warn"
      : tone === "offline"
        ? "nw-sync-panel nw-sync-panel--offline"
        : "nw-sync-panel";
  const pct =
    percent != null && Number.isFinite(percent)
      ? Math.min(100, Math.max(0, percent))
      : null;

  return (
    <div className={cx(panelClass, "rounded-xl border px-3 py-2.5")} aria-live="polite">
      <div className="flex min-w-0 items-center gap-3">
        <div className="min-w-0 flex-1">
          <p className="text-[0.65rem] font-bold uppercase tracking-[0.18em] text-emerald-300/70">
            Chain sync
          </p>
          <SyncGlitchText className="mt-0.5 block truncate text-sm font-bold text-emerald-100">
            {headline}
          </SyncGlitchText>
          {detail ? (
            <SyncGlitchText className="mt-0.5 block truncate text-xs font-medium text-emerald-200/80">
              {detail}
            </SyncGlitchText>
          ) : null}
        </div>
        {pct != null ? (
          <SyncGlitchText className="shrink-0 text-lg font-extrabold tabular-nums text-emerald-200">
            {`${Math.floor(pct)}%`}
          </SyncGlitchText>
        ) : null}
      </div>
      {pct != null ? (
        <div className="mt-2.5">
          <ProgressBar percent={pct} variant="cyberpunk" />
        </div>
      ) : null}
    </div>
  );
}

export function StatRow({
  label,
  value,
  tone
}: {
  label: ReactNode;
  value: ReactNode;
  tone?: "accent" | "success" | "muted";
}) {
  const color =
    tone === "accent"
      ? "var(--nw-platinum)"
      : tone === "success"
        ? "var(--nw-success)"
        : tone === "muted"
          ? "var(--nw-faint)"
          : "var(--nw-ink)";
  return (
    <div className="flex items-center justify-between gap-3 text-xs">
      <span style={{ color: "var(--nw-faint)" }}>{label}</span>
      <span className="nw-mono truncate" style={{ color }}>
        {value}
      </span>
    </div>
  );
}

export function ListRow({
  title,
  description,
  onClick,
  trailing
}: {
  title: ReactNode;
  description?: ReactNode;
  onClick?: () => void;
  trailing?: ReactNode;
}) {
  return (
    <button type="button" className="nw-row" onClick={onClick}>
      <span className="min-w-0">
        <span className="block text-[13px] font-semibold">{title}</span>
        {description && (
          <span className="nw-hint mt-0.5 block">{description}</span>
        )}
      </span>
      <span className="shrink-0" style={{ color: "var(--nw-faint)" }}>
        {trailing ?? <Icon name="chevron" size={14} />}
      </span>
    </button>
  );
}

export function LogBlock({ children }: { children: ReactNode }) {
  return <pre className="nw-log">{children}</pre>;
}

export function EmptyState({ children }: { children: ReactNode }) {
  return (
    <p className="nw-hint py-3 text-center">{children}</p>
  );
}

/* ── Clipboard ──────────────────────────────────────────── */

export function useCopy(resetMs = 1500): [boolean, (text: string) => void] {
  const [copied, setCopied] = useState(false);
  const copy = (text: string) => {
    if (!text) return;
    void navigator.clipboard
      .writeText(text)
      .then(() => {
        setCopied(true);
        setTimeout(() => setCopied(false), resetMs);
      })
      .catch(() => undefined);
  };
  return [copied, copy];
}

export function CopyButton({
  value,
  label = "Copy",
  fullWidth
}: {
  value: string;
  label?: string;
  fullWidth?: boolean;
}) {
  const [copied, copy] = useCopy();
  return (
    <Button
      size="sm"
      variant="secondary"
      fullWidth={fullWidth}
      icon={copied ? "check" : "copy"}
      disabled={!value}
      onClick={() => copy(value)}
    >
      {copied ? "Copied" : label}
    </Button>
  );
}
