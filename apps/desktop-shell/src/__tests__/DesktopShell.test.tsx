import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import App from "../App";
import * as api from "../api";

// Mock the API module
vi.mock("../api", () => ({
  login: vi.fn(),
  getProfile: vi.fn(),
  logout: vi.fn(),
  listWindows: vi.fn().mockResolvedValue([]),
  openWindow: vi.fn(),
  closeWindow: vi.fn(),
  minimizeWindow: vi.fn(),
  maximizeWindow: vi.fn(),
  restoreWindow: vi.fn(),
  focusWindow: vi.fn(),
  moveWindow: vi.fn(),
  resizeWindow: vi.fn(),
  listWorkspaces: vi.fn().mockResolvedValue([]),
  getActiveWorkspace: vi.fn().mockResolvedValue({
    id: "ws-1",
    user_id: "u-1",
    name: "Workspace 1",
    index: 0,
    active: true,
    created_at: "",
  }),
  createWorkspace: vi.fn(),
  switchWorkspace: vi.fn(),
  deleteWorkspace: vi.fn(),
  launchApp: vi.fn(),
  stopApp: vi.fn(),
  listRunningApps: vi.fn().mockResolvedValue([]),
  listNotifications: vi.fn().mockResolvedValue([]),
  markNotificationRead: vi.fn(),
  dismissNotification: vi.fn(),
  search: vi.fn().mockResolvedValue([]),
  getSetting: vi.fn(),
  setSetting: vi.fn(),
  listSettings: vi.fn().mockResolvedValue([]),
  healthCheck: vi.fn().mockResolvedValue({ status: "ok" }),
}));

// Mock BusClient
vi.mock("../bus", () => ({
  BusClient: vi.fn().mockImplementation(() => ({
    onEvent: vi.fn(),
    setOnConnectionChange: vi.fn(),
    connect: vi.fn(),
    dispose: vi.fn(),
    send: vi.fn(),
    isConnected: vi.fn().mockReturnValue(false),
  })),
}));

describe("DesktopShell", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows login screen when not authenticated", () => {
    render(<App />);
    expect(screen.getByRole("main", { name: /login/i })).toBeInTheDocument();
    expect(screen.getByText("Sign in to your account")).toBeInTheDocument();
  });

  it("shows username and password fields on login", () => {
    render(<App />);
    expect(screen.getByLabelText(/username/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /sign in/i })).toBeInTheDocument();
  });

  it("disables sign in button when fields are empty", () => {
    render(<App />);
    const btn = screen.getByRole("button", { name: /sign in/i });
    expect(btn).toBeDisabled();
  });

  it("enables sign in button when fields are filled", () => {
    render(<App />);
    fireEvent.change(screen.getByLabelText(/username/i), { target: { value: "admin" } });
    fireEvent.change(screen.getByLabelText(/password/i), { target: { value: "password" } });
    const btn = screen.getByRole("button", { name: /sign in/i });
    expect(btn).not.toBeDisabled();
  });

  it("handles login submission", async () => {
    const mockLogin = vi.mocked(api.login).mockResolvedValue({
      session_id: "s-1",
      token: "tok-1",
      user_id: "u-1",
      expires_at: "2099-01-01T00:00:00Z",
    });
    vi.mocked(api.getProfile).mockResolvedValue({
      user_id: "u-1",
      username: "admin",
      display_name: "Admin",
      created_at: "2024-01-01T00:00:00Z",
    });
    vi.mocked(api.listWorkspaces).mockResolvedValue([
      { id: "ws-1", user_id: "u-1", name: "Workspace 1", index: 0, active: true, created_at: "" },
    ]);
    vi.mocked(api.getActiveWorkspace).mockResolvedValue({
      id: "ws-1",
      user_id: "u-1",
      name: "Workspace 1",
      index: 0,
      active: true,
      created_at: "",
    });

    render(<App />);
    fireEvent.change(screen.getByLabelText(/username/i), { target: { value: "admin" } });
    fireEvent.change(screen.getByLabelText(/password/i), { target: { value: "password" } });

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /sign in/i }));
    });

    expect(mockLogin).toHaveBeenCalledWith("admin", "password");
  });

  it("shows error on failed login", async () => {
    vi.mocked(api.login).mockRejectedValue(new Error("Invalid credentials"));

    render(<App />);
    fireEvent.change(screen.getByLabelText(/username/i), { target: { value: "admin" } });
    fireEvent.change(screen.getByLabelText(/password/i), { target: { value: "wrong" } });

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /sign in/i }));
    });

    expect(screen.getByRole("alert")).toHaveTextContent("Invalid credentials");
  });

  it("renders desktop shell after successful login", async () => {
    vi.mocked(api.login).mockResolvedValue({
      session_id: "s-1",
      token: "tok-1",
      user_id: "u-1",
      expires_at: "2099-01-01T00:00:00Z",
    });
    vi.mocked(api.getProfile).mockResolvedValue({
      user_id: "u-1",
      username: "admin",
      display_name: "Admin",
      created_at: "",
    });
    vi.mocked(api.listWorkspaces).mockResolvedValue([
      { id: "ws-1", user_id: "u-1", name: "Workspace 1", index: 0, active: true, created_at: "" },
    ]);
    vi.mocked(api.getActiveWorkspace).mockResolvedValue({
      id: "ws-1",
      user_id: "u-1",
      name: "Workspace 1",
      index: 0,
      active: true,
      created_at: "",
    });

    render(<App />);

    // Fill in login form and submit
    fireEvent.change(screen.getByLabelText(/username/i), { target: { value: "admin" } });
    fireEvent.change(screen.getByLabelText(/password/i), { target: { value: "password" } });

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /sign in/i }));
    });

    // Wait for bootstrap to complete
    await act(async () => {
      await new Promise((r) => setTimeout(r, 100));
    });

    // Should show taskbar with app launcher button
    expect(screen.getByRole("toolbar", { name: /taskbar/i })).toBeInTheDocument();
  });
});
