import { useState, useRef, useEffect } from "react";
import {
  ArrowLeft,
  ArrowRight,
  Refresh,
  Home,
  Lock,
  Bookmark,
  Shield,
  History as HistoryIcon,
  CloseCircle,
} from "@solar-icons/react";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { TransactionApprovalDialog } from "../components/TransactionApprovalDialog";
import { MessageSigningDialog } from "../components/MessageSigningDialog";
import { BrowserTab } from "../components/BrowserTab";
import toast from "react-hot-toast";
import { useWalletStore } from "../store/walletStore";
import { walletApi } from "../lib/api";
import { logger } from "../utils/logger";
import { useKeyboardShortcuts } from "../utils/keyboardShortcuts";

interface BrowserPageProps {
  onBack?: () => void;
}

interface PendingRequest {
  type: "transaction" | "message";
  method: string;
  params: any[];
  requestId: string;
  origin: string;
}

interface Bookmark {
  name: string;
  url: string;
}

interface HistoryEntry {
  url: string;
  title: string;
  timestamp: number;
}

// Trusted sites that don't need warnings
const TRUSTED_SITES = [
  "zfnd.org",
  "zcashcommunity.com",
  "zcashblockexplorer.com",
  "z.cash",
];

// Known phishing patterns and malicious sites
const PHISHING_PATTERNS = [
  /zcash.*wallet.*login/i,
  /zcash.*verify.*account/i,
  /zcash.*recover.*funds/i,
  /zcash.*suspended/i,
  /zcash.*security.*alert/i,
];

const MALICIOUS_DOMAINS = [
  "zcash-wallet.com",
  "zcash-wallet.org",
  "zcash-recovery.com",
  "zcash-support.com",
];

interface Tab {
  id: string;
  url: string;
  title: string;
  isLoading: boolean;
}

// Rate limiting for Web3 requests
class RateLimiter {
  private requests: Map<string, number[]> = new Map();
  private maxRequests: number;
  private windowMs: number;

  constructor(maxRequests: number = 10, windowMs: number = 60000) {
    this.maxRequests = maxRequests;
    this.windowMs = windowMs;
  }

  isAllowed(origin: string): boolean {
    const now = Date.now();
    const requests = this.requests.get(origin) || [];
    const recentRequests = requests.filter(time => now - time < this.windowMs);
    
    if (recentRequests.length >= this.maxRequests) {
      return false;
    }
    
    recentRequests.push(now);
    this.requests.set(origin, recentRequests);
    return true;
  }

  reset(origin: string) {
    this.requests.delete(origin);
  }
}

export function BrowserPage({ onBack: _onBack }: BrowserPageProps) {
  const [url, setUrl] = useState("https://");
  const [tabs, setTabs] = useState<Tab[]>([{ id: "1", url: "", title: "New Tab", isLoading: false }]);
  const [activeTabId, setActiveTabId] = useState("1");
  const [canGoBack, setCanGoBack] = useState(false);
  const [canGoForward, _setCanGoForward] = useState(false);
  const [isLoading, _setIsLoading] = useState(false);
  const [bookmarks, setBookmarks] = useState<Bookmark[]>([]);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [isBookmarked, setIsBookmarked] = useState(false);
  const [showBookmarks, setShowBookmarks] = useState(false);
  const [showHistory, setShowHistory] = useState(false);
  const [_pendingRequest, _setPendingRequest] = useState<PendingRequest | null>(null);
  const [pendingTransaction, setPendingTransaction] = useState<any>(null);
  const [pendingMessage, setPendingMessage] = useState<string>("");
  const [pendingOrigin, setPendingOrigin] = useState<string>("");
  const [showSecurityWarning, setShowSecurityWarning] = useState(false);
  const [_phishingWarning, setPhishingWarning] = useState<string | null>(null);
  const [sitePermissions, setSitePermissions] = useState<Record<string, boolean>>({});
  const [transactionTimeout, setTransactionTimeout] = useState<ReturnType<typeof setTimeout> | null>(null);
  const rateLimiter = useRef(new RateLimiter(10, 60000)); // 10 requests per minute
  const pendingRequestRef = useRef<{ resolve: (value: any) => void; reject: (error: Error) => void } | null>(null);
  const iframeRefs = useRef<Map<string, HTMLIFrameElement>>(new Map());
  const addressBarRef = useRef<HTMLInputElement>(null);
  const { address } = useWalletStore();

  const activeTab = tabs.find(t => t.id === activeTabId) || tabs[0];
  const currentUrl = activeTab?.url || "";

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = useKeyboardShortcuts({
      newTab: createNewTab,
      closeTab: () => closeTab(activeTabId),
      refresh: handleRefresh,
      focusAddressBar: () => {
        addressBarRef.current?.focus();
      },
      goBack: handleBack,
      goForward: handleForward,
    }, true);

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [activeTabId]);

  // Load bookmarks, history, and permissions from localStorage
  useEffect(() => {
    const savedBookmarks = localStorage.getItem("nozy-browser-bookmarks");
    if (savedBookmarks) {
      try {
        setBookmarks(JSON.parse(savedBookmarks));
      } catch (e) {
        console.error("Failed to load bookmarks:", e);
      }
    }
    const savedHistory = localStorage.getItem("nozy-browser-history");
    if (savedHistory) {
      try {
        setHistory(JSON.parse(savedHistory));
      } catch (e) {
        console.error("Failed to load history:", e);
      }
    }
    const savedPermissions = localStorage.getItem("nozy-browser-permissions");
    if (savedPermissions) {
      try {
        setSitePermissions(JSON.parse(savedPermissions));
      } catch (e) {
        console.error("Failed to load permissions:", e);
      }
    }
  }, []);

  // Save bookmarks, history, and permissions to localStorage
  useEffect(() => {
    if (bookmarks.length > 0) {
      localStorage.setItem("nozy-browser-bookmarks", JSON.stringify(bookmarks));
    }
  }, [bookmarks]);

  useEffect(() => {
    if (history.length > 0) {
      // Keep only last 100 entries
      const recentHistory = history.slice(-100);
      localStorage.setItem("nozy-browser-history", JSON.stringify(recentHistory));
    }
  }, [history]);

  useEffect(() => {
    if (Object.keys(sitePermissions).length > 0) {
      localStorage.setItem("nozy-browser-permissions", JSON.stringify(sitePermissions));
    }
  }, [sitePermissions]);

  // Check if current URL is bookmarked
  useEffect(() => {
    if (currentUrl) {
      const bookmarked = bookmarks.some(b => b.url === currentUrl);
      setIsBookmarked(bookmarked);
      
      // Add to history
      try {
        const urlObj = new URL(currentUrl);
        const hostname = urlObj.hostname;
        const newEntry: HistoryEntry = {
          url: currentUrl,
          title: hostname,
          timestamp: Date.now(),
        };
        setHistory(prev => {
          // Remove duplicate and add to end
          const filtered = prev.filter(h => h.url !== currentUrl);
          return [...filtered, newEntry];
        });
      } catch (e) {
        // Invalid URL, skip history
      }
    }
  }, [currentUrl, bookmarks]);

  // Check if site needs security warning
  const checkSecurityWarning = (url: string): boolean => {
    try {
      const urlObj = new URL(url);
      const hostname = urlObj.hostname.replace(/^www\./, "");
      return !TRUSTED_SITES.some(trusted => hostname.includes(trusted));
    } catch {
      return true; // Show warning for invalid URLs
    }
  };

  // Check for phishing patterns
  const checkPhishing = (url: string): string | null => {
    try {
      const urlObj = new URL(url);
      const hostname = urlObj.hostname.toLowerCase();
      const pathname = urlObj.pathname.toLowerCase();
      const fullUrl = (hostname + pathname).toLowerCase();

      // Check against malicious domains
      if (MALICIOUS_DOMAINS.some(domain => hostname.includes(domain))) {
        return "This domain is known to be malicious. Do not enter your wallet password or private keys.";
      }

      // Check for phishing patterns in URL
      for (const pattern of PHISHING_PATTERNS) {
        if (pattern.test(fullUrl)) {
          return "This URL matches known phishing patterns. Be extremely cautious.";
        }
      }

      return null;
    } catch {
      return null;
    }
  };

  // Search functionality
  const performSearch = (query: string): string => {
    // Check if it's already a URL
    if (query.match(/^https?:\/\//)) {
      return query;
    }

    // Check if it's a domain (contains .)
    if (query.includes(".") && !query.includes(" ")) {
      return `https://${query}`;
    }

    // Otherwise, use DuckDuckGo search
    return `https://duckduckgo.com/?q=${encodeURIComponent(query)}`;
  };

  // Tab management functions
  const createNewTab = () => {
    const newTabId = Date.now().toString();
    const newTab: Tab = {
      id: newTabId,
      url: "",
      title: "New Tab",
      isLoading: false,
    };
    setTabs([...tabs, newTab]);
    setActiveTabId(newTabId);
    setUrl("https://");
  };

  const closeTab = (tabId: string) => {
    if (tabs.length === 1) {
      // Don't close the last tab
      return;
    }
    const newTabs = tabs.filter(t => t.id !== tabId);
    setTabs(newTabs);
    if (activeTabId === tabId) {
      setActiveTabId(newTabs[newTabs.length - 1].id);
    }
  };

  const selectTab = (tabId: string) => {
    setActiveTabId(tabId);
    const tab = tabs.find(t => t.id === tabId);
    if (tab) {
      setUrl(tab.url || "https://");
    }
  };

  const updateTab = (tabId: string, updates: Partial<Tab>) => {
    setTabs(tabs.map(t => t.id === tabId ? { ...t, ...updates } : t));
  };

  // Inject Web3 provider when iframe loads
  useEffect(() => {
    const iframe = iframeRefs.current.get(activeTabId);
    if (!iframe || !currentUrl) return;

    // Get origin from URL
    const getOrigin = (url: string): string => {
      try {
        const urlObj = new URL(url);
        return urlObj.origin;
      } catch {
        return url;
      }
    };

    const origin = getOrigin(currentUrl);

    // Create a Web3 provider compatible with EIP-1193
    const createWeb3Provider = () => ({
      isNozyWallet: true,
      isConnected: () => !!address,
      request: async (args: { method: string; params?: any[] }): Promise<any> => {
        console.log("Web3 request:", args);

        switch (args.method) {
          case "eth_requestAccounts":
          case "zcash_requestAccounts":
            if (!address) {
              throw new Error("Wallet not connected");
            }
            // Check if site has permission
            if (!sitePermissions[origin]) {
              // Request permission
              const granted = await new Promise<boolean>((resolve) => {
                toast((t) => (
                  <div className="flex items-center gap-3">
                    <span className="text-sm">
                      {origin} wants to connect to your wallet
                    </span>
                    <div className="flex gap-2">
                      <button
                        onClick={() => {
                          resolve(false);
                          toast.dismiss(t.id);
                        }}
                        className="px-3 py-1 bg-gray-200 dark:bg-gray-700 rounded text-sm"
                      >
                        Deny
                      </button>
                      <button
                        onClick={() => {
                          resolve(true);
                          toast.dismiss(t.id);
                        }}
                        className="px-3 py-1 bg-primary-600 text-white rounded text-sm"
                      >
                        Allow
                      </button>
                    </div>
                  </div>
                ), { duration: 10000 });
              });
              
              if (granted) {
                setSitePermissions(prev => ({ ...prev, [origin]: true }));
              } else {
                throw new Error("User denied connection");
              }
            }
            return [address];

          case "eth_accounts":
          case "zcash_accounts":
            return address ? [address] : [];

          case "eth_chainId":
          case "zcash_chainId":
            return "0x5ba3"; // Zcash mainnet chain ID

          case "eth_sendTransaction":
          case "zcash_sendTransaction":
            // Rate limiting check
            if (!rateLimiter.current.isAllowed(origin)) {
              throw new Error("Too many requests. Please wait a moment.");
            }

            // Show transaction approval dialog
            const tx = args.params?.[0];
            if (!tx) {
              throw new Error("Invalid transaction parameters");
            }
            
            return new Promise((resolve, reject) => {
              setPendingTransaction(tx);
              setPendingOrigin(origin);
              pendingRequestRef.current = { resolve, reject };
              
              // Set timeout for transaction approval (5 minutes)
              const timeout = setTimeout(() => {
                if (pendingRequestRef.current) {
                  pendingRequestRef.current.reject(new Error("Transaction request timed out"));
                  setPendingTransaction(null);
                  setPendingOrigin("");
                  pendingRequestRef.current = null;
                }
              }, 5 * 60 * 1000);
              
              setTransactionTimeout(timeout);
            });

          case "personal_sign":
          case "zcash_signMessage":
            // Show message signing dialog
            const message = args.params?.[0];
            if (!message) {
              throw new Error("Invalid message parameters");
            }
            
            return new Promise((resolve, reject) => {
              setPendingMessage(message);
              setPendingOrigin(origin);
              pendingRequestRef.current = { resolve, reject };
            });

          default:
            throw new Error(`Unsupported method: ${args.method}`);
        }
      },
      on: (event: string, _callback: Function) => {
        console.log("Web3 event listener:", event);
      },
      removeListener: (event: string, _callback: Function) => {
        console.log("Web3 remove listener:", event);
      },
    });

    const injectWeb3Provider = () => {
      try {
        const iframeWindow = iframe.contentWindow;
        if (!iframeWindow) return;

        const web3Provider = createWeb3Provider();

        // For cross-origin iframes, use postMessage
        iframeWindow.postMessage(
          {
            type: "NOZY_WALLET_PROVIDER_INJECT",
            provider: {
              isNozyWallet: true,
              address: address || null,
            },
          },
          "*"
        );

        // Try direct injection (works for same-origin)
        try {
          (iframeWindow as any).ethereum = web3Provider;
          (iframeWindow as any).zcash = web3Provider;
          (iframeWindow as any).nozy = web3Provider;
        } catch (e) {
          console.log("Direct injection blocked (cross-origin), using postMessage");
        }
      } catch (error) {
        logger.error("Error injecting Web3 provider", error as Error);
      }
    };

    // Listen for messages from iframe (for cross-origin communication)
    const handleMessage = (event: MessageEvent) => {
      // Verify message is from the current iframe
      if (event.source !== iframe.contentWindow) return;
      
      // Additional security: Verify origin matches current URL (if possible)
      try {
        const currentOrigin = new URL(currentUrl).origin;
        // Note: event.origin may be unavailable for cross-origin, but we check source
        if (event.origin && event.origin !== currentOrigin && event.origin !== '*') {
          console.warn('Message from unexpected origin:', event.origin);
          return;
        }
      } catch (e) {
        // Invalid URL, but source check is primary security
      }

      if (event.data?.type === "NOZY_WALLET_REQUEST") {
        const { method, params, requestId } = event.data;
        const web3Provider = createWeb3Provider();
        
        // Handle the request
        web3Provider.request({ method, params })
          .then((result) => {
            iframe.contentWindow?.postMessage(
              {
                type: "NOZY_WALLET_RESPONSE",
                requestId,
                result,
              },
              "*"
            );
          })
          .catch((error) => {
            iframe.contentWindow?.postMessage(
              {
                type: "NOZY_WALLET_RESPONSE",
                requestId,
                error: error.message,
              },
              "*"
            );
          });
      }

      if (event.data?.type === "NOZY_WALLET_REQUEST_PROVIDER") {
        // Send provider info
        iframe.contentWindow?.postMessage(
          {
            type: "NOZY_WALLET_PROVIDER_INJECT",
            provider: {
              isNozyWallet: true,
              address: address || null,
            },
          },
          "*"
        );
      }
    };

    window.addEventListener("message", handleMessage);

    // Inject after iframe loads
    iframe.addEventListener("load", injectWeb3Provider);
    
    return () => {
      iframe.removeEventListener("load", injectWeb3Provider);
      window.removeEventListener("message", handleMessage);
    };
  }, [currentUrl, address, sitePermissions]);

  const handleNavigate = (targetUrl: string) => {
    let finalUrl = targetUrl.trim();
    
    // Perform search if needed
    finalUrl = performSearch(finalUrl);
    
    // Add https:// if no protocol
    if (!finalUrl.match(/^https?:\/\//)) {
      finalUrl = `https://${finalUrl}`;
    }

    // Validate URL
    try {
      new URL(finalUrl);
      
      // Check for phishing
      const phishingWarning = checkPhishing(finalUrl);
      if (phishingWarning) {
        setPhishingWarning(phishingWarning);
        // Still allow navigation but show warning
      }
      
      // Check security warning
      if (checkSecurityWarning(finalUrl)) {
        setShowSecurityWarning(true);
        // Store the URL to navigate after warning
        const proceed = () => {
          updateTab(activeTabId, { url: finalUrl, isLoading: true });
          setUrl(finalUrl);
          setShowSecurityWarning(false);
        };
        // For now, auto-proceed after 2 seconds or user can click
        setTimeout(proceed, 2000);
      } else {
        updateTab(activeTabId, { url: finalUrl, isLoading: true });
        setUrl(finalUrl);
      }
    } catch (e) {
      logger.warn("Invalid URL attempted", { url: targetUrl });
      toast.error("Invalid URL");
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    handleNavigate(url);
  };

  const handleBack = () => {
    const iframe = iframeRefs.current.get(activeTabId);
    if (iframe?.contentWindow) {
      iframe.contentWindow.history.back();
    }
  };

  const handleForward = () => {
    const iframe = iframeRefs.current.get(activeTabId);
    if (iframe?.contentWindow) {
      iframe.contentWindow.history.forward();
    }
  };

  const handleRefresh = () => {
    const iframe = iframeRefs.current.get(activeTabId);
    if (iframe) {
      iframe.src = iframe.src;
      updateTab(activeTabId, { isLoading: true });
    }
  };

  const handleHome = () => {
    setUrl("https://");
    updateTab(activeTabId, { url: "" });
  };

  const handleIframeLoad = () => {
    updateTab(activeTabId, { isLoading: false });
    try {
      const iframe = iframeRefs.current.get(activeTabId);
      const iframeWindow = iframe?.contentWindow;
      if (iframeWindow) {
        setCanGoBack(iframeWindow.history.length > 1);
        // Update tab title from page
        try {
          const title = iframeWindow.document.title || new URL(currentUrl).hostname;
          updateTab(activeTabId, { title });
        } catch (e) {
          // Cross-origin restrictions
        }
      }
    } catch (e) {
      // Cross-origin restrictions
    }
  };

  const handleToggleBookmark = () => {
    if (!currentUrl) return;

    if (isBookmarked) {
      // Remove bookmark
      setBookmarks(prev => prev.filter(b => b.url !== currentUrl));
      toast.success("Bookmark removed");
    } else {
      // Add bookmark
      try {
        const urlObj = new URL(currentUrl);
        const name = urlObj.hostname.replace(/^www\./, "");
        setBookmarks(prev => [...prev, { name, url: currentUrl }]);
        toast.success("Bookmark added");
      } catch (e) {
        toast.error("Failed to add bookmark");
      }
    }
  };

  const handleApproveTransaction = async () => {
    if (!pendingTransaction || !address) {
      toast.error("Invalid transaction");
      return;
    }

    try {
      // Convert transaction to send format
      // Zcash uses zatoshis (1 ZEC = 10^8 zatoshis), similar to satoshis
      const parseValue = (value: string): number => {
        try {
          const hexValue = value.startsWith("0x") ? value.slice(2) : value;
          const zatoshis = parseInt(hexValue, 16);
          return zatoshis / 100_000_000; // Convert zatoshis to ZEC
        } catch {
          return 0;
        }
      };
      
      const amount = parseValue(pendingTransaction.value || "0x0");
      const recipient = pendingTransaction.to;

      if (!recipient) {
        throw new Error("Invalid recipient address");
      }

      // Send transaction
      const result = await walletApi.sendTransaction({
        recipient,
        amount,
        memo: pendingTransaction.data || undefined,
      });

      if (result.data.success && result.data.txid) {
        toast.success("Transaction sent successfully!");
        if (pendingRequestRef.current) {
          pendingRequestRef.current.resolve(result.data.txid);
        }
      } else {
        throw new Error(result.data.message || "Transaction failed");
      }
    } catch (error: any) {
      toast.error(error.message || "Failed to send transaction");
      if (pendingRequestRef.current) {
        pendingRequestRef.current.reject(new Error(error.message || "Transaction failed"));
      }
    } finally {
      setPendingTransaction(null);
      setPendingOrigin("");
      pendingRequestRef.current = null;
    }
  };

  const handleRejectTransaction = () => {
    // Clear timeout
    if (transactionTimeout) {
      clearTimeout(transactionTimeout);
      setTransactionTimeout(null);
    }

    if (pendingRequestRef.current) {
      pendingRequestRef.current.reject(new Error("User rejected transaction"));
    }
    setPendingTransaction(null);
    setPendingOrigin("");
    pendingRequestRef.current = null;
    toast("Transaction rejected");
  };

  const handleApproveMessage = async (password: string) => {
    if (!pendingMessage || !address || !password) {
      toast.error("Invalid message or password");
      return;
    }

    try {
      // Decode message if it's hex
      const decodeMessage = (msg: string): string => {
        try {
          if (msg.startsWith("0x")) {
            const hex = msg.slice(2);
            let decoded = "";
            for (let i = 0; i < hex.length; i += 2) {
              const charCode = parseInt(hex.substr(i, 2), 16);
              if (charCode === 0) break;
              decoded += String.fromCharCode(charCode);
            }
            return decoded || msg;
          }
          return msg;
        } catch {
          return msg;
        }
      };

      const messageToSign = decodeMessage(pendingMessage);
      
      // Sign the message using the wallet API
      const response = await walletApi.signMessage({
        message: messageToSign,
        password: password,
      });

      toast.success("Message signed successfully");
      
      if (pendingRequestRef.current) {
        pendingRequestRef.current.resolve(response.data.signature);
      }
    } catch (error: any) {
      const errorMessage = error?.message || "Failed to sign message";
      logger.error("Message signing failed", error as Error, { 
        origin: pendingOrigin,
        messageLength: pendingMessage?.length 
      });
      toast.error(errorMessage);
      if (pendingRequestRef.current) {
        pendingRequestRef.current.reject(new Error(errorMessage));
      }
    } finally {
      setPendingMessage("");
      setPendingOrigin("");
      pendingRequestRef.current = null;
    }
  };

  const handleRejectMessage = () => {
    if (pendingRequestRef.current) {
      pendingRequestRef.current.reject(new Error("User rejected message signing"));
    }
    setPendingMessage("");
    setPendingOrigin("");
    pendingRequestRef.current = null;
    toast("Message signing cancelled");
  };

  const quickLinks = [
    { name: "Zcash Foundation", url: "https://zfnd.org" },
    { name: "Zcash Community", url: "https://zcashcommunity.com" },
    { name: "Zcash Explorer", url: "https://zcashblockexplorer.com" },
  ];

  return (
    <div className="h-full flex flex-col bg-white dark:bg-gray-900">
      {/* Browser Tabs */}
      {tabs.length > 0 && (
        <div className="flex items-center gap-1 bg-gray-100 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-2 overflow-x-auto">
          {tabs.map((tab) => (
            <BrowserTab
              key={tab.id}
              id={tab.id}
              title={tab.title}
              url={tab.url}
              isActive={tab.id === activeTabId}
              isLoading={tab.isLoading}
              onSelect={() => selectTab(tab.id)}
              onClose={() => closeTab(tab.id)}
            />
          ))}
        </div>
      )}

      {/* Browser Toolbar */}
      <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border-b border-gray-200 dark:border-gray-700 p-4">
        <div className="flex items-center gap-2 mb-4">
          <Button
            variant="secondary"
            size="sm"
            onClick={handleBack}
            disabled={!canGoBack}
            className="p-2"
          >
            <ArrowLeft size={18} />
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={handleForward}
            disabled={!canGoForward}
            className="p-2"
          >
            <ArrowRight size={18} />
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={handleRefresh}
            className="p-2"
          >
            <Refresh size={18} />
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={handleHome}
            className="p-2"
            title="Home (Ctrl+H)"
          >
            <Home size={18} />
          </Button>

          <Button
            variant="secondary"
            size="sm"
            onClick={createNewTab}
            className="p-2"
            title="New Tab (Ctrl+T)"
          >
            <span className="text-lg font-bold">+</span>
          </Button>

          <form onSubmit={handleSubmit} className="flex-1 flex gap-2">
            <div className="flex-1 relative">
              <Input
                ref={addressBarRef}
                type="text"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="Enter URL or search... (Ctrl+L to focus)"
                className="w-full pr-10"
                aria-label="Address bar"
                title="Press Ctrl+L to focus"
              />
              {currentUrl && (
                <div className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1">
                  <button
                    type="button"
                    onClick={handleToggleBookmark}
                    className="text-gray-400 hover:text-primary transition-colors"
                    title={isBookmarked ? "Remove bookmark" : "Add bookmark"}
                  >
                    <Bookmark size={16} />
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      window.open(currentUrl, "_blank");
                    }}
                    className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                    title="Open in external browser"
                  >
                    <Shield size={16} />
                  </button>
                </div>
              )}
            </div>
            <Button type="submit" size="sm">
              Go
            </Button>
          </form>

          <Button
            variant="secondary"
            size="sm"
            onClick={() => setShowBookmarks(!showBookmarks)}
            className="p-2"
            title="Bookmarks"
          >
            <Bookmark size={18} />
          </Button>

          <Button
            variant="secondary"
            size="sm"
            onClick={() => setShowHistory(!showHistory)}
            className="p-2"
            title="History"
          >
            <HistoryIcon size={18} />
          </Button>

          {address && (
            <div className="flex items-center gap-2 px-3 py-1.5 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800">
              <div className="w-2 h-2 bg-green-500 rounded-full"></div>
              <span className="text-xs text-green-700 dark:text-green-400 font-medium">
                Wallet Connected
              </span>
            </div>
          )}
        </div>

        {/* Quick Links & Bookmarks/History */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4 text-sm">
            <span className="text-gray-500 dark:text-gray-400 text-xs">Quick Links:</span>
            {quickLinks.map((link) => (
              <button
                key={link.name}
                onClick={() => {
                  setUrl(link.url);
                  handleNavigate(link.url);
                }}
                className="text-primary-600 dark:text-primary-400 hover:underline text-xs"
              >
                {link.name}
              </button>
            ))}
          </div>

          {/* Bookmarks Dropdown */}
          {showBookmarks && (
            <div className="absolute right-4 top-20 bg-white dark:bg-gray-800 rounded-lg shadow-xl border border-gray-200 dark:border-gray-700 p-3 max-w-xs w-full z-50">
              <div className="flex items-center justify-between mb-2">
                <h4 className="text-sm font-semibold text-gray-900 dark:text-gray-100">Bookmarks</h4>
                <button
                  onClick={() => setShowBookmarks(false)}
                  className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                >
                  <CloseCircle size={16} />
                </button>
              </div>
              <div className="max-h-60 overflow-y-auto space-y-1">
                {bookmarks.length === 0 ? (
                  <p className="text-xs text-gray-500 dark:text-gray-400 text-center py-4">
                    No bookmarks yet
                  </p>
                ) : (
                  bookmarks.map((bookmark, index) => (
                    <button
                      key={index}
                      onClick={() => {
                        setUrl(bookmark.url);
                        handleNavigate(bookmark.url);
                        setShowBookmarks(false);
                      }}
                      className="w-full text-left px-2 py-1.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-sm text-gray-700 dark:text-gray-300"
                    >
                      <p className="font-medium truncate">{bookmark.name}</p>
                      <p className="text-xs text-gray-500 dark:text-gray-400 truncate">{bookmark.url}</p>
                    </button>
                  ))
                )}
              </div>
            </div>
          )}

          {/* History Dropdown */}
          {showHistory && (
            <div className="absolute right-4 top-20 bg-white dark:bg-gray-800 rounded-lg shadow-xl border border-gray-200 dark:border-gray-700 p-3 max-w-xs w-full z-50">
              <div className="flex items-center justify-between mb-2">
                <h4 className="text-sm font-semibold text-gray-900 dark:text-gray-100">History</h4>
                <button
                  onClick={() => setShowHistory(false)}
                  className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                >
                  <CloseCircle size={16} />
                </button>
              </div>
              <div className="max-h-60 overflow-y-auto space-y-1">
                {history.length === 0 ? (
                  <p className="text-xs text-gray-500 dark:text-gray-400 text-center py-4">
                    No history yet
                  </p>
                ) : (
                  [...history].reverse().slice(0, 20).map((entry, index) => (
                    <button
                      key={index}
                      onClick={() => {
                        setUrl(entry.url);
                        handleNavigate(entry.url);
                        setShowHistory(false);
                      }}
                      className="w-full text-left px-2 py-1.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-sm text-gray-700 dark:text-gray-300"
                    >
                      <p className="font-medium truncate">{entry.title}</p>
                      <p className="text-xs text-gray-500 dark:text-gray-400 truncate">{entry.url}</p>
                    </button>
                  ))
                )}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Browser Content */}
      <div className="flex-1 relative">
        {!currentUrl ? (
          <div className="h-full flex items-center justify-center bg-gray-50 dark:bg-gray-800">
            <div className="text-center space-y-4 max-w-md">
              <div className="w-16 h-16 mx-auto bg-primary-100 dark:bg-primary-900/30 rounded-full flex items-center justify-center">
                <Lock size={32} className="text-primary-600 dark:text-primary-400" />
              </div>
              <h3 className="text-xl font-semibold text-gray-900 dark:text-gray-100">
                Web3 Browser
              </h3>
              <p className="text-gray-500 dark:text-gray-400 text-sm">
                Enter a URL above to browse web3 applications. Your wallet is ready to connect
                to dApps that support Zcash.
              </p>
              {!address && (
                <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg p-3">
                  <p className="text-xs text-amber-800 dark:text-amber-200">
                    ⚠️ Wallet not connected. Please unlock your wallet to use web3 features.
                  </p>
                </div>
              )}
            </div>
          </div>
        ) : (
          <>
            {isLoading && (
              <div className="absolute top-0 left-0 right-0 h-1 bg-gray-200 dark:bg-gray-700 z-10">
                <div className="h-full bg-primary-600 animate-pulse" style={{ width: "30%" }} />
              </div>
            )}
            <iframe
              ref={(el) => {
                if (el) {
                  iframeRefs.current.set(activeTabId, el);
                  if (el.id !== activeTabId) {
                    el.id = activeTabId;
                  }
                }
              }}
              src={currentUrl}
              className="w-full h-full border-0"
              onLoad={handleIframeLoad}
              sandbox="allow-same-origin allow-scripts allow-forms allow-popups allow-modals allow-top-navigation"
              title={`Browser tab: ${activeTab?.title || currentUrl}`}
              allow="clipboard-read; clipboard-write"
              aria-label={`Browser content for ${activeTab?.title || currentUrl}`}
            />
          </>
        )}
      </div>

      {/* Security Warning */}
      {showSecurityWarning && (
        <div className="fixed inset-0 bg-black/60 dark:bg-black/80 flex items-center justify-center z-50 p-4">
          <div className="bg-white dark:bg-gray-800 rounded-2xl p-6 max-w-md w-full shadow-2xl border border-gray-200 dark:border-gray-700">
            <div className="flex items-center gap-3 mb-4">
              <div className="w-10 h-10 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
                <Shield size={20} className="text-red-600 dark:text-red-400" />
              </div>
              <div>
                <h3 className="font-semibold text-gray-900 dark:text-gray-100">
                  Security Warning
                </h3>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  You are about to visit an untrusted site
                </p>
              </div>
            </div>

            <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg p-3 mb-4">
              <p className="text-xs text-amber-800 dark:text-amber-200">
                ⚠️ This website is not in our trusted list. Be cautious when:
              </p>
              <ul className="text-xs text-amber-800 dark:text-amber-200 mt-2 list-disc list-inside space-y-1">
                <li>Connecting your wallet</li>
                <li>Signing transactions</li>
                <li>Entering sensitive information</li>
              </ul>
            </div>

            <div className="flex gap-3">
              <Button
                variant="secondary"
                onClick={() => {
                  setShowSecurityWarning(false);
                  setUrl("https://");
                  updateTab(activeTabId, { url: "" });
                }}
                className="flex-1"
              >
                Go Back
              </Button>
              <Button
                onClick={() => {
                  setShowSecurityWarning(false);
                  // Navigation will proceed automatically
                }}
                className="flex-1"
              >
                Continue Anyway
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Transaction Approval Dialog */}
      <TransactionApprovalDialog
        isOpen={!!pendingTransaction}
        transaction={pendingTransaction || { to: "", value: "0x0" }}
        origin={pendingOrigin}
        onApprove={handleApproveTransaction}
        onReject={handleRejectTransaction}
      />

      {/* Message Signing Dialog */}
      <MessageSigningDialog
        isOpen={!!pendingMessage}
        message={pendingMessage}
        origin={pendingOrigin}
        onApprove={handleApproveMessage}
        onReject={handleRejectMessage}
      />
    </div>
  );
}
