const STORAGE_KEY = "nozy_web_api_base";

export function getApiBase(): string {
  return (
    localStorage.getItem(STORAGE_KEY)?.replace(/\/$/, "") || "http://127.0.0.1:3000"
  );
}

export function setApiBase(url: string) {
  localStorage.setItem(STORAGE_KEY, url.replace(/\/$/, ""));
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${getApiBase()}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...(init?.headers ?? {}),
    },
  });
  if (!res.ok) {
    let msg = res.statusText;
    try {
      const j = await res.json();
      msg = j.error || j.message || msg;
    } catch {
      /* ignore */
    }
    throw new Error(msg);
  }
  return res.json() as Promise<T>;
}

export type Profile = {
  role: string;
  orchard_account: number;
  business_display_name?: string;
  linked_zns_name?: string;
  linked_zns_display?: string;
  receive_address?: string;
  business_address?: string;
  personal_address?: string;
  network: string;
};

export const api = {
  getProfile: (password?: string) => {
    const q = password ? `?password=${encodeURIComponent(password)}` : "";
    return request<Profile>(`/api/profile${q}`);
  },
  updateProfile: (body: {
    password?: string;
    role?: string;
    business_display_name?: string;
  }) =>
    request<Profile>("/api/profile", {
      method: "POST",
      body: JSON.stringify(body),
    }),
  linkZns: (body: { name: string; password?: string }) =>
    request<{
      linked: boolean;
      name: string;
      display: string;
      address: string;
      business_address: string;
    }>("/api/zns/link", { method: "POST", body: JSON.stringify(body) }),
  unlinkZns: () =>
    request<{ linked: boolean }>("/api/zns/link", { method: "DELETE" }),
  getBalance: () =>
    request<{ balance_zec: number; available_zec?: number }>("/api/balance"),
  sync: (password?: string) =>
    request<{ success: boolean }>("/api/sync", {
      method: "POST",
      body: JSON.stringify({ password: password ?? null }),
    }),
};
