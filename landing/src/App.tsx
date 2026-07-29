import { useState } from "react";
import { BrowserRouter as Router, Routes, Route, useLocation } from "react-router-dom";
import Header from "./components/Header";
import Footer from "./components/Footer";
import Starfield from "./components/Starfield";
import PageLoader from "./components/PageLoader";
import Home from "./pages/Home";
import Privacy from "./pages/Privacy";
import Security from "./pages/Security";
import Mobile from "./pages/Mobile";
import Ironwood from "./pages/Ironwood";
import { Analytics } from "@vercel/analytics/react";

/** Matches vite.config.ts `base` (GitHub Pages: /Nozy-wallet/) */
const routerBasename = import.meta.env.BASE_URL.replace(/\/$/, "") || "/";

function AppShell() {
  const location = useLocation();
  const isIronwoodDashboard = location.pathname === "/ironwood";
  const [ready, setReady] = useState(false);

  return (
    <>
        {!isIronwoodDashboard ? (
          <PageLoader
            onDone={() => {
              setReady(true);
            }}
          />
        ) : null}
        {!isIronwoodDashboard ? <Starfield /> : null}

      <div
        className={
          isIronwoodDashboard
            ? "min-h-screen bg-[#0a0a0a] text-zinc-100"
            : `relative z-10 min-h-screen bg-transparent text-[#f5f0e6] selection:bg-[#c8ccd4]/30 selection:text-[#f5f0e6] transition-opacity duration-700 ${
                ready ? "opacity-100" : "opacity-0"
              }`
        }
      >
        {!isIronwoodDashboard ? <Header /> : null}
        <main>
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/privacy" element={<Privacy />} />
            <Route path="/security" element={<Security />} />
            <Route path="/mobile" element={<Mobile />} />
            <Route path="/ironwood" element={<Ironwood />} />
          </Routes>
        </main>
        {!isIronwoodDashboard ? <Footer /> : null}
      </div>
    </>
  );
}

function App() {
  return (
    <>
      <Analytics />
      <Router basename={routerBasename}>
        <AppShell />
      </Router>
    </>
  );
}

export default App;
