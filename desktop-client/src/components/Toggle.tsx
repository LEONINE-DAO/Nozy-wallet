import React from "react";

interface ToggleProps {
  icon?: React.ReactNode;
  title: string;
  description?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  className?: string;
}

export function Toggle({
  icon,
  title,
  description,
  checked,
  onChange,
  className = "",
}: ToggleProps) {
  return (
    <div className={`flex items-center justify-between py-3 ${className}`}>
      <div className="flex items-center gap-4">
        {icon && (
          <div className="w-10 h-10 rounded-xl flex items-center justify-center transition-all duration-300 text-black">
            {React.isValidElement(icon)
              ? React.cloneElement(icon as React.ReactElement<any>, {
                  weight: checked ? "Bold" : "Linear",
                })
              : icon}
          </div>
        )}
        <div>
          <p className="font-medium text-gray-900 transition-colors">{title}</p>
          {description && (
            <p className="text-sm text-gray-500">{description}</p>
          )}
        </div>
      </div>
      <label className="relative inline-flex items-center cursor-pointer">
        <input
          type="checkbox"
          checked={checked}
          onChange={(e) => onChange(e.target.checked)}
          className="sr-only peer"
        />
        <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-[#f0a113]/20 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:left-0.5 after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[#f0a113]"></div>
      </label>
    </div>
  );
}
