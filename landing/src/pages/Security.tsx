const Security = () => {
  return (
    <div className="max-w-4xl mx-auto px-6 py-24 my-24">
      <h1 className="text-4xl font-bold text-zinc-900 mb-8">Security Policy</h1>
      
      <div className="prose prose-zinc max-w-none space-y-8 text-zinc-600">
        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">1. Security Architecture</h2>
          <p>
            NozyWallet is a non-custodial wallet. This means you have full control over your funds. 
            Private keys are generated client-side and never leave your device unencrypted.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">2. Password Protection</h2>
          <ul className="list-disc pl-6 space-y-2">
            <li>Uses <strong>Argon2</strong> for password hashing.</li>
            <li>Salt is randomly generated for each wallet.</li>
            <li>Passwords are never stored in plain text.</li>
          </ul>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">3. Wallet Storage</h2>
          <ul className="list-disc pl-6 space-y-2">
            <li>Wallets are encrypted with <strong>AES-256-GCM</strong>.</li>
            <li>Encryption key is derived from your password.</li>
            <li>Backup files are also encrypted to ensure safety.</li>
          </ul>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">4. Private Key Management</h2>
          <ul className="list-disc pl-6 space-y-2">
            <li>Private keys are never stored in plain text.</li>
            <li>Keys are derived from mnemonic phrases using <strong>BIP32</strong>.</li>
            <li>Spending keys are only loaded into memory when absolutely needed.</li>
          </ul>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">5. Network Upgrades</h2>
          <p>
             NozyWallet is fully updated and ready for Zcash Network Upgrade 6.1 (NU 6.1). 
             We support Protocol Version 170140 and the latest privacy features like ZIP 271 and ZIP 1016.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">6. Reporting Vulnerabilities</h2>
          <p>
            If you discover a security vulnerability, please report it to us immediately via our GitHub repository or official contact channels. 
            We take all security reports seriously and will investigate them promptly.
          </p>
        </section>

        {/* <p className="text-sm text-zinc-400 mt-12 pt-8 border-t border-zinc-100">
          Last updated: {new Date().toLocaleDateString()}
        </p> */}
      </div>
    </div>
  );
};

export default Security;
