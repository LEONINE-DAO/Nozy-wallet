import React from "react";
import { cn } from "../lib/cn";

export { cn };

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "outline" | "ghost" | "danger";
  size?: "sm" | "md" | "lg";
  /** Shows a spinner and keeps the button fully visible so long work does not look frozen. */
  loading?: boolean;
}

function Spinner({ className }: { className?: string }) {
  return (
    <svg
      className={cn("h-4 w-4 animate-spin shrink-0", className)}
      viewBox="0 0 24 24"
      fill="none"
      aria-hidden
    >
      <circle
        className="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        strokeWidth="3"
      />
      <path
        className="opacity-90"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V2C5.373 2 2 5.373 2 12h2z"
      />
    </svg>
  );
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = "primary", size = "md", loading = false, disabled, children, ...props }, ref) => {
    const isDisabled = Boolean(disabled || loading);
    return (
      <button
        ref={ref}
        aria-busy={loading || undefined}
        disabled={isDisabled}
        className={cn(
          "inline-flex items-center justify-center gap-2 rounded-xl font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50 focus-visible:ring-offset-2 focus-visible:ring-offset-transparent disabled:pointer-events-none active:scale-[0.98]",
          loading ? "opacity-100 cursor-wait" : "disabled:opacity-50",
          {
            "bg-primary text-gray-900 shadow-md shadow-primary/20 hover:bg-primary-300":
              variant === "primary",
            "bg-white/60 dark:bg-gray-800/60 text-gray-900 dark:text-gray-100 hover:bg-white dark:hover:bg-gray-700 border border-white/50 dark:border-gray-700/50 backdrop-blur-sm":
              variant === "secondary",
            "border-2 border-primary text-primary-600 dark:text-primary-300 hover:bg-primary/10 dark:hover:bg-primary/15":
              variant === "outline",
            "hover:bg-gray-100 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300":
              variant === "ghost",
            "bg-red-600 text-primary-100 hover:bg-red-700 shadow-md shadow-red-600/20 focus-visible:ring-red-500":
              variant === "danger",
            "h-9 px-4 text-sm": size === "sm",
            "h-11 px-6 text-sm": size === "md",
            "h-12 px-8 text-base": size === "lg",
          },
          className
        )}
        {...props}
      >
        {loading ? <Spinner /> : null}
        {children}
      </button>
    );
  }
);

Button.displayName = "Button";
