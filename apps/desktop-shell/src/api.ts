/**
 * HTTP API client for the CortexOS Desktop Shell.
 *
 * All backend communication goes through this module.
 * Responses are unwrapped from the `SuccessResponse<T>` envelope.
 * Authentication is handled via HttpOnly Secure SameSite=Strict cookies
 * set by the backend; the frontend never stores or sends tokens.
 *
 * WM mutation operations (move, resize, focus, switchWorkspace) are sent
 * through the WebSocket command bus for realtime performance, while
 * lifecycle operations (open, close) use HTTP.
 */

import type {
  ApiResponse,
  LoginResponse,
  ProfileResponse,
  WindowState,
  WorkspaceState,
  AppInstance,
  NotificationInfo,
  SearchResult,
} from "./types";

import { BusClient } from "./bus";

const API_BASE: string = import.meta.env.VITE_API_URL || "";

class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(options?.headers as Record<string, string>),
  };

  const res = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers,
    credentials: "include",
  });

  if (!res.ok) {
    let message = res.statusText;
    try {
      const body = await res.json();
      message = body.error?.message || message;
    } catch (err) {
      console.error("Failed to parse error response body:", err);
    }
    throw new ApiError(res.status, "HTTP_ERROR", message);
  }

  const json: ApiResponse<T> = await res.json();
  return json.data;
}

// ── Bus singleton ────────────────────────────────────────────────────────────

let busInstance: BusClient | null = null;

/**
 * Get (or create) the singleton BusClient used for WM mutations.
 * Auth is handled via HttpOnly cookies; no token is needed.
 */
export function getBus(): BusClient {
  if (!busInstance) {
    busInstance = new BusClient();
  }
  return busInstance;
}

// ── Auth ────────────────────────────────────────────────────────────────────────

export async function login(username: string, password: string): Promise<LoginResponse> {
  return request("/api/v1/auth/login", {
    method: "POST",
    body: JSON.stringify({ username, password }),
  });
}

export async function getProfile(): Promise<ProfileResponse> {
  return request("/api/v1/auth/profile");
}

// ── Windows ─────────────────────────────────────────────────────────────────────

export async function listWindows(workspaceId: string): Promise<WindowState[]> {
  return request(`/api/v1/wm/windows?workspace_id=${encodeURIComponent(workspaceId)}`);
}

export async function openWindow(
  opts: {
    instance_id: string;
    title: string;
    x?: number;
    y?: number;
    width?: number;
    height?: number;
    workspace_id?: string;
  },
): Promise<WindowState> {
  return request("/api/v1/wm/windows", {
    method: "POST",
    body: JSON.stringify({
      instance_id: opts.instance_id,
      title: opts.title,
      x: opts.x ?? 100,
      y: opts.y ?? 100,
      width: opts.width ?? 800,
      height: opts.height ?? 600,
      workspace_id: opts.workspace_id,
    }),
  });
}

export async function closeWindow(windowId: string): Promise<void> {
  await request(`/api/v1/wm/windows/${encodeURIComponent(windowId)}/close`, {
    method: "POST",
  });
}

export async function minimizeWindow(windowId: string): Promise<WindowState> {
  return request(`/api/v1/wm/windows/${encodeURIComponent(windowId)}/minimize`, {
    method: "POST",
  });
}

export async function maximizeWindow(windowId: string): Promise<WindowState> {
  return request(`/api/v1/wm/windows/${encodeURIComponent(windowId)}/maximize`, {
    method: "POST",
  });
}

export async function restoreWindow(windowId: string): Promise<WindowState> {
  return request(`/api/v1/wm/windows/${encodeURIComponent(windowId)}/restore`, {
    method: "POST",
  });
}

// ── WM mutations via command bus ─────────────────────────────────────────────
//
// moveWindow, resizeWindow, focusWindow, and switchWorkspace send commands
// through the WebSocket bus rather than HTTP POST, per the reconciled spec.
// Auth is handled via HttpOnly cookies on the WebSocket upgrade request.

export function focusWindow(windowId: string): void {
  getBus().send("wm.focus", { window_id: windowId });
}

export function moveWindow(windowId: string, x: number, y: number): void {
  getBus().send("wm.move", { window_id: windowId, x, y });
}

export function resizeWindow(
  windowId: string,
  width: number,
  height: number,
): void {
  getBus().send("wm.resize", { window_id: windowId, width, height });
}

// ── Workspaces ──────────────────────────────────────────────────────────────────

export async function listWorkspaces(): Promise<WorkspaceState[]> {
  return request("/api/v1/wm/workspaces");
}

export async function getActiveWorkspace(): Promise<WorkspaceState> {
  return request("/api/v1/wm/workspaces/active");
}

export async function createWorkspace(name: string): Promise<WorkspaceState> {
  return request("/api/v1/wm/workspaces", {
    method: "POST",
    body: JSON.stringify({ name }),
  });
}

export function switchWorkspace(workspaceId: string): void {
  getBus().send("wm.workspace.activate", { workspace_id: workspaceId });
}

export async function deleteWorkspace(workspaceId: string): Promise<void> {
  await request(`/api/v1/wm/workspaces/${encodeURIComponent(workspaceId)}`, {
    method: "DELETE",
  });
}

// ── Apps ─────────────────────────────────────────────────────────────────────────

export async function launchApp(appId: string): Promise<AppInstance> {
  return request("/api/v1/apps/launch", {
    method: "POST",
    body: JSON.stringify({ app_id: appId }),
  });
}

export async function stopApp(instanceId: string): Promise<void> {
  await request(`/api/v1/apps/stop/${encodeURIComponent(instanceId)}`, {
    method: "POST",
  });
}

export async function listRunningApps(): Promise<AppInstance[]> {
  return request("/api/v1/apps/running");
}

// ── Notifications ───────────────────────────────────────────────────────────────

export async function listNotifications(): Promise<NotificationInfo[]> {
  return request("/api/v1/notifications");
}

export async function markNotificationRead(id: string): Promise<void> {
  await request(`/api/v1/notifications/${encodeURIComponent(id)}/read`, {
    method: "POST",
  });
}

export async function dismissNotification(id: string): Promise<void> {
  await request(`/api/v1/notifications/${encodeURIComponent(id)}/dismiss`, {
    method: "POST",
  });
}

// ── Search ──────────────────────────────────────────────────────────────────────

export async function search(
  query: string,
  limit?: number,
): Promise<SearchResult[]> {
  const params = new URLSearchParams({ q: query });
  if (limit) params.set("limit", String(limit));
  return request(`/api/v1/search?${params}`);
}

// ── Settings ────────────────────────────────────────────────────────────────────

export async function getSetting(namespace: string, key: string): Promise<string> {
  return request(`/api/v1/settings/${encodeURIComponent(namespace)}/${encodeURIComponent(key)}`);
}

export async function setSetting(
  namespace: string,
  key: string,
  value: unknown,
): Promise<void> {
  await request("/api/v1/settings", {
    method: "POST",
    body: JSON.stringify({ namespace, key, value }),
  });
}

export async function listSettings(
  namespace?: string,
): Promise<Array<{ namespace: string; key: string; value: string }>> {
  const params = namespace ? `?namespace=${encodeURIComponent(namespace)}` : "";
  return request(`/api/v1/settings${params}`);
}

// ── Auth (additional) ──────────────────────────────────────────────────────────

export async function logout(): Promise<void> {
  await request("/api/v1/auth/logout", {
    method: "POST",
  });
}

// ── Health ──────────────────────────────────────────────────────────────────────

export async function healthCheck(): Promise<{ status: string }> {
  const res = await fetch(`${API_BASE}/api/v1/health`);
  return res.json();
}
