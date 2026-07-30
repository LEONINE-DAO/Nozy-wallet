import { useEffect, useState } from "react";
import BrandLogo from "./BrandLogo";

type Props = {
  onDone?: () => void;
};

/**
 * Brief boot loader — silver pulse + progress, then fades out.
 */
export default function PageLoader({ onDone }: Props) {
  const [phase, setPhase] = useState<"in" | "out" | "gone">("in");
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduceMotion) {
      setProgress(100);
      setPhase("gone");
      onDone?.();
      return;
    }

    const start = performance.now();
    const duration = 1400;
    let raf = 0;

    const tick = (now: number) => {
      const t = Math.min(1, (now - start) / duration);
      // ease-out cubic
      const eased = 1 - Math.pow(1 - t, 3);
      setProgress(Math.round(eased * 100));
      if (t < 1) {
        raf = requestAnimationFrame(tick);
      } else {
        setPhase("out");
        window.setTimeout(() => {
          setPhase("gone");
          onDone?.();
        }, 420);
      }
    };

    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [onDone]);

  if (phase === "gone") return null;

  return (
    <div
      className={`fixed inset-0 z-[100] flex flex-col items-center justify-center bg-[#0c0b09] transition-opacity duration-[420ms] ease-out ${
        phase === "out" ? "opacity-0 pointer-events-none" : "opacity-100"
      }`}
      role="status"
      aria-live="polite"
      aria-label="Loading NozyWallet"
    >
      <div className="nw-loader-stars" aria-hidden />

      <div className="mb-8 nw-loader-pulse">
        <BrandLogo markClassName="h-16 w-16" className="text-2xl" />
      </div>

      <div className="w-48 h-[2px] bg-[rgba(245,240,230,0.12)] overflow-hidden">
        <div
          className="h-full bg-gradient-to-r from-[#8b919c] via-[#e8eaed] to-[#c8ccd4] transition-[width] duration-75 ease-out"
          style={{ width: `${progress}%` }}
        />
      </div>
      <p className="mt-3 text-xs tracking-[0.2em] uppercase text-[#a39a88]">
        Initializing
      </p>
    </div>
  );
}
