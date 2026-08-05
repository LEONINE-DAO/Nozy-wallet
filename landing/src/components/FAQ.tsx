import { useState } from "react";
import { AltArrowDown } from "@solar-icons/react";

const FAQItem = ({
  question,
  answer,
  isOpen,
  onClick,
}: {
  question: string;
  answer: string;
  isOpen: boolean;
  onClick: () => void;
}) => (
  <div className="border-b border-[rgba(245,240,230,0.12)]">
    <button
      onClick={onClick}
      className="w-full flex items-center justify-between py-5 text-left gap-4"
    >
      <span className="font-display text-lg font-semibold text-[#f5f0e6]">{question}</span>
      <AltArrowDown
        className={`text-[#c8ccd4] shrink-0 transition-transform duration-300 ${
          isOpen ? "rotate-180" : ""
        }`}
        size={22}
      />
    </button>
    <div
      className={`grid transition-all duration-300 ease-in-out ${
        isOpen ? "grid-rows-[1fr] opacity-100 pb-5" : "grid-rows-[0fr] opacity-0"
      }`}
    >
      <div className="overflow-hidden">
        <p className="text-[#a39a88] leading-relaxed max-w-3xl">{answer}</p>
      </div>
    </div>
  </div>
);

const FAQ = () => {
  const [openIndex, setOpenIndex] = useState<number | null>(0);

  const faqData = [
    {
      question: "Is NozyWallet as private as Monero?",
      answer:
        "NozyWallet is shielded-first Zcash: Orchard (and Ironwood) hide sender, receiver, and amount. Transparent addresses are blocked for sends.",
    },
    {
      question: "Can I send transparent transactions?",
      answer:
        "No. NozyWallet blocks transparent recipients so privacy stays the default path.",
    },
    {
      question: "Which surface should I use?",
      answer:
        "CLI Lite for operators and production mainnet. Desktop for a native wallet UI (no browser). Companion API for localhost extension / local-app bridge. Extension for sites and dApps. Mobile when the phone companion ships.",
    },
    {
      question: "Do I need my own node?",
      answer:
        "Yes for full independence — pair with Zebrad + lightwalletd you control. The companion API runs on localhost (default :3000) next to the extension or your apps.",
    },
    {
      question: "Where do I get support?",
      answer:
        "Email support.team@nozywallet.com, or open a GitHub issue (never paste seed phrases).",
    },
  ];

  return (
    <section id="faq" className="py-24 border-t border-[rgba(245,240,230,0.12)] scroll-mt-24">
      <div className="max-w-3xl mx-auto px-6">
        <p className="nw-kicker mb-4">FAQ</p>
        <h2 className="font-display text-3xl lg:text-5xl font-bold text-[#f5f0e6] mb-10 tracking-tight">
          Questions
        </h2>
        <div>
          {faqData.map((item, index) => (
            <FAQItem
              key={item.question}
              question={item.question}
              answer={item.answer}
              isOpen={openIndex === index}
              onClick={() => setOpenIndex(openIndex === index ? null : index)}
            />
          ))}
        </div>
      </div>
    </section>
  );
};

export default FAQ;
