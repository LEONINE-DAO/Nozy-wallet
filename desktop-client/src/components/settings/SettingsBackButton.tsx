import { ArrowLeft } from "@solar-icons/react";
import { Button } from "../Button";

interface SettingsBackButtonProps {
  onClick: () => void;
  label?: string;
}

/** High-contrast back control used on settings sub-pages. */
export function SettingsBackButton({
  onClick,
  label = "Back to Settings",
}: SettingsBackButtonProps) {
  return (
    <Button
      type="button"
      variant="secondary"
      size="sm"
      onClick={onClick}
      className="mb-6 gap-2 font-semibold text-white border border-gray-500 bg-gray-800 hover:bg-gray-700 hover:border-gray-400 shadow-sm"
    >
      <ArrowLeft size={18} />
      {label}
    </Button>
  );
}
