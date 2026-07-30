import { Link } from "react-router-dom";

const Footer = () => {
  return (
    <footer className="border-t border-[rgba(245,240,230,0.12)] pt-16 pb-10">
      <div className="max-w-7xl mx-auto px-6">
        <div className="flex flex-col md:flex-row justify-between gap-10 mb-12">
          <div>
            <h3 className="font-display text-2xl font-bold text-[#f5f0e6] mb-2">NozyWallet</h3>
            <p className="text-[#a39a88] max-w-sm leading-relaxed">
              Shielded-first Zcash. Orchard · Ironwood · your keys.
            </p>
            <a
              href="mailto:support.team@nozywallet.com"
              className="inline-block mt-4 text-sm text-[#c8ccd4] hover:underline"
            >
              support.team@nozywallet.com
            </a>
          </div>

          <div className="flex flex-wrap gap-x-8 gap-y-3 text-sm">
            <a
              href="https://leonine-dao.github.io/Nozy-wallet/book/"
              target="_blank"
              rel="noopener noreferrer"
              className="text-[#a39a88] hover:text-[#c8ccd4] transition-colors"
            >
              Documentation
            </a>
            <a
              href="https://github.com/LEONINE-DAO/Nozy-wallet"
              target="_blank"
              rel="noopener noreferrer"
              className="text-[#a39a88] hover:text-[#c8ccd4] transition-colors"
            >
              GitHub
            </a>
            <a
              href="/whitepaper/"
              className="text-[#a39a88] hover:text-[#c8ccd4] transition-colors"
            >
              White paper
            </a>
            <Link to="/mobile" className="text-[#a39a88] hover:text-[#c8ccd4] transition-colors">
              Mobile
            </Link>
            <Link to="/ironwood" className="text-[#a39a88] hover:text-[#c8ccd4] transition-colors">
              Ironwood
            </Link>
            <Link to="/privacy" className="text-[#a39a88] hover:text-[#c8ccd4] transition-colors">
              Privacy
            </Link>
            <Link to="/security" className="text-[#a39a88] hover:text-[#c8ccd4] transition-colors">
              Security
            </Link>
          </div>
        </div>

        <div className="border-t border-[rgba(245,240,230,0.12)] pt-8 text-sm text-[#a39a88]">
          <p>&copy; {new Date().getFullYear()} NozyWallet · LEONINE DAO</p>
        </div>
      </div>
    </footer>
  );
};

export default Footer;
