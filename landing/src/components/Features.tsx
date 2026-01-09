import { ShieldCheck, Bolt, LockPassword } from "@solar-icons/react";

const FeatureCard = ({
  icon: Icon,
  title,
  description,
  points,
  learnMoreLink,
}: {
  icon: any;
  title: string;
  description: string;
  points: string[];
  learnMoreLink?: string;
}) => (
  <div className="bg-zinc-50 rounded-2xl p-8 border border-zinc-100 hover:border-yellow-500/30 transition-all duration-300 hover:shadow-lg hover:shadow-yellow-500/5 group">
    <div className="w-12 h-12 bg-white rounded-xl flex items-center justify-center mb-6 shadow-sm border border-zinc-100 group-hover:scale-110 transition-transform duration-300">
      <Icon
        className="text-yellow-600"
        size={24}
      />
    </div>

    <h3 className="text-xl font-bold text-zinc-900 mb-3">{title}</h3>
    <p className="text-zinc-500 mb-6 leading-relaxed">{description}</p>

    <ul className="space-y-3 mb-6">
      {points.map((point, index) => (
        <li
          key={index}
          className="flex items-start gap-3 text-sm text-zinc-600"
        >
          <span className="w-1.5 h-1.5 rounded-full bg-yellow-500 mt-2 shrink-0" />
          {point}
        </li>
      ))}
    </ul>

    {learnMoreLink && (
      <a
        href={learnMoreLink}
        target="_blank"
        rel="noopener noreferrer"
        className="text-yellow-600 hover:text-yellow-700 text-sm font-semibold inline-flex items-center gap-1 group-hover:gap-2 transition-all"
      >
        Learn More →
      </a>
    )}
  </div>
);

const Features = () => {
  const features = [
    {
      icon: ShieldCheck,
      title: "Absolute Privacy",
      description:
        "Your financial data is yours alone. NozyWallet enforces privacy by default, making every transaction untraceable.",
      points: [
        "Shielded-by-default architecture",
        "Sender, receiver, and amount hidden",
        "Cryptographically proven privacy",
      ],
      learnMoreLink: "https://github.com/LEONINE-DAO/Nozy-wallet#-privacy-guarantees",
    },
    {
      icon: Bolt,
      title: "Blazing Fast",
      description:
        "Don't wait hours for privacy. NozyWallet combines Zcash's efficiency with Monero-grade privacy for instant settlements.",
      points: [
        "Transactions settle in seconds",
        "Low network fees",
        "Scalable for daily use",
      ],
      learnMoreLink: "https://github.com/LEONINE-DAO/Nozy-wallet/blob/main/PERFORMANCE.md",
    },
    {
      icon: LockPassword,
      title: "Enterprise-Grade Security",
      description:
        "Your funds are secured with industry-leading encryption. We take security seriously so you don't have to worry.",
      points: [
        "Self-custodial control",
        "Password protected storage",
        "Open source and auditable",
      ],
      learnMoreLink: "https://github.com/LEONINE-DAO/Nozy-wallet/blob/main/SECURITY.md",
    },
  ];

  return (
    <section
      id="features"
      className="py-24 bg-white relative overflow-hidden"
    >
      <div className="max-w-7xl mx-auto px-6 relative z-10">
        <div className="text-center max-w-3xl mx-auto mb-16">
          <h2 className="text-3xl lg:text-5xl font-bold text-zinc-900 mb-6">
            Privacy Without{" "}
            <span className="text-gradient-primary">Compromise</span>
          </h2>
          <p className="text-zinc-500 text-lg">
            NozyWallet combines the best privacy tech with modern UX to deliver
            a wallet that just works.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          {features.map((feature, index) => (
            <FeatureCard
              key={index}
              {...feature}
            />
          ))}
        </div>
      </div>
    </section>
  );
};

export default Features;
