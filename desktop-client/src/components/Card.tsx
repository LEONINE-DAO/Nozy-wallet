import React from "react";
import { cn } from "../lib/cn";

interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: "glass" | "solid" | "elevated";
  padding?: "none" | "sm" | "md" | "lg";
}

const paddingClasses = {
  none: "",
  sm: "p-4",
  md: "p-6",
  lg: "p-8",
};

export function Card({
  className,
  variant = "glass",
  padding = "md",
  children,
  ...props
}: CardProps) {
  return (
    <div
      className={cn(
        "rounded-2xl border",
        variant === "glass" &&
          "bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm border-white/50 dark:border-gray-700/50 shadow-sm",
        variant === "solid" &&
          "bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700 shadow-sm",
        variant === "elevated" &&
          "bg-white/80 dark:bg-gray-800/80 backdrop-blur-xl border-white/50 dark:border-gray-700/50 shadow-lg shadow-primary/5",
        paddingClasses[padding],
        className
      )}
      {...props}
    >
      {children}
    </div>
  );
}
