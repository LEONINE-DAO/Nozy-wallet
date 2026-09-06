import { useEffect, useState } from "react";
import { Shield } from "@solar-icons/react";
import { walletApi } from "../lib/api";
import type { SendEgressKind, SendEgressSnapshot } from "../lib/types";

function badgeClass(kind: SendEgressKind): string {
  switch (kind) {
    case "local":
      return "border-emerald-200/80 bg-emerald-50/80 text-emerald-900 dark:border-emerald-800/50 dark:bg-emerald-950/30 dark:text-emerald-100";
    case "mixnet":
      return "border-violet-200/80 bg-violet-50/80 text-violet-900 dark:border-violet-800/50 dark:bg-violet-950/30 dark:text-violet-100";
    case "tor":
    case "i2p":
      return "border-sky-200/80 bg-sky-50/80 text-sky-900 dark:border-sky-800/50 dark:bg-sky-950/30 dark:text-sky-100";
    case "trusted":
      return "border-teal-200/80 bg-teal-50/80 text-teal-900 dark:border-teal-800/50 dark:bg-teal-950/30 dark:text-teal-100";
    case "direct_remote":
      return "border-amber-200/80 bg-amber-50/80 text-amber-900 dark:border-amber-800/50 dark:bg-amber-950/20 dark:text-amber-100";
    case "blocked":
      return "border-rose-200/80 bg-rose-50/80 text-rose-900 dark:border-rose-800/50 dark:bg-rose-950/20 dark:text-rose-100";
  }
}

export function SendEgressCard({ compact = false }: { compact?: boolean }) {
  const [egress, setEgress] = useState<SendEgressSnapshot | null>(null);

  useEffect(() => {
    let cancelled = false;
    void walletApi
      .getSendEgress()
      .then((res) => {
        if (!cancelled) setEgress(res.data);
      })
      .catch(() => {
        if (!cancelled) setEgress(null);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  if (!egress) return null;

  return (
    <div className={`rounded-xl border p-3 ${badgeClass(egress.kind)}`}>
      <div className="flex items-start gap-3">
        <Shield size={18} className="mt-0.5 shrink-0 opacity-80" />
        <div className="min-w-0">
          <p className="text-sm font-medium">
            Next send: {egress.label}
          </p>
          <p className="text-xs mt-1 opacity-90">{egress.summary}</p>
          {!compact && (
            <p className="text-xs mt-1 opacity-80">{egress.detail}</p>
          )}
          {egress.show_stopgap && (
            <p className="text-xs mt-2 opacity-90">
              Stopgap:{" "}
              <a
                href={egress.stopgap_url}
                target="_blank"
                rel="noreferrer"
                className="underline underline-offset-2"
              >
                {egress.stopgap_url}
              </a>
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
