import { Link } from "react-router-dom";

const Footer = () => {
  return (
    <footer className="border-t border-zinc-200 bg-white pt-16 pb-8">
      <div className="max-w-7xl mx-auto px-6">
        <div className="flex flex-col md:flex-row justify-between md:items-center gap-8 mb-12">
          <div className="md:text-left">
            <h3 className="text-2xl font-bold text-zinc-900 mb-2">
              NozyWallet
            </h3>
            <p className="text-zinc-500 italic">
              "Privacy is a right, not a privilege."
            </p>
          </div>

          <div className="flex flex-col md:flex-row text-sm md:items-center gap-4 md:gap-8">
            <a
              href="#"
              target="_blank"
              className="text-zinc-500 hover:text-yellow-600 transition-colors"
            >
              Documentation
            </a>
            <a
              href="https://github.com/LEONINE-DAO/Nozy-wallet"
              target="_blank"
              className="text-zinc-500 hover:text-yellow-600 transition-colors"
            >
              GitHub
            </a>
            <Link
              to="/privacy"
              className="text-zinc-500 hover:text-yellow-600 transition-colors"
            >
              Privacy Policy
            </Link>
            <Link
              to="/security"
              className="text-zinc-500 hover:text-yellow-600 transition-colors"
            >
              Security
            </Link>
          </div>
        </div>

        <div className="border-t border-zinc-100 pt-8 flex flex-col items-center gap-4 text-sm text-zinc-400">
          <p>
            &copy; {new Date().getFullYear()} NozyWallet. All rules reserved.
          </p>
        </div>
      </div>
    </footer>
  );
};

export default Footer;
