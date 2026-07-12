import React from "react";
import { cn } from "../lib/cn";

export { cn };

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "outline" | "ghost" | "danger";
  size?: "sm" | "md" | "lg";
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = "primary", size = "md", ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(
          "inline-flex items-center justify-center rounded-xl font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50 focus-visible:ring-offset-2 focus-visible:ring-offset-transparent disabled:pointer-events-none disabled:opacity-50 active:scale-[0.98]",
          {
            "bg-primary text-gray-900 shadow-md shadow-primary/20 hover:bg-primary-300":
              variant === "primary",
            "bg-white/60 dark:bg-gray-800/60 text-gray-900 dark:text-gray-100 hover:bg-white dark:hover:bg-gray-700 border border-white/50 dark:border-gray-700/50 backdrop-blur-sm":
              variant === "secondary",
            "border-2 border-primary text-primary-600 dark:text-primary-300 hover:bg-primary/10 dark:hover:bg-primary/15":
              variant === "outline",
            "hover:bg-gray-100 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300":
              variant === "ghost",
            "bg-red-600 text-white hover:bg-red-700 shadow-md shadow-red-600/20 focus-visible:ring-red-500":
              variant === "danger",
            "h-9 px-4 text-sm": size === "sm",
            "h-11 px-6 text-sm": size === "md",
            "h-12 px-8 text-base": size === "lg",
          },
          className
        )}
        {...props}
      />
    );
  }
);

Button.displayName = "Button";
