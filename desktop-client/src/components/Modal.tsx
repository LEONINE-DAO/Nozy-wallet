import React, { useEffect, useId, useRef } from "react";
import { CloseCircle } from "@solar-icons/react";
import { createPortal } from "react-dom";

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  children: React.ReactNode;
  initialFocusRef?: React.RefObject<HTMLElement | null>;
}

export function Modal({ isOpen, onClose, title, children, initialFocusRef }: ModalProps) {
  const modalRef = useRef<HTMLDivElement>(null);
  const onCloseRef = useRef(onClose);
  const titleId = useId();

  onCloseRef.current = onClose;

  useEffect(() => {
    if (!isOpen) return;

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCloseRef.current();
    };

    document.addEventListener("keydown", handleEscape);
    document.body.style.overflow = "hidden";

    // Focus once when opened; never steal focus while the user is typing inside the dialog.
    const focusTarget =
      initialFocusRef?.current ??
      modalRef.current?.querySelector<HTMLElement>("input, textarea, select, button");
    window.requestAnimationFrame(() => {
      if (!focusTarget || !modalRef.current) return;
      if (modalRef.current.contains(document.activeElement)) return;
      focusTarget.focus();
    });

    return () => {
      document.removeEventListener("keydown", handleEscape);
      document.body.style.overflow = "unset";
    };
  }, [isOpen, initialFocusRef]);

  if (!isOpen) return null;

  return createPortal(
    <div className="fixed inset-0 z-[200]">
      <div
        className="absolute inset-0 bg-black/40 backdrop-blur-sm animate-fade-in"
        onClick={() => onCloseRef.current()}
        aria-hidden="true"
      />
      <div className="relative z-10 flex min-h-full items-center justify-center p-4 pointer-events-none">
        <div
          ref={modalRef}
          role="dialog"
          aria-modal="true"
          aria-labelledby={title ? titleId : undefined}
          className="bg-gray-900 rounded-2xl w-full max-w-lg shadow-2xl animate-scale-up border border-gray-600 max-h-[90vh] flex flex-col pointer-events-auto text-white"
          onMouseDown={(e) => e.stopPropagation()}
          onClick={(e) => e.stopPropagation()}
        >
          <div className="flex items-center justify-between px-6 py-5 border-b border-gray-700 shrink-0">
            <h3 id={titleId} className="text-lg font-semibold text-white">
              {title}
            </h3>
            <button
              type="button"
              onClick={() => onCloseRef.current()}
              className="p-2 rounded-xl hover:bg-gray-800 transition-colors text-gray-300 hover:text-white"
            >
              <CloseCircle size={24} />
            </button>
          </div>

          <div className="p-6 overflow-y-auto custom-scrollbar">{children}</div>
        </div>
      </div>
    </div>,
    document.body
  );
}
