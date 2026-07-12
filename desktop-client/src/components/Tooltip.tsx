import React, { useState, useRef, useEffect, useId, useLayoutEffect } from "react";
import { createPortal } from "react-dom";

interface TooltipProps {
  content: string;
  children: React.ReactNode;
  placement?: "top" | "bottom" | "auto";
}

export function Tooltip({ content, children, placement = "auto" }: TooltipProps) {
  const [isOpen, setIsOpen] = useState(false);
  const wrapperRef = useRef<HTMLSpanElement>(null);
  const tooltipRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState({ top: 0, left: 0 });
  const [resolvedPlacement, setResolvedPlacement] = useState<"top" | "bottom">("bottom");
  const id = useId().replace(/:/g, "");

  useLayoutEffect(() => {
    if (!isOpen || !wrapperRef.current) return;
    const rect = wrapperRef.current.getBoundingClientRect();
    const gap = 8;
    const estimatedHeight = tooltipRef.current?.offsetHeight ?? 36;
    const spaceAbove = rect.top;
    const spaceBelow = window.innerHeight - rect.bottom;

    let place: "top" | "bottom";
    if (placement === "top" || placement === "bottom") {
      place = placement;
      // Flip if the preferred side would clip off-screen.
      if (place === "top" && spaceAbove < estimatedHeight + gap && spaceBelow > spaceAbove) {
        place = "bottom";
      } else if (place === "bottom" && spaceBelow < estimatedHeight + gap && spaceAbove > spaceBelow) {
        place = "top";
      }
    } else {
      // Prefer bottom so top-of-window header tooltips stay visible.
      place = spaceAbove < estimatedHeight + gap + 4 ? "bottom" : "top";
      if (place === "top" && spaceAbove < estimatedHeight + gap) place = "bottom";
    }

    setResolvedPlacement(place);
    setPosition({
      top: place === "top" ? rect.top - gap : rect.bottom + gap,
      left: Math.min(
        Math.max(rect.left + rect.width / 2, 12),
        window.innerWidth - 12
      ),
    });
  }, [isOpen, placement, content]);

  useEffect(() => {
    if (!isOpen) return;
    const close = () => setIsOpen(false);
    window.addEventListener("scroll", close, true);
    window.addEventListener("resize", close);
    return () => {
      window.removeEventListener("scroll", close, true);
      window.removeEventListener("resize", close);
    };
  }, [isOpen]);

  const tooltipEl = isOpen && content
    ? createPortal(
        <div
          ref={tooltipRef}
          id={id}
          role="tooltip"
          className="fixed z-[300] max-w-xs px-3 py-2 text-sm font-semibold text-white bg-gray-950 dark:bg-gray-100 dark:text-gray-900 rounded-xl shadow-xl border border-white/10 dark:border-gray-900/10 pointer-events-none animate-fade-in"
          style={{
            left: position.left,
            top: position.top,
            transform:
              resolvedPlacement === "top"
                ? "translate(-50%, -100%)"
                : "translate(-50%, 0)",
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
