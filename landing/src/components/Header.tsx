import { useState, useEffect } from "react";
import { HamburgerMenu, CloseSquare } from "@solar-icons/react";
import { Link, useLocation, useNavigate } from "react-router-dom";

const Header = () => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const location = useLocation();
  const navigate = useNavigate();

  useEffect(() => {
    if (isMenuOpen) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "unset";
    }
  }, [isMenuOpen]);

  const toggleMenu = () => setIsMenuOpen(!isMenuOpen);

  const handleNavClick = (e: React.MouseEvent, id: string) => {
    e.preventDefault();
    setIsMenuOpen(false);
    
    if (location.pathname === "/") {
      const element = document.getElementById(id);
      if (element) {
        element.scrollIntoView({ behavior: "smooth" });
        window.history.pushState(null, "", `/#${id}`);
      }
    } else {
      navigate("/", { state: { scrollTo: id } });
    }
  };

  return (
    <header
      className={`fixed top-0 w-full z-50 border-b border-zinc-200/50 ${
        isMenuOpen ? "bg-white" : "backdrop-blur-md bg-white/70"
      }`}
    >
      <div className="max-w-7xl mx-auto px-6 h-24 flex items-center justify-between relative z-50">
        <Link
          to="/"
          className="flex items-center gap-3"
        >
          <img
            src={`${import.meta.env.BASE_URL}logo.png`}
            alt="NozyWallet Logo"
            className="h-auto w-32"
          />
        </Link>

        <nav className="hidden md:flex items-center gap-8 text-sm font-medium text-zinc-500">
          <a
            href="#features"
            onClick={(e) => handleNavClick(e, "features")}
            className="hover:text-yellow-600 transition-colors cursor-pointer"
          >
            Features
          </a>
          <a
            href="#faq"
            onClick={(e) => handleNavClick(e, "faq")}
            className="hover:text-yellow-600 transition-colors cursor-pointer"
          >
            FAQ
          </a>
          <a
            href="#about"
            onClick={(e) => handleNavClick(e, "about")}
            className="hover:text-yellow-600 transition-colors cursor-pointer"
          >
            About
          </a>
          <a
            href="https://leonine-dao.github.io/Nozy-wallet/book/"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-yellow-600 transition-colors cursor-pointer"
          >
            Documentation
          </a>
          <a
            href="https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest"
            target="_blank"
            rel="noopener noreferrer"
            className="bg-zinc-900 hover:bg-zinc-800 text-white px-5 py-2 rounded-full transition-all border border-transparent shadow-md hover:shadow-lg"
          >
            Download
          </a>
        </nav>

        <button
          className="md:hidden text-zinc-900 p-2"
          onClick={toggleMenu}
          aria-label="Toggle menu"
        >
          {isMenuOpen ? <CloseSquare size={24} /> : <HamburgerMenu size={24} />}
        </button>
      </div>

      <div
        className={`fixed left-0 right-0 top-20 bottom-0 bg-white z-40 transition-all duration-300 md:hidden ${
          isMenuOpen
            ? "opacity-100 visible"
            : "opacity-0 invisible pointer-events-none"
        }`}
      >
        <nav className="flex flex-col h-full px-6 py-8">
          <div className="flex flex-col gap-6">
            <a
              href="#features"
              onClick={(e) => handleNavClick(e, "features")}
              className="text-lg font-semibold text-zinc-800 hover:text-yellow-600 transition-colors py-3 border-b border-zinc-100"
            >
              Features
            </a>
            <a
              href="#faq"
              onClick={(e) => handleNavClick(e, "faq")}
              className="text-lg font-semibold text-zinc-800 hover:text-yellow-600 transition-colors py-3 border-b border-zinc-100"
            >
              FAQ
            </a>
            <a
              href="#about"
              onClick={(e) => handleNavClick(e, "about")}
              className="text-lg font-semibold text-zinc-800 hover:text-yellow-600 transition-colors py-3 border-b border-zinc-100"
            >
              About
            </a>
            <a
              href="https://leonine-dao.github.io/Nozy-wallet/book/"
              target="_blank"
              rel="noopener noreferrer"
              className="text-lg font-semibold text-zinc-800 hover:text-yellow-600 transition-colors py-3 border-b border-zinc-100"
            >
              Documentation
            </a>
          </div>
          <div className="mt-8">
            <a
              href="https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest"
              target="_blank"
              rel="noopener noreferrer"
              className="w-full bg-zinc-900 hover:bg-zinc-800 text-white px-6 py-3 rounded-full text-base font-medium transition-all shadow-lg active:scale-95 block text-center"
            >
              Download NozyWallet
            </a>
          </div>
        </nav>
      </div>
    </header>
  );
};

export default Header;
