import { useCallback, useEffect, useState } from "react";
import toast from "react-hot-toast";
import QRCode from "react-qr-code";
import { Button } from "./Button";
import { Input } from "./Input";
import { walletApi } from "../lib/api";
import { resolveZnsName } from "../lib/zns";
import { formatErrorForDisplay } from "../utils/errors";

/** Business role + ZNS link for merchant / Sell identity. */
export function MerchantBusinessPanel({ compact = false }: { compact?: boolean }) {
  const [role, setRole] = useState("personal");
  const [displayName, setDisplayName] = useState("");
  const [linkedDisplay, setLinkedDisplay] = useState<string | null>(null);
  const [businessAddress, setBusinessAddress] = useState<string | null>(null);
  const [linkName, setLinkName] = useState("");
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const { data } = await walletApi.getWalletProfile();
      setRole(data.role);
      setDisplayName(data.business_display_name ?? "");
      setLinkedDisplay(data.linked_zns_display);
      if (data.business_address) setBusinessAddress(data.business_address);
    } catch {
      /* wallet may be locked */
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const setBusiness = async () => {
    setBusy(true);
    try {
      await walletApi.updateWalletProfile({
        role: "business",
        business_display_name: displayName || undefined,
      });
      const { data: addr } = await walletApi.generateAddress();
      setBusinessAddress(addr.address);
      setRole("business");
      toast.success("Business profile active (Orchard account 1)");
      await refresh();
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Could not switch to Business"));
    } finally {
      setBusy(false);
    }
  };

  const link = async () => {
    setBusy(true);
    try {
      const network = (await walletApi.getWalletProfile()).data.network === "testnet"
        ? "testnet"
        : "mainnet";
      const reg = await resolveZnsName(linkName.trim(), network);
      if (!reg?.address) {
        toast.error(`No Zcash name registered for “${linkName.trim()}”.`);
        return;
      }
      const { data } = await walletApi.linkZnsName({
        name: linkName.trim(),
        resolved_address: reg.address,
      });
      setLinkedDisplay(data.display);
      setBusinessAddress(data.business_address);
      setRole("business");
      setLinkName("");
      toast.success(`Linked ${data.display}`);
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Link failed"));
    } finally {
      setBusy(false);
    }
  };

  const unlink = async () => {
    setBusy(true);
    try {
      await walletApi.unlinkZnsName();
      setLinkedDisplay(null);
      toast.success("Name unlinked locally");
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Unlink failed"));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className={compact ? "space-y-3" : "space-y-4"}>
      {!compact && (
        <div>
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100">
            Business & Zcash name
          </h3>
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Business uses Orchard account 1. Claim a name on{" "}
            <a
              className="text-primary underline"
              href="https://www.zcashnames.com"
              target="_blank"
              rel="noreferrer"
            >
              zcashnames.com
            </a>{" "}
            pointing at your Business UA, then link it here.
          </p>
        </div>
      )}

      <p className="text-sm text-gray-700 dark:text-gray-300">
        Active:{" "}
        <span className="font-semibold">
          {role === "business" ? "Business (account 1)" : "Personal (account 0)"}
        </span>
        {linkedDisplay ? (
          <span className="ml-2 text-primary font-semibold">{linkedDisplay}</span>
        ) : null}
      </p>

      <Input
        label="Business display name"
        value={displayName}
        onChange={(e) => setDisplayName(e.target.value)}
        placeholder="Optional stall name"
      />

      <div className="flex flex-wrap gap-2">
        <Button size="sm" onClick={() => void setBusiness()} disabled={busy}>
          Use Business
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={busy}
          onClick={async () => {
            setBusy(true);
            try {
              await walletApi.updateWalletProfile({ role: "personal" });
              setRole("personal");
              toast.success("Personal profile active");
            } catch (e) {
              toast.error(formatErrorForDisplay(e, "Switch failed"));
            } finally {
              setBusy(false);
            }
          }}
        >
          Use Personal
        </Button>
      </div>

      {businessAddress && (
        <div className="flex flex-col items-center gap-2 py-2">
          <div className="bg-white p-2 rounded-lg">
            <QRCode value={businessAddress} size={compact ? 120 : 160} level="M" />
          </div>
          <p className="text-xs font-mono break-all text-gray-600 dark:text-gray-300 max-w-full">
            {businessAddress}
          </p>
        </div>
      )}

      {linkedDisplay ? (
        <Button size="sm" variant="ghost" onClick={() => void unlink()} disabled={busy}>
          Unlink {linkedDisplay}
        </Button>
      ) : (
        <div className="space-y-2">
          <Input
            label="Link Zcash name"
            value={linkName}
            onChange={(e) => setLinkName(e.target.value)}
            placeholder="mystore"
          />
          <Button size="sm" onClick={() => void link()} disabled={busy || !linkName.trim()}>
            Link name
          </Button>
        </div>
      )}
    </div>
  );
}
