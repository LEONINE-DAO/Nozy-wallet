import { Button } from "../Button";
import { ArrowLeft, Shield, Document } from "@solar-icons/react";

interface MultiSigSettingsProps {
  onBack: () => void;
}

export function MultiSigSettings({ onBack }: MultiSigSettingsProps) {
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
        <div className="w-12 h-12 rounded-full bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center">
          <Shield className="w-6 h-6 text-amber-600 dark:text-amber-400" />
        </div>
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            Multi-signature
          </h2>
          <p className="text-sm text-amber-600 dark:text-amber-400 font-medium">
            In design
          </p>
        </div>
      </div>

      <div className="p-4 rounded-xl bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm border border-white/50 dark:border-gray-700/50 space-y-4">
        <p className="text-sm text-gray-700 dark:text-gray-300">
          Multi-sig (co-signing) support is planned. The design covers:
        </p>
        <ul className="text-sm text-gray-600 dark:text-gray-400 list-disc list-inside space-y-1">
          <li>Coordinator-based co-signing (initiator → cosigner(s) → broadcast)</li>
          <li>Signing request export/import (file or QR)</li>
          <li>Backend APIs for create request, sign request, and combine + broadcast</li>
        </ul>
        <p className="text-sm text-gray-500 dark:text-gray-500">
          Full implementation will follow the security audit and backend work. See the design doc in the repo: <code className="text-xs bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded">MULTISIG_DESIGN.md</code>.
        </p>
        <Button
          variant="outline"
          size="sm"
          className="gap-2"
          onClick={() => window.open("https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/MULTISIG_DESIGN.md", "_blank", "noopener,noreferrer")}
        >
          <Document size={16} />
          View design doc (GitHub)
        </Button>
      </div>
    </div>
  );
}
