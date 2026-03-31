import React, { useState, useCallback, useEffect, useRef } from "react";
import type {
  ProfileResponse,
  WindowState,
  WorkspaceState,
  DesktopIcon,
  GridPosition,
  AppManifest,
  AppInstance,
  NotificationInfo,
  ShellSettings,
  ContextMenuState,
  OverlayType,
} from "./types";
import { BUILTIN_APPS, DEFAULT_SETTINGS } from "./types";
import * as api from "./api";
import { BusClient } from "./bus";
import { LoginScreen } from "./components/LoginScreen";
import { DesktopBackground } from "./components/DesktopBackground";
import { DesktopIconGrid } from "./components/DesktopIconGrid";
import { WindowManager } from "./components/WindowManager";
import { Taskbar } from "./components/Taskbar";
import { AppLauncher } from "./components/AppLauncher";
import {
  ContextMenu,
  buildDesktopContextMenu,
  buildIconContextMenu,
} from "./components/ContextMenu";
import { ConnectionBanner } from "./components/ConnectionBanner";
import { SettingsPanel } from "./components/SettingsPanel";
import CommandPalette from "./components/CommandPalette";
import "./index.css";

export default function App() {
  // ── Auth state ──────────────────────────────────────────────────────────
  const [authenticated, setAuthenticated] = useState(false);
  const [user, setUser] = useState<ProfileResponse | null>(null);
  const [initialized, setInitialized] = useState(false);

  // ── Shell state ───────────────────────────────────────────────────────────
  const [windows, setWindows] = useState<WindowState[]>([]);
  const [workspaces, setWorkspaces] = useState<WorkspaceState[]>([]);
  const [activeWorkspaceId, setActiveWorkspaceId] = useState<string | null>(null);
  const [desktopIcons, setDesktopIcons] = useState<DesktopIcon[]>(DEFAULT_SETTINGS.desktop.icons);
  const [installedApps] = useState<AppManifest[]>(BUILTIN_APPS);
  const [runningApps, setRunningApps] = useState<AppInstance[]>([]);
  const [notifications, setNotifications] = useState<NotificationInfo[]>([]);
  const [settings, setSettings] = useState<ShellSettings>(DEFAULT_SETTINGS);
  const [connected, setConnected] = useState(false);
  const [activeOverlay, setActiveOverlay] = useState<OverlayType>(null);
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
  const [ariaMessage, setAriaMessage] = useState("");

  const busRef = useRef<BusClient | null>(null);

  // ── Announce helper ───────────────────────────────────────────────────────
  const announce = useCallback((message: string) => {
    setAriaMessage(message);
    setTimeout(() => setAriaMessage(""), 2000);
  }, []);

  // ── Bootstrap ─────────────────────────────────────────────────────────────
  useEffect(() => {
    if (!authenticated) {
      setInitialized(true);
      return;
    }

    let cancelled = false;

    async function bootstrap() {
      try {
        const [profile, wsList, activeWs, notifs] = await Promise.allSettled([
          api.getProfile(),
          api.listWorkspaces(),
          api.getActiveWorkspace(),
          api.listNotifications(),
        ]);

        if (cancelled) return;

        if (profile.status === "fulfilled") {
          setUser(profile.value);
        } else {
          // Profile fetch failed - session cookie is invalid
          setAuthenticated(false);
          setInitialized(true);
          return;
        }

        if (wsList.status === "fulfilled") {
          setWorkspaces(wsList.value);
        }

        if (activeWs.status === "fulfilled") {
          setActiveWorkspaceId(activeWs.value.id);

          // Load windows for the active workspace
          try {
            const wins = await api.listWindows(activeWs.value.id);
            if (!cancelled) setWindows(wins);
          } catch (err) {
            console.error("Failed to load workspace windows:", err);
          }
        }

        if (notifs.status === "fulfilled") {
          setNotifications(notifs.value);
        }
      } catch (err) {
        console.error("Bootstrap failed:", err);
      } finally {
        if (!cancelled) setInitialized(true);
      }
    }

    bootstrap();
    return () => {
      cancelled = true;
    };
  }, [authenticated]);

  // ── WebSocket ─────────────────────────────────────────────────────────────
  useEffect(() => {
    if (!authenticated) return;

    const bus = new BusClient();
    bus.setOnConnectionChange((c) => setConnected(c));
    bus.onEvent((event, payload) => {
      switch (event) {
        case "window.created":
        case "window.updated":
        case "wm.focus.changed": {
          const win = payload as WindowState;
          setWindows((prev) => {
            const idx = prev.findIndex((w) => w.id === win.id);
            if (idx >= 0) {
              const next = [...prev];
              next[idx] = { ...next[idx], ...win };
              return next;
            }
            return [...prev, win];
          });
          break;
        }
        case "window.closed": {
          const { window_id } = payload as { window_id: string };
          setWindows((prev) => prev.filter((w) => w.id !== window_id));
          break;
        }
        case "wm.workspace.changed": {
          const ws = payload as WorkspaceState;
          setActiveWorkspaceId(ws.id);
          setWorkspaces((prev) => prev.map((w) => ({ ...w, active: w.id === ws.id })));
          break;
        }
        case "app.launched": {
          const inst = payload as AppInstance;
          setRunningApps((prev) => [
            ...prev.filter((r) => r.instance_id !== inst.instance_id),
            inst,
          ]);
          announce(`${inst.app_id} launched`);
          break;
        }
        case "app.stopped": {
          const { instance_id } = payload as { instance_id: string };
          setRunningApps((prev) => prev.filter((r) => r.instance_id !== instance_id));
          break;
        }
        case "app.crashed": {
          const { app_id, error } = payload as { app_id: string; error: string };
          announce(`${app_id} crashed: ${error}`);
          break;
        }
        case "notification.created": {
          const n = payload as NotificationInfo;
          setNotifications((prev) => [n, ...prev]);
          break;
        }
        case "notification.dismissed": {
          const { notification_id } = payload as { notification_id: string };
          setNotifications((prev) => prev.filter((n) => n.notification_id !== notification_id));
          break;
        }
        case "notification.read": {
          const { notification_id } = payload as { notification_id: string };
          setNotifications((prev) =>
            prev.map((n) => (n.notification_id === notification_id ? { ...n, is_read: true } : n)),
          );
          break;
        }
        case "notification.all_read": {
          setNotifications((prev) => prev.map((n) => ({ ...n, is_read: true })));
          break;
        }
        case "settings.changed": {
          // Re-fetch settings when they change
          break;
        }
        case "connection.lost": {
          announce("Connection lost");
          break;
        }
        case "os.shutdown.initiated": {
          announce("System shutting down");
          break;
        }
      }
    });
    bus.connect();
    busRef.current = bus;

    return () => {
      bus.dispose();
      busRef.current = null;
    };
  }, [authenticated, announce]);

  // ── Auth handlers ─────────────────────────────────────────────────────────
  const handleLogin = useCallback(
    async (username: string, password: string) => {
      await api.login(username, password);
      setAuthenticated(true);
      announce("Signed in");
    },
    [announce],
  );

  const handleLogout = useCallback(() => {
    api.logout().catch((err) => console.error("Logout request failed:", err));
    setAuthenticated(false);
    setUser(null);
    setWindows([]);
    setWorkspaces([]);
    setRunningApps([]);
    setNotifications([]);
    announce("Signed out");
  }, [announce]);

  // ── Window operations ─────────────────────────────────────────────────────
  const handleWindowFocus = useCallback(
    async (windowId: string) => {
      if (!authenticated) return;
      try {
        api.focusWindow(windowId);
        setWindows((prev) =>
          prev.map((w) => ({
            ...w,
            focused: w.id === windowId,
          })),
        );
      } catch (err) {
        console.error("Failed to focus window:", err);
      }
    },
    [authenticated],
  );

  const handleWindowClose = useCallback(
    async (windowId: string) => {
      if (!authenticated) return;
      try {
        await api.closeWindow(windowId);
        setWindows((prev) => prev.filter((w) => w.id !== windowId));
      } catch (err) {
        console.error("Failed to close window:", err);
      }
    },
    [authenticated],
  );

  const handleWindowMinimize = useCallback(
    async (windowId: string) => {
      if (!authenticated) return;
      try {
        const win = await api.minimizeWindow(windowId);
        setWindows((prev) => prev.map((w) => (w.id === windowId ? { ...w, ...win } : w)));
      } catch (err) {
        console.error("Failed to minimize window:", err);
      }
    },
    [authenticated],
  );

  const handleWindowMaximize = useCallback(
    async (windowId: string) => {
      if (!authenticated) return;
      try {
        const win = await api.maximizeWindow(windowId);
        setWindows((prev) => prev.map((w) => (w.id === windowId ? { ...w, ...win } : w)));
      } catch (err) {
        console.error("Failed to maximize window:", err);
      }
    },
    [authenticated],
  );

  const handleWindowMove = useCallback(
    async (windowId: string, x: number, y: number) => {
      if (!authenticated) return;
      try {
        await api.moveWindow(windowId, x, y);
      } catch (err) {
        console.error("Failed to move window:", err);
      }
    },
    [authenticated],
  );

  const handleWindowResize = useCallback(
    async (windowId: string, w: number, h: number) => {
      if (!authenticated) return;
      try {
        await api.resizeWindow(windowId, w, h);
      } catch (err) {
        console.error("Failed to resize window:", err);
      }
    },
    [authenticated],
  );

  // ── App launching ─────────────────────────────────────────────────────────
  const handleLaunchApp = useCallback(
    async (appId: string) => {
      if (!authenticated) return;
      setActiveOverlay(null);

      // Find app manifest
      const manifest = BUILTIN_APPS.find((a) => a.id === appId);
      const title = manifest?.name || appId;

      try {
        const instance = await api.launchApp(appId);
        announce(`Launching ${title}`);

        // Open a window for the app
        const wsId = activeWorkspaceId || "";
        const win = await api.openWindow({
          instance_id: instance.instance_id,
          title,
          x: 100 + Math.random() * 200,
          y: 50 + Math.random() * 150,
          width: 800,
          height: 600,
          workspace_id: wsId || undefined,
        });

        setWindows((prev) => [...prev, win]);
        setRunningApps((prev) => [...prev, instance]);
      } catch (err) {
        announce(
          `Failed to launch ${title}: ${err instanceof Error ? err.message : "unknown error"}`,
        );
      }
    },
    [authenticated, activeWorkspaceId, announce],
  );

  // ── Desktop icon launch ───────────────────────────────────────────────────
  const handleIconLaunch = useCallback(
    (icon: DesktopIcon) => {
      if (icon.app_id) {
        handleLaunchApp(icon.app_id);
      }
    },
    [handleLaunchApp],
  );

  // ── Command palette settings handler ────────────────────────────────────
  const handleOpenSetting = useCallback((_key: string) => {
    setActiveOverlay("settings");
  }, []);

  // ── Command palette command handler ─────────────────────────────────────
  useEffect(() => {
    const handler = (e: Event) => {
      const cmd = (e as CustomEvent).detail as string;
      switch (cmd) {
        case "toggle-theme":
          setSettings((prev) => ({
            ...prev,
            theme: { mode: prev.theme.mode === "dark" ? "light" : "dark" },
          }));
          break;
        case "toggle-fullscreen":
          if (!document.fullscreenElement) {
            document.documentElement.requestFullscreen?.();
          } else {
            document.exitFullscreen?.();
          }
          break;
        case "new-workspace":
          if (authenticated) {
            api.createWorkspace("Workspace").catch((err) => console.error("Failed to create workspace:", err));
          }
          break;
        case "open-launcher":
          setActiveOverlay("launcher");
          break;
        case "close-window": {
          const focusedWin = windows.find((w) => w.focused);
          if (focusedWin && authenticated) {
            api.closeWindow(focusedWin.id).catch((err) => console.error("Failed to close window:", err));
          }
          break;
        }
      }
    };
    window.addEventListener("cortexos:command", handler);
    return () => window.removeEventListener("cortexos:command", handler);
  }, [authenticated, windows]);

  // ── Desktop icon move ─────────────────────────────────────────────────────
  const handleIconMove = useCallback((id: string, position: GridPosition) => {
    setDesktopIcons((prev) => prev.map((icon) => (icon.id === id ? { ...icon, position } : icon)));
  }, []);

  // ── Context menu ──────────────────────────────────────────────────────────
  const handleDesktopContextMenu = useCallback(
    (e: React.MouseEvent, icon?: DesktopIcon) => {
      const items = icon
        ? buildIconContextMenu(icon.label, () => handleIconLaunch(icon))
        : buildDesktopContextMenu(
            () => setActiveOverlay("settings"),
            () => setActiveOverlay("settings"),
            () =>
              setDesktopIcons((prev) => [...prev].sort((a, b) => a.label.localeCompare(b.label))),
            () => setDesktopIcons((prev) => [...prev]),
          );

      setContextMenu({ x: e.clientX, y: e.clientY, items });
    },
    [handleIconLaunch],
  );

  // ── Workspace switching ───────────────────────────────────────────────────
  const handleWorkspaceSwitch = useCallback(
    async (workspaceId: string) => {
      if (!authenticated) return;
      try {
        api.switchWorkspace(workspaceId);
        setActiveWorkspaceId(workspaceId);
        setWorkspaces((prev) => prev.map((w) => ({ ...w, active: w.id === workspaceId })));

        // Reload windows for the new workspace
        const wins = await api.listWindows(workspaceId);
        setWindows(wins);
        const target = workspaces.find((w) => w.id === workspaceId);
        announce(`Switched to ${target?.name ?? "workspace"}`);
      } catch (err) {
        console.error("Failed to switch workspace:", err);
      }
    },
    [authenticated, announce, workspaces],
  );

  // ── Taskbar app click ─────────────────────────────────────────────────────
  const handleTaskbarAppClick = useCallback(
    async (instanceId: string) => {
      const win = windows.find((w) => w.instance_id === instanceId && w.state !== "closed");
      if (win) {
        if (win.state === "minimized") {
          if (authenticated) {
            try {
              await api.restoreWindow(win.id);
            } catch (err) {
              console.error("Failed to restore window:", err);
            }
          }
          setWindows((prev) =>
            prev.map((w) => (w.id === win.id ? { ...w, state: "normal" as const } : w)),
          );
        }
        handleWindowFocus(win.id);
      }
    },
    [windows, authenticated, handleWindowFocus],
  );

  // ── Settings update ───────────────────────────────────────────────────────
  const handleUpdateSetting = useCallback(
    (key: string, value: unknown) => {
      setSettings((prev) => {
        const next = { ...prev };
        const parts = key.split(".");
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        let obj: any = next;
        for (let i = 0; i < parts.length - 1; i++) {
          obj[parts[i]] = { ...obj[parts[i]] };
          obj = obj[parts[i]];
        }
        obj[parts[parts.length - 1]] = value;

        // Apply theme mode immediately
        if (key === "theme.mode") {
          document.documentElement.setAttribute("data-theme", value as string);
        }

        return next;
      });

      // Persist to server
      if (authenticated) {
        api.setSetting("shell", key, value).catch((err) => console.error("Failed to persist setting:", err));
      }
    },
    [authenticated],
  );

  // ── Keyboard shortcuts ────────────────────────────────────────────────────
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      // Ignore if an input is focused
      const tag = (e.target as HTMLElement).tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;

      // Ctrl+Space: Command palette (toggle)
      if (e.ctrlKey && e.key === " ") {
        e.preventDefault();
        setActiveOverlay((prev) => (prev === "palette" ? null : "palette"));
        return;
      }

      // Meta or Ctrl+Shift+A: App launcher
      if (e.key === "Meta" || (e.ctrlKey && e.shiftKey && e.key === "A")) {
        e.preventDefault();
        setActiveOverlay((prev) => (prev === "launcher" ? null : "launcher"));
        return;
      }

      // Escape: Close overlays
      if (e.key === "Escape") {
        setActiveOverlay(null);
        setContextMenu(null);
        return;
      }

      // F11: Toggle fullscreen
      if (e.key === "F11") {
        e.preventDefault();
        if (document.fullscreenElement) {
          document.exitFullscreen();
        } else {
          document.documentElement.requestFullscreen().catch((err) => console.error("Fullscreen request failed:", err));
        }
        return;
      }

      // Alt+Tab: Cycle window focus
      if (e.altKey && e.key === "Tab") {
        e.preventDefault();
        const visibleWindows = windows.filter(
          (w) => w.state !== "closed" && w.state !== "minimized",
        );
        if (visibleWindows.length < 2) return;
        const focusedIdx = visibleWindows.findIndex((w) => w.focused);
        const nextIdx = (focusedIdx + 1) % visibleWindows.length;
        handleWindowFocus(visibleWindows[nextIdx].id);
        return;
      }

      // Ctrl+W / Ctrl+Shift+W: Workspace switch
      if (e.ctrlKey && e.key === "W" && workspaces.length > 1) {
        e.preventDefault();
        const currentIdx = workspaces.findIndex((w) => w.active);
        if (e.shiftKey) {
          // Previous workspace
          const prevIdx = currentIdx > 0 ? currentIdx - 1 : workspaces.length - 1;
          handleWorkspaceSwitch(workspaces[prevIdx].id);
        } else {
          // Next workspace
          const nextIdx = (currentIdx + 1) % workspaces.length;
          handleWorkspaceSwitch(workspaces[nextIdx].id);
        }
        return;
      }

      // Meta+,: Settings
      if (e.metaKey && e.key === ",") {
        e.preventDefault();
        setActiveOverlay((prev) => (prev === "settings" ? null : "settings"));
        return;
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [windows, workspaces, handleWindowFocus, handleWorkspaceSwitch]);

  // ── Render ────────────────────────────────────────────────────────────────

  // Loading state
  if (authenticated && !initialized) {
    return (
      <div className="shell-loading" role="status" aria-label="Loading">
        <div className="shell-loading__spinner" />
        <span className="shell-loading__text">Loading CortexOS...</span>
        <div className="sr-only" aria-live="polite">
          Loading CortexOS
        </div>
      </div>
    );
  }

  // Login screen
  if (!authenticated) {
    return <LoginScreen onLogin={handleLogin} />;
  }

  const taskbarHeight = settings.taskbar.height;

  return (
    <div
      className="shell"
      data-theme={settings.theme.mode}
      style={{ "--taskbar-height": `${taskbarHeight}px` } as React.CSSProperties}
    >
      {/* Connection loss banner */}
      <ConnectionBanner connected={connected} />

      {/* Desktop background */}
      <DesktopBackground background={settings.desktop.background} />

      {/* Desktop icons */}
      <div className="desktop" style={{ bottom: taskbarHeight }}>
        <DesktopIconGrid
          icons={desktopIcons}
          onLaunch={handleIconLaunch}
          onContextMenu={handleDesktopContextMenu}
          onMoveIcon={handleIconMove}
        />
      </div>

      {/* Window manager */}
      <WindowManager
        windows={windows}
        installedApps={installedApps}
        runningApps={runningApps}
        onFocus={handleWindowFocus}
        onClose={handleWindowClose}
        onMinimize={handleWindowMinimize}
        onMaximize={handleWindowMaximize}
        onMove={handleWindowMove}
        onResize={handleWindowResize}
      />

      {/* Taskbar */}
      <Taskbar
        pinnedApps={settings.taskbar.pinned_apps}
        installedApps={installedApps}
        runningApps={runningApps}
        windows={windows}
        activeWorkspaceId={activeWorkspaceId}
        workspaces={workspaces}
        notifications={notifications}
        clockFormat={settings.clock.format}
        connected={connected}
        onLauncherClick={() =>
          setActiveOverlay((prev) => (prev === "launcher" ? null : "launcher"))
        }
        onAppClick={handleTaskbarAppClick}
        onWorkspaceSwitch={handleWorkspaceSwitch}
        onNotificationsClick={() =>
          setActiveOverlay((prev) => (prev === "notifications" ? null : "notifications"))
        }
        onSettingsClick={() =>
          setActiveOverlay((prev) => (prev === "settings" ? null : "settings"))
        }
      />

      {/* App launcher overlay */}
      {activeOverlay === "launcher" && (
        <AppLauncher
          apps={installedApps}
          onLaunch={handleLaunchApp}
          onClose={() => setActiveOverlay(null)}
        />
      )}

      {/* Command palette */}
      {activeOverlay === "palette" && (
        <CommandPalette
          onClose={() => setActiveOverlay(null)}
          onLaunchApp={handleLaunchApp}
          onOpenSetting={() => setActiveOverlay("settings")}
          apps={installedApps}
        />
      )}

      {/* Notifications panel */}
      {activeOverlay === "notifications" && (
        <div
          className="notifications-overlay"
          role="dialog"
          aria-label="Notifications"
          onClick={(e) => {
            if (e.target === e.currentTarget) setActiveOverlay(null);
          }}
        >
          <div className="notifications-panel">
            <div className="notifications-panel__header">
              <h2 className="notifications-panel__title">Notifications</h2>
              {notifications.length > 0 && (
                <button
                  className="notifications-panel__action"
                  onClick={() => {
                    notifications.forEach((n) => {
                      if (!n.is_read)
                        api.markNotificationRead(n.notification_id).catch((err) => console.error("Failed to mark notification read:", err));
                    });
                    setNotifications((prev) => prev.map((n) => ({ ...n, is_read: true })));
                  }}
                >
                  Mark all read
                </button>
              )}
            </div>
            {notifications.length === 0 ? (
              <p className="notifications-empty">No notifications</p>
            ) : (
              <ul className="notifications-list">
                {notifications.map((n) => (
                  <li
                    key={n.notification_id}
                    className={`notification-item${n.is_read ? "" : " unread"}`}
                  >
                    <div className="notification-item__content">
                      <span className="notification-item__title">{n.title}</span>
                      <span className="notification-item__body">{n.body}</span>
                      <time className="notification-item__time">
                        {new Date(n.created_at).toLocaleString()}
                      </time>
                    </div>
                    <button
                      className="notification-item__dismiss"
                      aria-label={`Dismiss notification: ${n.title}`}
                      title="Dismiss"
                      onClick={() => {
                        api.dismissNotification(n.notification_id).catch((err) => console.error("Failed to dismiss notification:", err));
                        setNotifications((prev) =>
                          prev.filter((x) => x.notification_id !== n.notification_id),
                        );
                      }}
                    >
                      &#x2715;
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>
      )}

      {/* Settings panel */}
      {activeOverlay === "settings" && (
        <SettingsPanel
          settings={settings}
          onUpdateSettings={handleUpdateSetting}
          onClose={() => setActiveOverlay(null)}
        />
      )}

      {/* Context menu */}
      {contextMenu && <ContextMenu menu={contextMenu} onClose={() => setContextMenu(null)} />}

      {/* ARIA live region for screen reader announcements */}
      <div className="sr-only" aria-live="polite" aria-atomic="true">
        {ariaMessage}
      </div>
    </div>
  );
}
