import { AltArrowDown, ShieldKeyholeMinimalistic } from "@solar-icons/react";

const Hero = () => {
  return (
    <section className="relative pt-32 pb-20 lg:pt-48 lg:pb-32 overflow-hidden bg-white">
      <div className="absolute top-0 left-1/2 -translate-x-1/2 w-full h-full max-w-7xl pointer-events-none">
        <div
          className="absolute top-20 left-20 w-72 h-72 bg-yellow-400/20 rounded-full blur-3xl animate-pulse"
          style={{ animationDuration: "4s" }}
        ></div>
        <div
          className="absolute bottom-20 right-20 w-96 h-96 bg-amber-200/40 rounded-full blur-3xl animate-pulse"
          style={{ animationDuration: "7s" }}
        ></div>
      </div>

      <div className="max-w-7xl mx-auto px-6 text-center relative z-10">
        <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-yellow-500/10 border border-yellow-500/20 text-xs font-medium text-yellow-700 mb-8 animate-fade-in-up">
          <ShieldKeyholeMinimalistic size={14} />
          <span>Privacy by Default</span>
        </div>

        <h1 className="text-5xl lg:text-7xl font-bold tracking-tight mb-6 leading-tight text-zinc-900">
          <span className="block">Monero-Level Privacy.</span>
          <span className="text-gradient-primary">Zcash Speed.</span>
        </h1>

        <p className="max-w-2xl mx-auto text-lg text-zinc-600 mb-10 leading-relaxed">
          NozyWallet is the only wallet that enforces complete privacy for every
          transaction. Send and receive money instantly, without leaving a
          trace.
        </p>

        <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
          <a
            href="https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest"
            target="_blank"
            rel="noopener noreferrer"
            className="w-full sm:w-auto px-8 py-4 bg-yellow-500 hover:bg-yellow-400 text-white rounded-xl font-bold transition-all shadow-lg shadow-yellow-500/20 flex items-center justify-center gap-2 group"
          >
            <AltArrowDown
              size={20}
              className="group-hover:translate-y-1 transition-transform"
            />
            Download NozyWallet
          </a>
          <a
            href="https://leonine-dao.github.io/Nozy-wallet/book/"
            target="_blank"
            rel="noopener noreferrer"
            className="w-full sm:w-auto px-8 py-4 bg-zinc-100 hover:bg-zinc-200 text-zinc-900 rounded-xl font-semibold transition-all border border-zinc-200 hover:border-zinc-300"
          >
            View Documentation
          </a>
        </div>

        <div className="mt-20 pt-10 border-t border-zinc-200 grid grid-cols-1 sm:grid-cols-3 gap-8">
          <div className="flex flex-col items-center gap-2">
            <span className="text-3xl font-bold text-zinc-900">100%</span>
            <span className="text-sm text-zinc-500 uppercase tracking-wider">
              Private
            </span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <span className="text-3xl font-bold text-zinc-900">Zero</span>
            <span className="text-sm text-zinc-500 uppercase tracking-wider">
              Tracking
            </span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <span className="text-3xl font-bold text-zinc-900">Instant</span>
            <span className="text-sm text-zinc-500 uppercase tracking-wider">
              Settlement
            </span>
          </div>
        </div>
      </div>
    </section>
  );
};

export default Hero;
