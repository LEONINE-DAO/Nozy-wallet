import logoMark from "../assets/logo-mark-clean.png";

type Props = {
  /** Visual size of the mark */
  markClassName?: string;
  /** Show the Syne wordmark next to the mark */
  showWordmark?: boolean;
  className?: string;
  /** Accessible label when used as a lone image */
  alt?: string;
};

/**
 * Clean brand lockup: lightened existing mark + CSS wordmark (no haloed PNG text).
 */
export default function BrandLogo({
  markClassName = "h-9 w-9",
  showWordmark = true,
  className = "",
  alt = "NozyWallet",
}: Props) {
  return (
    <span className={`inline-flex items-center gap-2.5 ${className}`}>
      <img
        src={logoMark}
        alt={showWordmark ? "" : alt}
        className={`${markClassName} object-contain object-center block brightness-[1.12] contrast-[1.06] saturate-[0.92]`}
        draggable={false}
      />
      {showWordmark ? (
        <span className="font-display font-bold tracking-tight text-[#f5f0e6] leading-none">
          Nozy
          <span className="text-[#c8ccd4]">Wallet</span>
        </span>
      ) : null}
    </span>
  );
}
