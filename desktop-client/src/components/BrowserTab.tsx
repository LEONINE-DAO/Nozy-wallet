import { CloseCircle } from "@solar-icons/react";

interface BrowserTabProps {
  id: string;
  title: string;
  url: string;
  isActive: boolean;
  isLoading: boolean;
  onSelect: () => void;
  onClose: () => void;
}

export function BrowserTab({
  id: _id,
  title,
  url,
  isActive,
  isLoading,
  onSelect,
  onClose,
}: BrowserTabProps) {
  const displayTitle = title || new URL(url).hostname || "New Tab";

  return (
    <div
      className={`
        flex items-center gap-2 px-4 py-2 rounded-t-lg border-b-2 transition-all cursor-pointer
        ${isActive
          ? "bg-white dark:bg-gray-800 border-primary-600 dark:border-primary-400"
          : "bg-gray-100 dark:bg-gray-700 border-transparent hover:bg-gray-200 dark:hover:bg-gray-600"
        }
      `}
      onClick={onSelect}
    >
      {isLoading && (
        <div className="w-3 h-3 border-2 border-primary-600 dark:border-primary-400 border-t-transparent rounded-full animate-spin" />
      )}
      <span
        className={`
          text-sm truncate max-w-[200px]
          ${isActive
            ? "text-gray-900 dark:text-gray-100 font-medium"
            : "text-gray-600 dark:text-gray-400"
          }
        `}
        title={displayTitle}
      >
        {displayTitle}
      </span>
      <button
        onClick={(e) => {
          e.stopPropagation();
          onClose();
        }}
        className={`
          p-1 rounded hover:bg-gray-300 dark:hover:bg-gray-500 transition-colors
          ${isActive ? "text-gray-700 dark:text-gray-300" : "text-gray-500 dark:text-gray-400"}
        `}
      >
        <CloseCircle size={14} />
      </button>
    </div>
  );
}
