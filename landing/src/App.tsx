import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import Header from "./components/Header";
import Footer from "./components/Footer";
import Home from "./pages/Home";
import Privacy from "./pages/Privacy";
import Security from "./pages/Security";
import { Analytics } from "@vercel/analytics/react";

function App() {
  return (
    <>
      <Analytics />
      <Router>
        <div className="min-h-screen bg-white text-zinc-900 selection:bg-yellow-200 selection:text-yellow-900">
          <Header />
          <main>
            <Routes>
              <Route
                path="/"
                element={<Home />}
              />
              <Route
                path="/privacy"
                element={<Privacy />}
              />
              <Route
                path="/security"
                element={<Security />}
              />
            </Routes>
          </main>
          <Footer />
        </div>
      </Router>
    </>
  );
}

export default App;
