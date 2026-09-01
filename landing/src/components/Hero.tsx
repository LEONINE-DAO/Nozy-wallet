import BrandLogo from "./BrandLogo";
import { PATHS } from "../lib/links";

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
        <div className="mb-8 animate-fade-in-up">
          <BrandLogo
            markClassName="h-12 w-12 sm:h-14 sm:w-14"
            className="text-2xl sm:text-3xl"
          />
        </div>

        <h1 className="font-display font-extrabold tracking-tight text-[#f5f0e6] text-[clamp(2.75rem,8vw,5.5rem)] leading-[0.95] mb-6 animate-fade-in-up">
          Shielded ZEC.
          <br />
          <span className="text-gradient-primary">Yours alone.</span>
        </h1>

        <p className="max-w-xl text-lg sm:text-xl text-[#a39a88] leading-relaxed mb-10 animate-fade-in-up">
          Self-custodial shielded Zcash — Orchard and Ironwood. CLI for operators,
          desktop for day-to-day, extension for the open web. Still holding Orchard?
          Finish ZIP 318 migration.
        </p>

        <div className="flex flex-col sm:flex-row flex-wrap gap-3 animate-fade-in-up">
          <a href="#download" className="nw-btn nw-btn-primary">
            Download CLI Lite
          </a>
          <a href={PATHS.ironwoodPage} className="nw-btn nw-btn-ghost">
            Finish Ironwood migration
          </a>
          <a
            href={PATHS.extensionRelease}
            target="_blank"
            rel="noopener noreferrer"
            className="nw-btn nw-btn-ghost"
          >
            Get extension
          </a>
        </div>

        <div className="mt-12 flex items-center gap-3 text-sm text-[#a39a88] animate-fade-in-up">
          <BrandLogo showWordmark={false} markClassName="h-7 w-7" alt="" />
          <span>Orchard · Ironwood · open source Rust core</span>
        </div>
      </div>
    </section>
  );
};

export default Hero;
