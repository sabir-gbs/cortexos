import React from "react";
import type { AppManifest, AppInstance } from "../types";

interface TaskbarProps {
  pinnedApps: string[];
  installedApps: AppManifest[];
  runningApps: AppInstance[];
  windows: Array<{
    id: string;
    instance_id: string;
    title: string;
    focused: boolean;
    state: string;
  }>;
  activeWorkspaceId: string | null;
  workspaces: Array<{ id: string; name: string; active: boolean; index: number }>;
  notifications: Array<{ is_read: boolean }>;
  clockFormat: "12h" | "24h";
  connected: boolean;
  onLauncherClick: () => void;
  onAppClick: (instanceId: string) => void;
  onWorkspaceSwitch: (workspaceId: string) => void;
  onNotificationsClick: () => void;
  onSettingsClick: () => void;
}

function Clock({ format }: { format: "12h" | "24h" }) {
  const [now, setNow] = React.useState(() => new Date());

  React.useEffect(() => {
    const id = setInterval(() => setNow(new Date()), 30_000);
    return () => clearInterval(id);
  }, []);

  const timeStr = now.toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    hour12: format === "12h",
  });
  const dateStr = now.toLocaleDateString([], { month: "short", day: "numeric" });
  const fullDateStr = now.toLocaleDateString([], {
    weekday: "long",
    year: "numeric",
    month: "long",
    day: "numeric",
  });

  return (
    <span className="taskbar-clock" aria-label={fullDateStr} title={fullDateStr}>
      {dateStr} {timeStr}
    </span>
  );
}

export function Taskbar({
  pinnedApps,
  installedApps,
  runningApps,
  windows,
  workspaces,
  notifications,
  clockFormat,
  connected,
  onLauncherClick,
  onAppClick,
  onWorkspaceSwitch,
  onNotificationsClick,
  onSettingsClick,
}: TaskbarProps) {
  const appMap = new Map(installedApps.map((a) => [a.id, a]));

  // Build taskbar items: pinned + running (non-duplicate)
  const taskbarItems: Array<{
    appId: string;
    instanceId?: string;
    icon: string;
    name: string;
    isRunning: boolean;
    hasFocus: boolean;
  }> = [];

  for (const appId of pinnedApps) {
    const app = appMap.get(appId);
    const runningInstance = runningApps.find((r) => r.app_id === appId);
    const win = runningInstance
      ? windows.find((w) => w.instance_id === runningInstance.instance_id && w.state !== "closed")
      : undefined;
    taskbarItems.push({
      appId,
      instanceId: runningInstance?.instance_id,
      icon: app?.icon || "\uD83D\uDCC6",
      name: app?.name || appId,
      isRunning: !!runningInstance,
      hasFocus: win?.focused ?? false,
    });
  }

  // Add running apps not in pinned list
  for (const running of runningApps) {
    if (taskbarItems.some((t) => t.appId === running.app_id)) continue;
    const app = appMap.get(running.app_id);
    const win = windows.find((w) => w.instance_id === running.instance_id && w.state !== "closed");
    taskbarItems.push({
      appId: running.app_id,
      instanceId: running.instance_id,
      icon: app?.icon || "\uD83D\uDCC6",
      name: app?.name || running.app_id,
      isRunning: true,
      hasFocus: win?.focused ?? false,
    });
  }

  const unreadCount = notifications.filter((n) => !n.is_read).length;
  const activeWs = workspaces.find((w) => w.active) || workspaces[0];

  return (
    <nav className="taskbar" role="toolbar" aria-label="Taskbar">
      {/* Left zone: launcher + apps */}
      <div className="taskbar-start">
        <button
          className="taskbar-btn taskbar-btn--launcher"
          onClick={onLauncherClick}
          aria-label="App launcher"
          title="App launcher"
        >
          <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
            <rect x="1" y="1" width="8" height="8" rx="1.5" />
            <rect x="11" y="1" width="8" height="8" rx="1.5" />
            <rect x="1" y="11" width="8" height="8" rx="1.5" />
            <rect x="11" y="11" width="8" height="8" rx="1.5" />
          </svg>
        </button>

        {taskbarItems.map((item) => (
          <button
            key={item.appId}
            className={`taskbar-btn taskbar-app-btn${item.hasFocus ? " focused" : ""}`}
            onClick={() => {
              if (item.instanceId) onAppClick(item.instanceId);
              else onLauncherClick();
            }}
            aria-label={item.name}
            title={item.name}
          >
            <span className="taskbar-app-icon" aria-hidden="true">
              {item.icon}
            </span>
            {item.isRunning && <span className="taskbar-app-dot" />}
            {item.hasFocus && <span className="taskbar-app-bar" />}
          </button>
        ))}
      </div>

      {/* Center zone: spacer */}
      <div className="taskbar-center" />

      {/* Right zone: workspace, notifications, connection, clock, settings */}
      <div className="taskbar-end">
        {/* Workspace switcher */}
        {activeWs && (
          <button
            className="taskbar-btn taskbar-workspace-btn"
            onClick={() => {
              // Cycle to next workspace
              const currentIdx = workspaces.findIndex((w) => w.id === activeWs.id);
              const nextIdx = (currentIdx + 1) % workspaces.length;
              onWorkspaceSwitch(workspaces[nextIdx].id);
            }}
            aria-label={`Workspace ${activeWs.index + 1} of ${workspaces.length}: ${activeWs.name}`}
            title={`Workspace: ${activeWs.name}`}
          >
            {activeWs.index + 1}/{workspaces.length}
          </button>
        )}

        {/* Notification bell */}
        <button
          className="taskbar-btn taskbar-notification-btn"
          onClick={onNotificationsClick}
          aria-label={`Notifications${unreadCount > 0 ? `, ${unreadCount} unread` : ""}`}
          title="Notifications"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
            <path d="M8 1a4.5 4.5 0 0 0-4.5 4.5v2.67L2 11h12l-1.5-2.83V5.5A4.5 4.5 0 0 0 8 1zm0 14a2 2 0 0 1-2-2h4a2 2 0 0 1-2 2z" />
          </svg>
          {unreadCount > 0 && (
            <span className="taskbar-notification-badge" aria-hidden="true">
              {unreadCount > 9 ? "9+" : unreadCount}
            </span>
          )}
        </button>

        {/* Connection indicator */}
        {!connected && (
          <span className="taskbar-connection-icon" aria-label="Disconnected" title="Disconnected">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor" aria-hidden="true">
              <path d="M1.4 0L0 1.4l3.5 3.5A5 5 0 0 0 2 8H0v2h2.3A5 5 0 0 0 6 13.9V14h2v-.1A5 5 0 0 0 11.7 10H14V8h-2a5 5 0 0 0-1-3l2.6-2.6L12.2 1.2 1.4 0zM7 4a3 3 0 0 1 3 3H4a3 3 0 0 1 3-3z" />
            </svg>
          </span>
        )}

        <Clock format={clockFormat} />

        <button
          className="taskbar-btn"
          onClick={onSettingsClick}
          aria-label="Settings"
          title="Settings"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
            <path d="M8 5.5a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5zM6.1 1L5.5 3.3l-2 .8-2.1-1-.7.7 1.4 2-.6 2L0 8v1l2.2.6.6 2-1.4 2 .7.7 2.1-1 2 .8.5 2.2h1l.5-2.2 2-.8 2.1 1 .7-.7-1.4-2 .6-2L14 9V8l-2.2-.6-.6-2 1.4-2-.7-.7-2.1 1-2-.8L7 1h-.9z" />
          </svg>
        </button>
      </div>
    </nav>
  );
}
