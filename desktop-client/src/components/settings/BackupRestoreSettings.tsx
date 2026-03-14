import { useEffect, useState } from "react";
import toast from "react-hot-toast";
import { Button } from "../Button";
import { Input } from "../Input";
import { walletApi } from "../../lib/api";
import { formatErrorForDisplay } from "../../utils/errors";
import { ArrowLeft, Document, Download, Refresh } from "@solar-icons/react";

interface BackupRestoreSettingsProps {
  onBack: () => void;
}

export function BackupRestoreSettings({ onBack }: BackupRestoreSettingsProps) {
  const [backupDir, setBackupDir] = useState("");
  const [backupFile, setBackupFile] = useState("");
  const [backups, setBackups] = useState<string[]>([]);
  const [isExporting, setIsExporting] = useState(false);
  const [isRestoring, setIsRestoring] = useState(false);

  const refreshBackups = async () => {
    try {
      const response = await walletApi.listBackups();
      setBackups(response.data ?? []);
    } catch {
      // Intentionally silent; list is best-effort.
      setBackups([]);
    }
  };

  useEffect(() => {
    void refreshBackups();
  }, []);

  const handleExportBackup = async () => {
    if (!backupDir.trim()) {
      toast.error("Backup destination directory is required.");
      return;
    }
    const toastId = toast.loading("Exporting encrypted backup...");
    setIsExporting(true);
    try {
      const response = await walletApi.exportBackup({ backup_path: backupDir.trim() });
      toast.success(response.data.message || "Backup exported successfully.", { id: toastId });
      if (response.data.path) {
        setBackupFile(response.data.path);
      }
      await refreshBackups();
    } catch (error: unknown) {
      toast.error(formatErrorForDisplay(error, "Failed to export backup."), { id: toastId });
    } finally {
      setIsExporting(false);
    }
  };

  const handleRestoreBackup = async () => {
    if (!backupFile.trim()) {
      toast.error("Backup file path is required.");
      return;
    }

    const confirmed = window.confirm(
      "Restoring a backup will replace your current wallet file. Continue?"
    );
    if (!confirmed) {
      return;
    }

    const toastId = toast.loading("Restoring wallet from backup...");
    setIsRestoring(true);
    try {
      const response = await walletApi.restoreFromBackup({ backup_path: backupFile.trim() });
      toast.success(response.data.message || "Wallet restored from backup.", { id: toastId });
      await refreshBackups();
    } catch (error: unknown) {
      toast.error(formatErrorForDisplay(error, "Failed to restore from backup."), { id: toastId });
    } finally {
      setIsRestoring(false);
    }
  };

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
          <p className="text-sm text-primary-600 dark:text-primary-400 font-medium">
            Encrypted backup tools enabled
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
          Backups are created as encrypted wallet files in your chosen directory. Keep at least one offline copy. See <code className="text-xs bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded">SECURE_BACKUPS_DESIGN.md</code>.
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

      <div className="space-y-4">
        <Input
          label="Backup destination directory"
          placeholder="e.g. C:\\Users\\you\\Documents\\NozyBackups"
          value={backupDir}
          onChange={(e) => setBackupDir(e.target.value)}
        />
        <Button
          variant="outline"
          className="w-full gap-2"
          onClick={handleExportBackup}
          disabled={isExporting}
        >
          <Download size={18} />
          {isExporting ? "Exporting backup..." : "Export encrypted backup"}
        </Button>

        <Input
          label="Backup file path to restore"
          placeholder="e.g. C:\\Users\\you\\Documents\\NozyBackups\\wallet_backup_123456.dat"
          value={backupFile}
          onChange={(e) => setBackupFile(e.target.value)}
        />
        <Button
          variant="outline"
          className="w-full gap-2"
          onClick={handleRestoreBackup}
          disabled={isRestoring}
        >
          <Refresh size={18} />
          {isRestoring ? "Restoring backup..." : "Restore from backup file"}
        </Button>

        <div className="rounded-lg border border-gray-200/60 dark:border-gray-700/60 bg-white/50 dark:bg-gray-800/40 p-3">
          <div className="flex items-center justify-between mb-2">
            <p className="text-sm font-medium text-gray-800 dark:text-gray-200">
              Known backups
            </p>
            <button
              type="button"
              className="text-xs text-primary hover:underline"
              onClick={() => void refreshBackups()}
            >
              Refresh
            </button>
          </div>
          {backups.length === 0 ? (
            <p className="text-xs text-gray-500 dark:text-gray-500">
              No backups discovered in wallet storage yet.
            </p>
          ) : (
            <div className="space-y-2 max-h-40 overflow-auto">
              {backups.map((path) => (
                <button
                  key={path}
                  type="button"
                  className="w-full text-left text-xs rounded border border-gray-200/50 dark:border-gray-700/50 px-2 py-1 hover:border-primary/40 hover:bg-primary/5 transition-colors"
                  onClick={() => setBackupFile(path)}
                >
                  {path}
                </button>
              ))}
            </div>
          )}
        </div>

        <p className="text-xs text-gray-500 dark:text-gray-500">
          Cloud backup (upload/download) is planned for a later phase.
        </p>
      </div>
    </div>
  );
}
