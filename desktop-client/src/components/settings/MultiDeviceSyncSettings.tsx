import { Button } from "../Button";
import { ArrowLeft, Document, Refresh, Download } from "@solar-icons/react";
import { Tooltip } from "../Tooltip";

interface MultiDeviceSyncSettingsProps {
  onBack: () => void;
}

export function MultiDeviceSyncSettings({ onBack }: MultiDeviceSyncSettingsProps) {
  return (
    <div className="max-w-2xl mx-auto animate-fade-in">
      <button
        onClick={onBack}
        className="flex items-center gap-2 text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 mb-6 transition-colors"
      >
        <ArrowLeft className="w-5 h-5" />
        <span className="font-medium">Back to Settings</span>
      </button>

      <div className="flex items-center gap-3 mb-6">
        <div className="w-12 h-12 rounded-full bg-primary/20 dark:bg-primary/30 flex items-center justify-center">
          <Refresh className="w-6 h-6 text-primary-600 dark:text-primary-400" />
        </div>
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            Sync across devices
          </h2>
          <p className="text-sm text-amber-600 dark:text-amber-400 font-medium">
            In design
          </p>
        </div>
      </div>

      <div className="p-4 rounded-xl bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm border border-white/50 dark:border-gray-700/50 space-y-4 mb-6">
        <p className="text-sm text-gray-700 dark:text-gray-300">
          Use the same wallet on multiple devices (e.g. desktop + phone) by syncing encrypted state. Your seed phrase is never uploaded—only scan progress, contacts, labels, and settings.
        </p>
        <ul className="text-sm text-gray-600 dark:text-gray-400 list-disc list-inside space-y-1">
          <li>Export an encrypted backup (file) from this device</li>
          <li>Import the backup on another device (same wallet, same password)</li>
          <li>The other device resumes from the last scanned height and gets your contacts and labels</li>
        </ul>
        <p className="text-sm text-gray-500 dark:text-gray-500">
          Backend support for export/import is coming soon. See the design doc: <code className="text-xs bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded">MULTI_DEVICE_SYNC_DESIGN.md</code>.
        </p>
        <Button
          variant="outline"
          size="sm"
          className="gap-2"
          onClick={() => window.open("https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/MULTI_DEVICE_SYNC_DESIGN.md", "_blank", "noopener,noreferrer")}
        >
          <Document size={16} />
          View design doc (GitHub)
        </Button>
      </div>

      <div className="space-y-3">
        <Tooltip content="Backend support for encrypted export is coming soon.">
          <div className="flex">
            <Button
              variant="outline"
              disabled
              className="w-full gap-2 opacity-60 cursor-not-allowed"
            >
              <Download size={18} />
              Export encrypted backup
              <span className="text-xs">(coming soon)</span>
            </Button>
          </div>
        </Tooltip>
        <Tooltip content="Backend support for encrypted import is coming soon.">
          <div className="flex">
            <Button
              variant="outline"
              disabled
              className="w-full gap-2 opacity-60 cursor-not-allowed"
            >
              <Refresh size={18} />
              Import backup from file
              <span className="text-xs">(coming soon)</span>
            </Button>
          </div>
        </Tooltip>
      </div>
    </div>
  );
}
