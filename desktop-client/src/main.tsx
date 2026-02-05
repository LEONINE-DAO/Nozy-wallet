import ReactDOM from "react-dom/client";
import App from "./App.tsx";
import "./index.css";
import { QueryClientProvider } from "@tanstack/react-query";
import { queryClient } from "./lib/queryClient.ts";

// Fix React Refresh issue - ensure root element exists
const rootElement = document.getElementById("root");
if (!rootElement) {
  throw new Error("Root element not found");
}

// Create root once and reuse
const root = ReactDOM.createRoot(rootElement);

// Render without StrictMode temporarily to fix React Refresh
root.render(
  <QueryClientProvider client={queryClient}>
    <App />
  </QueryClientProvider>
);
