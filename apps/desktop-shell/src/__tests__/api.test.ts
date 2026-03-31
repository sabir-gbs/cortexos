import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock fetch globally
const mockFetch = vi.fn();
globalThis.fetch = mockFetch;

// Import after mock setup
import * as api from "../api";

describe("API Client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("healthCheck returns status", async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ status: "ok" }),
    });
    const result = await api.healthCheck();
    expect(result).toEqual({ status: "ok" });
    expect(mockFetch).toHaveBeenCalledWith("/api/v1/health");
  });

  it("login sends POST to /api/v1/auth/login", async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: () =>
        Promise.resolve({
          data: { session_id: "s-1", user_id: "u-1", expires_at: "" },
        }),
    });
    const result = await api.login("admin", "password");
    expect(result.session_id).toBe("s-1");
    expect(mockFetch).toHaveBeenCalledWith(
      "/api/v1/auth/login",
      expect.objectContaining({ method: "POST" }),
    );
  });

  it("throws on HTTP error", async () => {
    mockFetch.mockResolvedValue({
      ok: false,
      status: 401,
      statusText: "Unauthorized",
      json: () => Promise.resolve({ error: { message: "invalid credentials" } }),
    });
    await expect(api.login("admin", "wrong")).rejects.toThrow("invalid credentials");
  });

  it("throws on non-JSON error response", async () => {
    mockFetch.mockResolvedValue({
      ok: false,
      status: 500,
      statusText: "Internal Server Error",
      json: () => Promise.reject(new Error("not json")),
    });
    await expect(api.healthCheck()).rejects.toThrow();
  });

  it("sends credentials with requests", async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ data: [] }),
    });
    await api.listWindows("ws-1");
    const call = mockFetch.mock.calls[0];
    const opts = call[1] as RequestInit;
    expect(opts.credentials).toBe("include");
  });
});
