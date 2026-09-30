import type { IronwoodDesktopStatusResponse } from "./types";

export type IronwoodNextStep = {
  /** Short label for badges / buttons */
  title: string;
  /** One-line guidance for Home banner and Ironwood card */
  detail: string;
  /** Which action is primary right now */
  primaryAction: "plan" | "split" | "migrate" | "broadcast" | "wait" | "done" | "fix" | null;
};

/**
 * Map backend readiness_state to a single user-facing next step.
 * Keeps Home + Ironwood tab aligned so residual Orchard holders know what to do.
 */
export function ironwoodNextStep(
  status: IronwoodDesktopStatusResponse | null
): IronwoodNextStep | null {
  if (!status) return null;

  if (status.readiness_state === "no-orchard-notes" || status.orchard_wallet_zat <= 0) {
    if (status.ironwood_active) {
      return {
        title: "Migration complete",
        detail:
          "No Orchard notes left in this wallet. New shielded sends use Ironwood.",
        primaryAction: "done",
      };
    }
    return {
      title: "No Orchard to migrate",
      detail: "Wallet has no Orchard notes. Sync or restore if you expected a balance.",
      primaryAction: null,
    };
  }

  switch (status.readiness_state) {
    case "planning-only":
      return {
        title: "Plan migration",
        detail:
          "Save a ZIP 318 schedule now. Start migration unlocks after Ironwood is active on-chain.",
        primaryAction: "plan",
      };
    case "needs-plan":
      return {
        title: "Plan migration first",
        detail:
          "Only Plan is active until a schedule is saved. Click Plan migration, then Start migration → Broadcast.",
        primaryAction: "plan",
      };
    case "split-required":
      return {
        title: "Split notes",
        detail:
          "Canonical denominations need a note split before the next turnstile transfer.",
        primaryAction: "split",
      };
    case "ready-to-prebuild":
      return {
        title: "Start migration",
        detail:
          "Next ZIP 318 transfer is eligible. Prebuild the turnstile tx, then Broadcast in-window.",
        primaryAction: "migrate",
      };
    case "waiting-for-window":
      return {
        title: "Waiting for window",
        detail: status.next_anchor_bucket_height
          ? `Plan is saved. Next bucket at height ${status.next_anchor_bucket_height.toLocaleString()}. Keep synced; Start migration unlocks in-window.`
          : "Plan is saved. Waiting for the next ZIP 318 bucket — keep the wallet synced.",
        primaryAction: "wait",
      };
    case "presigned-waiting-for-broadcast":
    case "ready-to-broadcast":
      return {
        title: "Broadcast",
        detail:
          "Presigned turnstile is ready. Prefer local Zebrad (or attested private egress), then Broadcast.",
        primaryAction: "broadcast",
      };
    case "blocked":
      return {
        title: "Blocked",
        detail:
          status.blockers[0] ??
          "Migration is blocked. Check Safety gates on the Ironwood tab.",
        primaryAction: "fix",
      };
    default:
      if (status.orchard_funds_at_risk || status.migration_recommended) {
        return {
          title: "Plan migration first",
          detail:
            "Orchard is sealed. Click Plan migration to save the ZIP 318 schedule — other buttons unlock after that.",
          primaryAction: "plan",
        };
      }
      return null;
  }
}

/** Why a migration button is disabled (for tooltips / helper text). */
export function ironwoodActionDisableReason(
  action: "plan" | "split" | "migrate" | "broadcast",
  status: IronwoodDesktopStatusResponse | null,
  networkPrivacyAllowed: boolean | undefined
): string | null {
  if (!status) return "Waiting for Ironwood status…";
  if (!status.migration_enabled) {
    return "Migration tools unlock once Ironwood is active (or Ironwood RPC on testnet).";
  }
  switch (action) {
    case "plan":
      return null;
    case "split":
      if (!status.ironwood_active) return "Split needs Ironwood active on-chain.";
      if (!status.zip318_note_split_required) {
        return "Split not required for the current plan.";
      }
      return null;
    case "migrate":
      if (status.readiness_state === "needs-plan" || status.readiness_state === "planning-only") {
        return "Click Plan migration first to save the ZIP 318 schedule.";
      }
      if (status.zip318_note_split_required) {
        return "Split notes first, then Start migration.";
      }
      if (status.readiness_state === "waiting-for-window") {
        return status.next_anchor_bucket_height
          ? `Waiting for ZIP 318 bucket at height ${status.next_anchor_bucket_height.toLocaleString()}.`
          : "Waiting for the next ZIP 318 bucket window.";
      }
      if (!status.ready_to_prebuild) {
        return status.blockers[0] ?? "Not ready to prebuild yet — check Safety gates.";
      }
      return null;
    case "broadcast":
      if (!status.ready_to_broadcast) {
        return "Broadcast unlocks after Start migration prebuilds a turnstile tx.";
      }
      if (!networkPrivacyAllowed) {
        return "Broadcast blocked until local Zebrad or private-network attestation (Settings → Network privacy).";
      }
      return null;
  }
}
