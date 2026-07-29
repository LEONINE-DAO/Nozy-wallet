import logoUrl from "../assets/logo-transparent.png";
import logoMark from "../assets/logo-mark.png";

const Hero = () => {
  return (
    <section className="relative min-h-[100svh] flex items-center overflow-hidden">
      {/* Soft vignette so text stays readable over the starfield */}
      <div
        className="pointer-events-none absolute inset-0"
        aria-hidden
        style={{
          background:
            "radial-gradient(ellipse at 30% 40%, rgba(12,11,9,0.35), transparent 55%), linear-gradient(to bottom, rgba(12,11,9,0.2), rgba(12,11,9,0.75))",
        }}
      />

      <div className="relative z-10 w-full max-w-6xl mx-auto px-6 py-28 lg:py-32">
        <img
          src={logoUrl}
          alt="NozyWallet"
          className="h-14 sm:h-16 lg:h-20 w-auto object-contain object-left mb-8 animate-fade-in-up"
        />

        <h1 className="font-display font-extrabold tracking-tight text-[#f5f0e6] text-[clamp(2.75rem,8vw,5.5rem)] leading-[0.95] mb-6 animate-fade-in-up">
          Shielded ZEC.
          <br />
          <span className="text-gradient-primary">Yours alone.</span>
        </h1>

        <p className="max-w-xl text-lg sm:text-xl text-[#a39a88] leading-relaxed mb-10 animate-fade-in-up">
          Self-custodial Orchard wallet — private by default. CLI for operators,
          desktop for day-to-day, extension for the open web.
        </p>

        <div className="flex flex-col sm:flex-row flex-wrap gap-3 animate-fade-in-up">
          <a href="#download" className="nw-btn nw-btn-primary">
            Download CLI Lite
          </a>
          <a
            href="https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/extension-v0.1.7"
            target="_blank"
            rel="noopener noreferrer"
            className="nw-btn nw-btn-ghost"
          >
            Get extension beta
          </a>
        </div>

        <div className="mt-12 flex items-center gap-3 text-sm text-[#a39a88] animate-fade-in-up">
          <img src={logoMark} alt="" className="h-7 w-7 opacity-80" />
          <span>Orchard · Ironwood · open source Rust core</span>
        </div>
      </div>
    </section>
  );
};

export default Hero;
