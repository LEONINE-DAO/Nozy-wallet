import { ShieldCheck, Bolt, LockPassword } from "@solar-icons/react";

const features = [
  {
    icon: ShieldCheck,
    title: "Shielded by default",
    description:
      "Orchard and Ironwood only. Transparent sends are blocked — privacy is the product, not a toggle.",
    points: [
      "Orchard + Ironwood pools",
      "Amounts and parties stay private",
      "Zero-knowledge proofs",
    ],
    href: "https://leonine-dao.github.io/Nozy-wallet/book/features/absolute-privacy.html",
  },
  {
    icon: Bolt,
    title: "Built for Zebrad",
    description:
      "Designed for the modern Zcash stack — local witnesses, lightwalletd sync, operator-grade tooling.",
    points: [
      "Zebrad + lightwalletd",
      "Witness-aware sends",
      "Ironwood migrate path",
    ],
    href: "https://leonine-dao.github.io/Nozy-wallet/book/features/performance.html",
  },
  {
    icon: LockPassword,
    title: "Your keys",
    description:
      "Self-custodial everywhere. Seed and keys stay on devices you control — never on our servers.",
    points: [
      "Local encrypted storage",
      "Open-source Rust core",
      "No custodial accounts",
    ],
    href: "https://leonine-dao.github.io/Nozy-wallet/book/features/security.html",
  },
];

const Features = () => {
  return (
    <section id="features" className="py-24 border-t border-[rgba(245,240,230,0.12)] scroll-mt-24">
      <div className="max-w-7xl mx-auto px-6">
        <div className="max-w-2xl mb-14">
          <p className="nw-kicker mb-4">Why Nozy</p>
          <h2 className="font-display text-3xl lg:text-5xl font-bold text-[#f5f0e6] mb-4 tracking-tight">
            Privacy without the theater
          </h2>
          <p className="text-[#a39a88] text-lg">
            A shielded-first wallet with a clear surface split — not another everything-app.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-8 md:gap-10">
          {features.map((feature) => {
            const Icon = feature.icon;
            return (
              <div key={feature.title} className="border-t border-[rgba(245,240,230,0.12)] pt-8">
                <Icon className="text-[#c8ccd4] mb-5" size={28} />
                <h3 className="font-display text-xl font-bold text-[#f5f0e6] mb-3">
                  {feature.title}
                </h3>
                <p className="text-[#a39a88] mb-6 leading-relaxed">{feature.description}</p>
                <ul className="space-y-2 mb-6">
                  {feature.points.map((point) => (
                    <li key={point} className="text-sm text-[#a39a88] flex gap-2">
                      <span className="text-[#c8ccd4]">—</span>
                      {point}
                    </li>
                  ))}
                </ul>
                <a
                  href={feature.href}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-[#c8ccd4] text-sm font-semibold hover:underline"
                >
                  Learn more →
                </a>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
};

export default Features;
