import React, { useState, useRef, useEffect, useId } from "react";
import { createPortal } from "react-dom";

interface TooltipProps {
  content: string;
  children: React.ReactNode;
  placement?: "top" | "bottom";
}

export function Tooltip({ content, children, placement = "top" }: TooltipProps) {
  const [isOpen, setIsOpen] = useState(false);
  const wrapperRef = useRef<HTMLSpanElement>(null);
  const [position, setPosition] = useState({ top: 0, left: 0 });
  const id = useId().replace(/:/g, "");

  useEffect(() => {
    if (!isOpen || !wrapperRef.current) return;
    const rect = wrapperRef.current.getBoundingClientRect();
    const gap = 6;
    const tooltipHeight = 32;
    if (placement === "top") {
      setPosition({
        top: rect.top - tooltipHeight - gap,
        left: rect.left + rect.width / 2,
      });
    } else {
      setPosition({
        top: rect.bottom + gap,
        left: rect.left + rect.width / 2,
      });
    }
  }, [isOpen, placement]);

  const tooltipEl = isOpen && content
    ? createPortal(
        <div
          id={id}
          role="tooltip"
          className="fixed z-[100] px-3 py-1.5 text-xs font-medium text-white bg-gray-900 rounded-lg shadow-lg whitespace-nowrap pointer-events-none animate-fade-in"
          style={{
            left: position.left,
            top: position.top,
            transform: "translate(-50%, 0)",
          }}
        >
          {content}
        </div>,
        document.body
      )
    : null;

  return (
    <>
      <span
        ref={wrapperRef}
        className="inline-flex"
        onMouseEnter={() => setIsOpen(true)}
        onMouseLeave={() => setIsOpen(false)}
        onFocus={() => setIsOpen(true)}
        onBlur={() => setIsOpen(false)}
        aria-describedby={isOpen ? id : undefined}
      >
        {children}
      </span>
      {tooltipEl}
    </>
  );
}
