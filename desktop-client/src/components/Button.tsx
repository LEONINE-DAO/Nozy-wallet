import React from "react";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

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
          "inline-flex items-center justify-center rounded-lg font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 disabled:pointer-events-none disabled:opacity-50",
          {
            "bg-[#f0a113] text-black shadow-lg shadow-[#f0a113]/20":
              variant === "primary",
            "bg-white/60 text-gray-900 hover:bg-white border border-transparent hover:border-white/50 backdrop-blur-sm":
              variant === "secondary",
            "border-2 border-[#f0a113] text-[#f0a113] hover:bg-[#f0a113]-50":
              variant === "outline",
            "hover:bg-gray-100 text-gray-700": variant === "ghost",
            "bg-red-600 text-black hover:bg-red-700 shadow-lg shadow-red-600/20 focus-visible:ring-red-500":
              variant === "danger",
            "h-9 px-4 text-sm": size === "sm",
            "h-11 px-8 text-base": size === "md",
            "h-14 px-10 text-lg": size === "lg",
          },
          className
        )}
        {...props}
      />
    );
  }
);
