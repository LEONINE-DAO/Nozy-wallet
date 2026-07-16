const LAST_UPDATED = "July 16, 2026";

const Privacy = () => {
  return (
    <div className="max-w-4xl mx-auto px-6 py-24 my-24">
      <h1 className="text-4xl font-bold text-zinc-900 mb-2">Privacy Policy</h1>
      <p className="text-sm text-zinc-500 mb-10">Last updated: {LAST_UPDATED}</p>

      <div className="prose prose-zinc max-w-none space-y-10 text-zinc-600">
        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">
            1. Who we are
          </h2>
          <p>
            This policy describes how <strong>NozyWallet</strong> (developed by
            LEONINE DAO) handles information when you use our website, desktop
            app, mobile app, CLI, or related documentation. NozyWallet is an
            open-source Orchard / Ironwood (shielded) Zcash wallet. We do not
            operate a custodial exchange and we do not hold your funds.
          </p>
          <p>
            Contact:{" "}
            <a
              href="mailto:support@leoninedao.org"
              className="text-yellow-700 hover:underline"
            >
              support@leoninedao.org
            </a>
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">
            2. On-chain vs off-chain privacy (Orchard/Ironwood vs API/node
            logging)
          </h2>
          <p>
            <strong>On-chain (Orchard / Ironwood):</strong> Shielded
            transactions hide sender, receiver, and amount on the public Zcash
            ledger. NozyWallet is shielded-only by design — transparent
            addresses are not supported. Ironwood (NU6.3) migration and related
            wallet tools follow the same shielded privacy model on-chain.
          </p>
          <p>
            <strong>Off-chain (API / node logging):</strong> Orchard and
            Ironwood do not hide your IP address, sync schedule, or the fact
            that you contacted an API or node. Anyone who runs the
            infrastructure you use may log connection metadata. For strongest
            privacy, run your own Zebrad node and your own NozyWallet API on
            hardware you control.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">
            3. NozyWallet Mobile (companion app)
          </h2>
          <p>
            The mobile app is a <strong>companion client</strong>. It does not
            download the full blockchain. It sends HTTPS requests to a{" "}
            <strong>NozyWallet API server</strong> you configure (for example
            your home PC or a VPS). That API connects to a Zebra node for sync,
            proving, and broadcast.
          </p>
          <h3 className="text-lg font-semibold text-zinc-800 mt-6 mb-2">
            Stored on your phone
          </h3>
          <ul className="list-disc pl-6 space-y-2">
            <li>
              API server URL and optional API key (device storage /
              AsyncStorage)
            </li>
            <li>Session preferences (for example unlock state, display theme)</li>
            <li>
              No wallet seed is required to stay on the phone in companion mode;
              wallet files live on the API server you connect to
            </li>
          </ul>
          <h3 className="text-lg font-semibold text-zinc-800 mt-6 mb-2">
            Stored on your API server
          </h3>
          <ul className="list-disc pl-6 space-y-2">
            <li>
              Wallet scan data, notes, and transaction history needed to show
              balance and send shielded ZEC
            </li>
            <li>
              Seed phrase and keys if you create or restore a wallet through
              that API
            </li>
          </ul>
          <p className="mt-4">
            <strong>If you use someone else&apos;s hosted API</strong> (including
            any Nozy-operated service), that operator can see when you connect,
            your IP address, and wallet data stored on their server. Read the
            in-app hosted-mode disclosure before using a third-party API.
          </p>
          <p>
            <strong>Current product note:</strong> NozyWallet does not yet
            operate its own Zebrad node for public mobile hosting. Sync requires
            a Zebrad reachable from the API you use — typically your own home
            or VPS setup, or another operator you trust.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">
            4. Desktop app and CLI
          </h2>
          <p>
            Desktop and command-line builds run wallet logic locally or against a
            node you configure. Wallet files and keys are stored on your machine
            under your user profile unless you choose a remote API deployment.
            The same on-chain / off-chain distinction applies: your Zebrad
            operator (often you) may see RPC and sync metadata.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">
            5. What we collect
          </h2>
          <p>
            <strong>We do not sell personal data.</strong> NozyWallet software
            does not include third-party advertising or in-app analytics SDKs.
          </p>
          <ul className="list-disc pl-6 space-y-2 mt-4">
            <li>
              <strong>Mobile app:</strong> No telemetry is sent to NozyWallet by
              default. Connection settings stay on your device unless you
              configure an API that logs requests on its server.
            </li>
            <li>
              <strong>This website:</strong> Our hosting provider may collect
              standard web logs (IP address, browser type, pages visited). If
              analytics are enabled on the marketing site, they are used only to
              understand traffic to this website — not to track wallet usage.
            </li>
            <li>
              <strong>Support:</strong> If you email us or open a GitHub issue,
              we receive the information you choose to send.
            </li>
          </ul>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">
            6. Third-party services
          </h2>
          <p>You may choose to connect NozyWallet to:</p>
          <ul className="list-disc pl-6 space-y-2 mt-4">
            <li>
              <strong>Zebra nodes</strong> — for blockchain sync and
              transaction broadcast
            </li>
            <li>
              <strong>lightwalletd</strong> — optional compact sync (experimental
              mobile paths)
            </li>
            <li>
              <strong>Your own or third-party API hosts</strong> — for mobile
              companion mode
            </li>
            <li>
              <strong>Block explorers</strong> — when you open transaction links
              from the app
            </li>
          </ul>
          <p className="mt-4">
            Each service has its own privacy practices. We do not control what a
            remote node or API operator logs.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">
            7. Security
          </h2>
          <ul className="list-disc pl-6 space-y-2">
            <li>
              Use HTTPS for any API URL exposed on the public internet.
            </li>
            <li>
              Protect your recovery phrase and API keys. We cannot recover them
              for you.
            </li>
            <li>
              Uninstalling the mobile app removes local settings; server-side
              wallet data remains on whichever API host you used until you
              delete it there.
            </li>
          </ul>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">
            8. Children
          </h2>
          <p>
            NozyWallet is not directed at children under 13 (or the minimum age
            in your jurisdiction). We do not knowingly collect personal
            information from children.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">
            9. Your choices
          </h2>
          <ul className="list-disc pl-6 space-y-2">
            <li>
              Run your own API and Zebrad for maximum control.
            </li>
            <li>
              Clear mobile connection settings in the app or uninstall the app.
            </li>
            <li>
              Use a new wallet profile or server if you no longer trust a host.
            </li>
          </ul>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">
            10. Changes to this policy
          </h2>
          <p>
            We may update this page when the product or law changes. The
            &quot;Last updated&quot; date at the top will change. Continued use
            after an update means you accept the revised policy.
          </p>
        </section>

        <section>
          <h2 className="text-2xl font-semibold text-zinc-900 mb-4">
            11. More detail
          </h2>
          <p>
            Technical privacy architecture:{" "}
            <a
              href="https://leonine-dao.github.io/Nozy-wallet/book/nozy/privacy-model.html"
              className="text-yellow-700 hover:underline"
              target="_blank"
              rel="noopener noreferrer"
            >
              NozyWallet privacy model (documentation)
            </a>
          </p>
        </section>
      </div>
    </div>
  );
};

export default Privacy;
