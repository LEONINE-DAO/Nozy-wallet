const About = () => {
  return (
    <section
      id="about"
      className="py-24 border-t border-[rgba(245,240,230,0.12)] relative overflow-hidden scroll-mt-24"
    >
      <div
        className="pointer-events-none absolute inset-0 opacity-70"
        style={{
          background:
            "radial-gradient(ellipse at 70% 40%, rgba(200,205,212,0.12), transparent 55%)",
        }}
        aria-hidden
      />

      <div className="max-w-3xl mx-auto px-6 relative z-10">
        <p className="nw-kicker mb-4">About</p>
        <h2 className="font-display text-3xl lg:text-5xl font-bold text-[#f5f0e6] mb-6 tracking-tight">
          Privacy is a right, not a privilege.
        </h2>
        <p className="text-lg text-[#a39a88] leading-relaxed mb-10">
          NozyWallet is an open-source, shielded-first Zcash wallet. Self-custody,
          Orchard-first design, and a clear split between wallet surfaces and the open web.
        </p>
        <div className="flex flex-col sm:flex-row gap-3">
          <a href="#download" className="nw-btn nw-btn-primary">
            Download
          </a>
          <a
            href="https://leonine-dao.github.io/Nozy-wallet/book/nozy/manifesto.html"
            target="_blank"
            rel="noopener noreferrer"
            className="nw-btn nw-btn-ghost"
          >
            Read the manifesto
          </a>
        </div>
      </div>
    </section>
  );
};

export default About;
