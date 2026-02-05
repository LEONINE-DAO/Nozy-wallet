import { useState, useEffect } from "react";
import { Button } from "../Button";
import { Input } from "../Input";
import { ArrowLeft, CheckCircle, Danger, InfoCircle } from "@solar-icons/react";
import toast from "react-hot-toast";
import { formatErrorForDisplay } from "../../utils/errors";
import { walletApi } from "../../lib/api";

interface NetworkSettingsProps {
  onBack: () => void;
}

export function NetworkSettings({ onBack }: NetworkSettingsProps) {
  const [url, setUrl] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [testStatus, setTestStatus] = useState<"idle" | "success" | "error">(
    "idle"
  );

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      setIsLoading(true);
      const res = await walletApi.getConfig();
      if (res.data) {
        setUrl(res.data.zebra_url);
        toast.success("Network configuration loaded");
      }
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Failed to load network configuration"));
    } finally {
      setIsLoading(false);
    }
  };

  const handleSave = async () => {
    try {
      setIsSaving(true);
      await walletApi.setZebraUrl({ url });
      toast.success("Network configuration saved");
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Failed to save configuration"));
    } finally {
      setIsSaving(false);
    }
  };

  const handleTestConnection = async () => {
    const testToast = toast.loading("Testing connection to node...");
    setTestStatus("idle");
    try {
      const res = await walletApi.testZebraConnection();
      if (res?.data) {
        setTestStatus("success");
        toast.success("Node connection successful!", { id: testToast });
      } else {
        setTestStatus("error");
        toast.error("Node connection failed", { id: testToast });
      }
    } catch (e) {
      setTestStatus("error");
      toast.error(formatErrorForDisplay(e, "Failed to connect to node. Check your URL."), {
        id: testToast,
      });
    }
  };

  return (
    <div className="max-w-2xl mx-auto animate-fade-in">
      <button
        onClick={onBack}
        className="flex items-center gap-2 text-gray-500 hover:text-gray-900 mb-6 transition-colors"
      >
        <ArrowLeft className="w-5 h-5" />
        <span className="font-medium">Back to Settings</span>
      </button>

      <h2 className="text-3xl font-bold text-gray-900 mb-2">Network & Node</h2>
      <p className="text-gray-500 mb-8">
        Configure your connection to the backend API.
      </p>

      <div className="space-y-6">
        <div className="bg-white/60 backdrop-blur-sm rounded-2xl p-6 border border-white/50 shadow-sm space-y-4">
          <Input
            label="Backend API URL"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="http://127.0.0.1:8000"
            disabled={isLoading}
          />
          <div className="flex items-center gap-2 text-xs text-gray-500">
            <InfoCircle size={14} />
            <span>
              Point this to your self-hosted NozyWallet backend instance.
            </span>
          </div>

          <div className="pt-2 flex gap-3">
            <Button
              variant="secondary"
              onClick={handleTestConnection}
              className="flex-1"
            >
              Test Connection
            </Button>
            <Button
              onClick={handleSave}
              disabled={isSaving}
              className="flex-1"
            >
              {isSaving ? "Saving..." : "Save Configuration"}
            </Button>
          </div>

          {testStatus !== "idle" && (
            <div
              className={`p-3 rounded-xl flex items-center gap-2 text-sm font-medium ${
                testStatus === "success"
                  ? "bg-green-50 text-green-700 border border-green-100"
                  : "bg-red-50 text-red-700 border border-red-100"
              }`}
            >
              {testStatus === "success" ? (
                <>
                  <CheckCircle size={18} /> Connection Successful
                </>
              ) : (
                <>
                  <Danger size={18} /> Connection Failed
                </>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
