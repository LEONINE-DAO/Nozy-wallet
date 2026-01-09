const Privacy = () => {
  return (
    <div className="max-w-4xl mx-auto px-6 py-24 my-24">
      <h1 className="text-4xl font-bold text-zinc-900 mb-8">Privacy Policy</h1>
      
      <div className="prose prose-zinc max-w-none space-y-8 text-zinc-600">
        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">1. Introduction</h2>
          <p>
            At NozyWallet, we respect your privacy and are committed to protecting it through our compliance with this policy. 
            This policy describes the types of information we may collect from you or that you may provide when you visit the website 
            and our practices for collecting, using, maintaining, protecting, and disclosing that information.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">2. Privacy by Default</h2>
          <p>
            NozyWallet is a privacy-first Orchard wallet that enforces complete transaction privacy by default. 
            Unlike other Zcash wallets, NozyWallet only supports shielded transactions - making it functionally 
            equivalent to Monero in terms of privacy, but with faster block times and lower fees.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">3. Privacy Guarantees</h2>
          <ul className="list-disc pl-6 space-y-2">
            <li><strong>Every transaction is private:</strong> No transparent transactions are possible.</li>
            <li><strong>Untraceable:</strong> Sender, receiver, and amount are all hidden.</li>
            <li><strong>Fungible:</strong> No blacklisted or tainted coins.</li>
            <li><strong>Zero-knowledge proofs:</strong> Cryptographically proven privacy.</li>
            <li><strong>Accident-proof:</strong> You cannot accidentally compromise your privacy.</li>
          </ul>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">4. Local Storage</h2>
          <p>
            Your wallet information is encrypted and stored locally on your device. We do not have access to your funds 
            or the ability to recover your wallet if you lose your credentials.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">5. Third-Party Services</h2>
          <p>
            Our application may interact with third-party blockchain nodes or APIs to fetch balance information and broadcast transactions. 
            These services may log your IP address or transaction data according to their own privacy policies.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">6. Updates to This Policy</h2>
          <p>
            We may update our privacy policy from time to time. We will notify you of any changes by posting the new privacy policy on this page.
          </p>
        </section>
        
        {/* <p className="text-sm text-zinc-400 mt-12 pt-8 border-t border-zinc-100">
          Last updated: {new Date().toLocaleDateString()}
        </p> */}
      </div>
    </div>
  );
};

export default Privacy;
