import { useState } from "react";
import { AltArrowDown, QuestionCircle } from "@solar-icons/react";

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
  <div className="border border-zinc-200 rounded-2xl overflow-hidden bg-white/50 backdrop-blur-sm transition-all duration-300 hover:border-yellow-500/30">
    <button
      onClick={onClick}
      className="w-full flex items-center justify-between p-6 text-left focus:outline-none"
    >
      <span className="text-lg font-bold text-zinc-900">{question}</span>
      <AltArrowDown
        className={`text-zinc-400 transition-transform duration-300 ${
          isOpen ? "rotate-180 text-yellow-500" : ""
        }`}
        size={24}
      />
    </button>
    <div
      className={`grid transition-all duration-300 ease-in-out ${
        isOpen
          ? "grid-rows-[1fr] opacity-100 pb-6"
          : "grid-rows-[0fr] opacity-0"
      }`}
    >
      <div className="overflow-hidden px-6">
        <p className="text-zinc-600 leading-relaxed">{answer}</p>
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
        "Yes. NozyWallet provides the same level of privacy as Monero. Every transaction is private, untraceable, and fungible.",
    },
    {
      question: "Can I send transparent transactions?",
      answer:
        "No. NozyWallet blocks transparent addresses to enforce privacy. You can only send shielded transactions.",
    },
    {
      question: "Why choose NozyWallet over Monero?",
      answer:
        "Same privacy, but faster block times, lower fees, and better scalability. Plus, clearer regulatory status.",
    },
    {
      question: "How does Orchard privacy work?",
      answer:
        "Orchard uses zero-knowledge proofs (zkSNARKs) to hide sender, receiver, and amount. It's cryptographically proven, not probabilistic.",
    },
  ];

  return (
    <section
      id="faq"
      className="py-24 relative bg-zinc-50/50"
    >
      <div className="max-w-7xl mx-auto px-6">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-12 lg:gap-24">
          <div className="lg:col-span-4">
            <div className="sticky top-32">
              <div className="inline-flex items-center gap-2 text-yellow-600 font-semibold mb-4">
                <QuestionCircle size={20} />
                <span className="uppercase tracking-wider text-sm">
                  Support
                </span>
              </div>
              <h2 className="text-4xl lg:text-6xl font-bold text-zinc-900 mb-6">
                Common
                <br />
                <span className="text-gradient-primary">Questions</span>
              </h2>
              <p className="text-zinc-500 text-lg leading-relaxed">
                Everything you need to know about NozyWallet privacy and
                security features.
              </p>
            </div>
          </div>

          <div className="lg:col-span-8 flex flex-col gap-4">
            {faqData.map((item, index) => (
              <FAQItem
                key={index}
                question={item.question}
                answer={item.answer}
                isOpen={openIndex === index}
                onClick={() => setOpenIndex(openIndex === index ? null : index)}
              />
            ))}
          </div>
        </div>
      </div>
    </section>
  );
};

export default FAQ;
