import {
  isTauri,
  getSyncStatus,
  triggerFullSync,
  getPlatformInfo,
  disconnectBackend,
  setServerUrl as setBackendServerUrl,
  type SyncStatus,
  type PlatformInfo,
} from "./tauri-bridge";

const STORAGE_KEY_TOKEN = "moa_token";
const STORAGE_KEY_SERVER = "moa_server_url";
const DEFAULT_SERVER_URL = "https://moanew-production.up.railway.app";

export interface PairResponse {
  paired: boolean;
  token: string;
}

export interface ChatResponse {
  response: string;
  model: string;
}

export interface HealthResponse {
  status: string;
}

export type { SyncStatus, PlatformInfo };

export class MoAClient {
  private serverUrl: string;
  private token: string | null;

  constructor() {
    this.serverUrl = localStorage.getItem(STORAGE_KEY_SERVER) || DEFAULT_SERVER_URL;
    this.token = localStorage.getItem(STORAGE_KEY_TOKEN);
  }

  getServerUrl(): string {
    return this.serverUrl;
  }

  setServerUrl(url: string): void {
    this.serverUrl = url.replace(/\/+$/, "");
    localStorage.setItem(STORAGE_KEY_SERVER, this.serverUrl);
    // Also update Tauri backend if running in Tauri
    if (isTauri()) {
      setBackendServerUrl(this.serverUrl).catch(() => {});
    }
  }

  getToken(): string | null {
    return this.token;
  }

  isConnected(): boolean {
    return this.token !== null && this.token.length > 0;
  }

  getMaskedToken(): string {
    if (!this.token) return "";
    if (this.token.length <= 8) return "****";
    return this.token.substring(0, 4) + "..." + this.token.substring(this.token.length - 4);
  }

  disconnect(): void {
    this.token = null;
    localStorage.removeItem(STORAGE_KEY_TOKEN);
    // Also disconnect on Tauri backend
    if (isTauri()) {
      disconnectBackend().catch(() => {});
    }
  }

  async pair(
    code: string,
    username?: string,
    password?: string,
  ): Promise<PairResponse> {
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      "X-Pairing-Code": code,
    };

    const body: Record<string, string> = {};
    if (username) body.username = username;
    if (password) body.password = password;

    const res = await fetch(`${this.serverUrl}/pair`, {
      method: "POST",
      headers,
      body: Object.keys(body).length > 0 ? JSON.stringify(body) : undefined,
    });

    if (!res.ok) {
      const text = await res.text().catch(() => "Unknown error");
      throw new Error(`Pairing failed (${res.status}): ${text}`);
    }

    const data: PairResponse = await res.json();

    if (data.paired && data.token) {
      this.token = data.token;
      localStorage.setItem(STORAGE_KEY_TOKEN, data.token);
    }

    return data;
  }

  async chat(message: string): Promise<ChatResponse> {
    if (!this.token) {
      throw new Error("Not authenticated. Please pair with the server first.");
    }

    const res = await fetch(`${this.serverUrl}/webhook`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${this.token}`,
      },
      body: JSON.stringify({ message }),
    });

    if (!res.ok) {
      if (res.status === 401) {
        this.disconnect();
        throw new Error("Authentication expired. Please re-pair with the server.");
      }
      const text = await res.text().catch(() => "Unknown error");
      throw new Error(`Chat request failed (${res.status}): ${text}`);
    }

    return await res.json();
  }

  async healthCheck(): Promise<HealthResponse> {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 5000);

    try {
      const res = await fetch(`${this.serverUrl}/health`, {
        method: "GET",
        signal: controller.signal,
      });

      if (!res.ok) {
        throw new Error(`Health check failed (${res.status})`);
      }

      return await res.json();
    } catch (err) {
      if (err instanceof DOMException && err.name === "AbortError") {
        throw new Error("Health check timed out");
      }
      throw err;
    } finally {
      clearTimeout(timeout);
    }
  }

  // ── Sync commands (Tauri backend only) ──────────────────────────

  /** Get sync status from Tauri backend. Returns null when not in Tauri. */
  async getSyncStatus(): Promise<SyncStatus | null> {
    return getSyncStatus();
  }

  /** Trigger a full sync (Layer 3). Returns null when not in Tauri. */
  async triggerFullSync(): Promise<string | null> {
    return triggerFullSync();
  }

  /** Get platform info. Returns null when not in Tauri. */
  async getPlatformInfo(): Promise<PlatformInfo | null> {
    return getPlatformInfo();
  }
}

export const apiClient = new MoAClient();
