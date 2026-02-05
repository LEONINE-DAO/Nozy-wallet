import { Button } from "../Button";
import { ArrowLeft, Document, Download, Refresh } from "@solar-icons/react";
import { Tooltip } from "../Tooltip";

interface BackupRestoreSettingsProps {
  onBack: () => void;
}

export function BackupRestoreSettings({ onBack }: BackupRestoreSettingsProps) {
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
          <Download className="w-6 h-6 text-primary-600 dark:text-primary-400" />
        </div>
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            Backup & restore
          </h2>
          <p className="text-sm text-amber-600 dark:text-amber-400 font-medium">
            Backend wiring in progress
          </p>
        </div>
      </div>

      <div className="p-4 rounded-xl bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm border border-white/50 dark:border-gray-700/50 space-y-4 mb-6">
        <p className="text-sm text-gray-700 dark:text-gray-300">
          Beyond your recovery phrase, you can create an <strong>encrypted backup file</strong> of your wallet. Store it safely (e.g. USB drive, external disk). Restore with the same password if you reinstall or switch devices.
        </p>
        <ul className="text-sm text-gray-600 dark:text-gray-400 list-disc list-inside space-y-1">
          <li>Export: saves an encrypted copy of your wallet (same format as the app)</li>
          <li>Restore: replaces the current wallet with the backup (current wallet is backed up first)</li>
          <li>Use the same password you use to unlock the wallet</li>
        </ul>
        <p className="text-sm text-gray-500 dark:text-gray-500">
          The Rust backend already has <code className="text-xs bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded">create_backup</code> and <code className="text-xs bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded">restore_from_backup</code> in storage. They need to be exposed via Tauri and wired here. See <code className="text-xs bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded">SECURE_BACKUPS_DESIGN.md</code>.
        </p>
        <Button
          variant="outline"
          size="sm"
          className="gap-2"
          onClick={() => window.open("https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/SECURE_BACKUPS_DESIGN.md", "_blank", "noopener,noreferrer")}
        >
          <Document size={16} />
          View design doc (GitHub)
        </Button>
      </div>

      <div className="space-y-3">
        <Tooltip content="Tauri backend must expose export_backup (with user-chosen path). Coming soon.">
          <div className="flex">
            <Button
              variant="outline"
              disabled
              className="w-full gap-2 opacity-60 cursor-not-allowed"
            >
              <Download size={18} />
              Export encrypted backup (file)
              <span className="text-xs">(coming soon)</span>
            </Button>
          </div>
        </Tooltip>
        <Tooltip content="Tauri backend must expose restore_from_backup (with user-chosen file). Coming soon.">
          <div className="flex">
            <Button
              variant="outline"
              disabled
              className="w-full gap-2 opacity-60 cursor-not-allowed"
            >
              <Refresh size={18} />
              Restore from backup file
              <span className="text-xs">(coming soon)</span>
            </Button>
          </div>
        </Tooltip>
        <p className="text-xs text-gray-500 dark:text-gray-500">
          Cloud backup (upload/download) is planned for a later phase.
        </p>
      </div>
    </div>
  );
}
