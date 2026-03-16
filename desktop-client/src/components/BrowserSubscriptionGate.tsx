interface Props {
  onGoToSettings: () => void;
}

export function BrowserSubscriptionGate({ onGoToSettings }: Props) {
  return (
    <div className="flex flex-col items-center justify-center h-full gap-4 p-8">
      <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100">
        Browser Unavailable
      </h2>
      <p className="text-sm text-gray-600 dark:text-gray-400 text-center max-w-md">
        The built-in browser requires a Nym subscription for enhanced privacy.
      </p>
      <button
        onClick={onGoToSettings}
        className="px-4 py-2 rounded-lg bg-primary text-white hover:bg-primary/90 transition-colors"
      >
        Go to Settings
      </button>
    </div>
  );
}
