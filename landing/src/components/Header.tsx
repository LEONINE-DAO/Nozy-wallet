import { useState, useEffect } from "react";
import { HamburgerMenu, CloseSquare } from "@solar-icons/react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import BrandLogo from "./BrandLogo";

const Header = () => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [scrolled, setScrolled] = useState(false);
  const location = useLocation();
  const navigate = useNavigate();

  useEffect(() => {
    document.body.style.overflow = isMenuOpen ? "hidden" : "unset";
  }, [isMenuOpen]);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 12);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  const toggleMenu = () => setIsMenuOpen(!isMenuOpen);

  const handleNavClick = (e: React.MouseEvent, id: string) => {
    e.preventDefault();
    setIsMenuOpen(false);

    if (location.pathname === "/") {
      const element = document.getElementById(id);
      if (element) {
        element.scrollIntoView({ behavior: "smooth" });
        const base = (import.meta.env.BASE_URL || "/").replace(/\/$/, "");
        window.history.pushState(null, "", `${base}/#${id}`);
      }
    } else {
      navigate("/", { state: { scrollTo: id } });
    }
  };

  const linkClass =
    "text-[#a39a88] hover:text-[#c8ccd4] transition-colors cursor-pointer";

  return (
    <header
      className={`fixed top-0 w-full z-50 border-b transition-colors duration-300 ${
        isMenuOpen || scrolled
          ? "bg-[#0c0b09]/95 backdrop-blur-md border-[rgba(245,240,230,0.12)]"
          : "bg-transparent border-transparent"
      }`}
    >
      <div className="max-w-7xl mx-auto px-4 sm:px-6 h-20 flex items-center justify-between relative z-50">
        <Link to="/" className="flex items-center shrink-0">
          <BrandLogo
            markClassName="h-9 w-9 sm:h-10 sm:w-10"
            className="text-[1.15rem] sm:text-[1.35rem]"
          />
        </Link>

        <nav className="hidden md:flex items-center gap-7 text-sm font-medium">
          <a href="#products" onClick={(e) => handleNavClick(e, "products")} className={linkClass}>
            Products
          </a>
          <Link to="/mobile" className={linkClass}>
            Mobile
          </Link>
          <Link to="/ironwood" className={linkClass}>
            Ironwood
          </Link>
          <a href="#features" onClick={(e) => handleNavClick(e, "features")} className={linkClass}>
            Features
          </a>
          <a href="#faq" onClick={(e) => handleNavClick(e, "faq")} className={linkClass}>
            FAQ
          </a>
          <a
            href="https://leonine-dao.github.io/Nozy-wallet/book/"
            target="_blank"
            rel="noopener noreferrer"
            className={linkClass}
          >
            Docs
          </a>
          <Link
            to={{ pathname: "/", hash: "download" }}
            className="nw-btn nw-btn-primary !py-2 !px-4 !text-sm"
          >
            Download
          </Link>
        </nav>

        <button
          className="md:hidden text-[#f5f0e6] p-2"
          onClick={toggleMenu}
          aria-label="Toggle menu"
        >
          {isMenuOpen ? <CloseSquare size={24} /> : <HamburgerMenu size={24} />}
        </button>
      </div>

      <div
        className={`fixed left-0 right-0 top-20 bottom-0 bg-[#0c0b09] z-40 transition-all duration-300 md:hidden ${
          isMenuOpen ? "opacity-100 visible" : "opacity-0 invisible pointer-events-none"
        }`}
      >
        <nav className="flex flex-col h-full px-6 py-8">
          <div className="flex flex-col gap-1">
            {[
              { label: "Products", id: "products" },
              { label: "Features", id: "features" },
              { label: "FAQ", id: "faq" },
              { label: "About", id: "about" },
            ].map((item) => (
              <a
                key={item.id}
                href={`#${item.id}`}
                onClick={(e) => handleNavClick(e, item.id)}
                className="font-display text-xl font-semibold text-[#f5f0e6] py-3 border-b border-[rgba(245,240,230,0.12)]"
              >
                {item.label}
              </a>
            ))}
            <Link
              to="/mobile"
              onClick={() => setIsMenuOpen(false)}
              className="font-display text-xl font-semibold text-[#f5f0e6] py-3 border-b border-[rgba(245,240,230,0.12)]"
            >
              Mobile
            </Link>
            <Link
              to="/ironwood"
              onClick={() => setIsMenuOpen(false)}
              className="font-display text-xl font-semibold text-[#f5f0e6] py-3 border-b border-[rgba(245,240,230,0.12)]"
            >
              Ironwood
            </Link>
          </div>
          <div className="mt-8">
            <Link
              to={{ pathname: "/", hash: "download" }}
              onClick={() => setIsMenuOpen(false)}
              className="nw-btn nw-btn-primary w-full"
            >
              Download NozyWallet
            </Link>
          </div>
        </nav>
      </div>
    </header>
  );
};

export default Header;
