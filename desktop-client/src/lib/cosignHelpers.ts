import type { CosignPreparedSend } from "./types";

export type CosignSignedSend = CosignPreparedSend & {
  signed_at?: string;
};

export function zecFromZatoshis(zatoshis: number): number {
  return zatoshis / 100_000_000;
}

export function parseCosignJson(text: string): CosignSignedSend {
  const parsed = JSON.parse(text) as CosignSignedSend;
  if (!parsed.pczt_hex?.trim() || !parsed.recipient?.trim()) {
    throw new Error("Missing pczt_hex or recipient");
  }
  if (typeof parsed.amount_zatoshis !== "number" || parsed.amount_zatoshis <= 0) {
    throw new Error("Missing or invalid amount_zatoshis");
  }
  return parsed;
}

export function cosignToJson(payload: object): string {
  return JSON.stringify(payload, null, 2);
}

export function downloadJsonFile(filename: string, payload: object): void {
  const blob = new Blob([cosignToJson(payload)], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  URL.revokeObjectURL(url);
}
