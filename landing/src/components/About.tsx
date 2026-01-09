import { Download } from "@solar-icons/react";

const About = () => {
  return (
    <section
      id="about"
      className="py-24 bg-zinc-900 text-white relative overflow-hidden"
    >
      <div className="absolute top-0 right-0 w-1/2 h-full bg-yellow-500/5 blur-3xl rounded-full translate-x-1/2" />
      <div className="absolute bottom-0 left-0 w-1/2 h-full bg-zinc-800/20 blur-3xl rounded-full -translate-x-1/2" />

      <div className="max-w-7xl mx-auto px-6 relative z-10 text-center">
        <h2 className="text-4xl lg:text-5xl font-bold mb-8">
          Experience Monero-Level Privacy with{" "}
          <span className="text-yellow-500">Zcash Speed</span>
        </h2>

        <p className="text-xl text-zinc-400 max-w-2xl mx-auto mb-12">
          Download NozyWallet today. Privacy by default. No compromises.
        </p>

        <div className="flex flex-col sm:flex-row items-center justify-center gap-6">
          <button className="bg-yellow-500 hover:bg-yellow-400 text-zinc-900 px-8 py-4 rounded-xl font-bold text-lg transition-all shadow-lg hover:shadow-yellow-500/20 flex items-center gap-3">
            <Download size={24} />
            Download For Windows
          </button>
          <button className="bg-white/10 hover:bg-white/20 text-white px-8 py-4 rounded-xl font-bold text-lg transition-all border border-white/10">
            Read The Manifesto
          </button>
        </div>
      </div>
    </section>
  );
};

export default About;
