# 07 - Desktop Shell

## 1. Purpose

The Desktop Shell is the primary user-facing surface of CortexOS. It is a browser-rendered desktop environment that serves as the user's entry point into the operating system. The Desktop Shell owns the desktop background, taskbar, application launcher, system tray, clock, notification center access, global command palette trigger, and desktop icons/shortcuts.

## 2. Scope

The Desktop Shell owns:

- Rendering the desktop background (solid color, gradient, or image).
- The taskbar: a persistent horizontal bar at the bottom of the viewport containing the app launcher button, running-app indicators, system tray icons, clock display, and notification center bell icon.
- The application launcher: a fullscreen or panel overlay that lists installed apps grouped by category, with search/filter capability.
- The system tray: a region of the taskbar showing status icons for network, volume, battery (if applicable), AI provider status, and other system-level indicators.
- The clock: a real-time clock widget in the taskbar showing local time and date.
- Notification center access: a bell icon or drawer trigger in the taskbar that opens the notification center (spec 13 owns the notification center panel itself; the shell owns the trigger point).
- Global command palette trigger: a keyboard shortcut (default `Ctrl+Space`) that opens the global command palette overlay (spec 12 owns the palette internals; the shell owns the trigger registration and visual overlay container).
- Desktop icons and shortcuts: user-placeable icons on the desktop background that launch apps, open files, or navigate to URLs within CortexOS.
- Right-click context menu on the desktop background (New File, New Folder, Change Background, Display Settings, Sort By, Refresh).
- Desktop workspace and virtual-desktop switcher widget in the taskbar (spec 08 owns virtual-desktop state; the shell owns the switcher UI widget).

This spec covers the frontend TypeScript application at `apps/desktop-shell`.

## 3. Out of Scope

- Window management, window chrome, minimize/maximize/restore animations (owned by spec 08 Window Manager).
- Notification center panel rendering and notification storage (owned by spec 13 Notifications Service).
- Global command palette search logic, indexing, and results ranking (owned by spec 12 Search/Indexing/Global Command Palette).
- Virtual filesystem operations (owned by spec 11 Virtual Filesystem).
- App installation and uninstallation (owned by spec 09 App Runtime).
- Permission prompting dialogs (owned by spec 04 Permissions/Policy).
- Theme engine and design token resolution (owned by spec 16 Theme/Design Tokens). The Desktop Shell consumes resolved theme tokens but does not define them.
- AI assistant panel (owned by spec 19 AI System Surfaces).

## 4. Objectives

1. Provide a responsive, visually consistent desktop environment that renders at 60 fps on modern browsers (Chrome 120+, Firefox 120+, Safari 17+, Edge 120+).
2. Serve as the single entry point for all user interaction with CortexOS.
3. Expose all top-level OS navigation affordances (app launcher, taskbar, command palette trigger, desktop icons).
4. Persist user customizations (desktop icon positions, background choice, taskbar preferences) across sessions via server-side state.
5. Support keyboard-driven workflow: every shell action must be achievable via keyboard alone.
6. Render correctly at viewport widths from 1024px to 3840px.

## 5. User-Visible Behavior

### 5.1 Desktop Background

- The desktop background fills the entire viewport behind all other layers.
- Default background is a solid dark color defined by the active theme's `desktop.background` token.
- Users can change the background via right-click context menu -> "Change Background" which opens the Settings app to the Background section.
- The background can be a solid color, a linear gradient (two-color, user-selectable direction), or an image from the virtual filesystem.
- Background image is scaled using `object-fit: cover` semantics and centered.
- Background setting is persisted to `shell.desktop.background` in the settings service.

### 5.2 Taskbar

- The taskbar is a fixed-position bar at the bottom of the viewport, 48px tall (configurable to 40px or 56px via `shell.taskbar.height`).
- The taskbar has three zones, left to right:
  - **Left zone**: App Launcher button (cortexOS logo icon, 40x40px), followed by pinned and running app icons.
  - **Center zone**: Flexible spacer (empty).
  - **Right zone**: System tray icons, virtual-desktop indicator, notification bell, clock.
- Taskbar is always on top (z-index: 9999 relative to the shell layer).
- Taskbar auto-hide is off by default. When enabled via `shell.taskbar.auto_hide`, the taskbar slides down below the viewport edge and reappears on cursor hover within 8px of the bottom edge or on `Meta` key press.
- Running apps show a dot indicator below their icon. The currently focused app shows a highlighted bar below its icon.
- Clicking a running app icon focuses that app's window (delegates to window manager via command bus).
- Right-clicking a running app icon shows a context menu: "Focus Window", "Close", "Pin to Taskbar" (if not pinned), "Unpin from Taskbar" (if pinned).

### 5.3 App Launcher

- Clicking the app launcher button or pressing the `Meta` (Windows/Command) key opens the app launcher overlay.
- The launcher overlay covers the full viewport with a semi-transparent backdrop (rgba(0,0,0,0.5)).
- The launcher contains a search field at the top, category tabs below it, and a grid of app icons below that.
- Categories are derived from the `category` field in each app's manifest. Default categories: "All", "Utilities", "Productivity", "Media", "Games", "System".
- The search field filters apps by name and description (fuzzy match, case-insensitive).
- Clicking an app icon or pressing `Enter` on a highlighted app closes the launcher and launches the app.
- Pressing `Escape` or clicking the backdrop closes the launcher without launching anything.
- The launcher animates in with a 200ms ease-out fade+scale transition and animates out with a 150ms ease-in transition.

### 5.4 System Tray

- The system tray displays status icons, each 20x20px, with 8px spacing between icons.
- Default icons (left to right): Network status, Volume, AI provider status, Battery (only on platforms reporting battery).
- Hovering a tray icon shows a tooltip with the current status text.
- Clicking a tray icon opens a small dropdown panel with controls relevant to that icon:
  - Network: connected/disconnected status, SSID display (read-only).
  - Volume: mute toggle, volume slider (0-100).
  - AI provider: current provider name, model name, connection status indicator.
  - Battery: percentage, charging status.
- The system tray icon set is extensible: apps can register tray icons via the command bus (spec 10). The Desktop Shell renders them in registration order after the default icons.

### 5.5 Clock

- The clock displays local time in `HH:MM` format (24-hour by default, configurable to 12-hour via `shell.clock.format`).
- Hovering the clock shows the full date: "Day, Month DD, YYYY".
- Clicking the clock opens a calendar dropdown showing the current month with today highlighted. The calendar is read-only (no event integration in v1).
- The clock updates every second. Time is sourced from the browser's `Date` object, synchronized with the server time on shell startup to correct for local clock skew.

### 5.6 Notification Center Access

- A bell icon in the taskbar right zone.
- A red badge with a count appears on the bell when unread notifications exist.
- Clicking the bell icon opens the notification center panel (spec 13 provides the panel content; the shell provides the trigger and the panel container slot).

### 5.7 Global Command Palette Trigger

- Pressing `Ctrl+Space` (configurable via `shell.command_palette.shortcut`) opens the command palette overlay.
- The shell captures the keyboard shortcut at the document level and sends a `ui.command_palette.toggle` command on the command bus.
- The shell provides the overlay container (full viewport, centered panel, semi-transparent backdrop). The command palette component from spec 12 renders inside this container.
- Pressing `Escape` or `Ctrl+Space` again closes the palette.

### 5.8 Desktop Icons

- The desktop supports a grid of icons placed on the background layer.
- Default icons: "File Manager", "Terminal", "Settings", "Trash".
- Icons are arranged in a grid with 96px cell size (icon 48x48 + label). Grid snaps automatically.
- Users can drag icons to reposition them. Position snaps to the nearest grid cell.
- Double-clicking an icon launches the associated app or opens the associated file.
- Right-clicking the desktop background shows a context menu with: "New Shortcut", "New Folder", "Sort Icons By" (Name, Date), "Change Background", "Display Settings", "Refresh".
- Right-clicking an icon shows: "Open", "Rename", "Delete Shortcut", "Properties".
- Icon positions are persisted to `shell.desktop.icon_layout` in the settings service.

## 6. System Behavior

### 6.1 Startup Sequence

1. Browser loads the Desktop Shell HTML entry point.
2. Shell initializes the command bus client connection (WebSocket to backend).
3. Shell requests `shell.*` settings from the settings service.
4. Shell requests the list of installed apps from the app runtime service (via command bus).
5. Shell renders the desktop background, taskbar, system tray, clock, and desktop icons.
6. Shell registers keyboard shortcuts (`Meta` for launcher, `Ctrl+Space` for command palette).
7. Shell emits `shell.started` event on the command bus.
8. Shell enters the main event loop, listening for command bus events.

### 6.2 Shutdown Sequence

1. Shell receives `os.shutdown.initiated` event.
2. Shell saves current icon layout and taskbar state to the settings service.
3. Shell emits `shell.stopped` event.
4. Shell renders a "Shutting down..." overlay and disables all interaction.

### 6.3 Desktop Icon Storage

- Each desktop icon is represented by a `DesktopIcon` data structure persisted via the settings service under the key `shell.desktop.icons`.
- When the user creates a shortcut, the shell writes a new `DesktopIcon` entry.
- When the user deletes a shortcut, the shell removes the entry.
- The "Trash" icon is always present and cannot be deleted.

### 6.4 Taskbar Pinning

- Pinned apps are stored in `shell.taskbar.pinned_apps` as an ordered list of app IDs.
- When an app is pinned, its ID is appended to the list.
- When an app is unpinned, its ID is removed from the list.
- The taskbar left zone renders pinned apps first, then running (non-pinned) apps.

### 6.5 Virtual Desktop Switcher

- The taskbar right zone includes a virtual desktop indicator showing the current desktop number and total count (e.g., "1/3").
- Clicking the indicator opens a small dropdown showing all virtual desktops by name.
- Selecting a desktop switches to it via the window manager command `wm.workspace.activate`.
- The shell subscribes to `wm.workspace.changed` events to update the indicator.

## 7. Architecture

### 7.1 Component Hierarchy

```
DesktopShell (root)
├── DesktopBackground
├── DesktopIconGrid
│   └── DesktopIcon (per icon)
├── Taskbar
│   ├── AppLauncherButton
│   ├── TaskbarAppList
│   │   └── TaskbarAppIcon (per app)
│   ├── SystemTray
│   │   └── TrayIcon (per icon, extensible)
│   ├── VirtualDesktopIndicator
│   ├── NotificationBell
│   └── Clock
├── AppLauncherOverlay
│   ├── SearchField
│   ├── CategoryTabs
│   └── AppGrid
│       └── AppLauncherIcon (per app)
├── CommandPaletteOverlay (container, spec 12 provides content)
├── NotificationCenterContainer (container, spec 13 provides content)
├── ContextMenu (singleton, repositioned per use)
└── CalendarDropdown (from clock click)
```

### 7.2 Rendering Architecture

- The Desktop Shell is a single-page TypeScript application using a reactive component framework (React or equivalent).
- The shell renders into a full-viewport `<div>` at z-index layer 0.
- The taskbar renders at z-index 9999.
- Overlay containers (launcher, palette, notification center) render at z-index 10000.
- Context menus render at z-index 10001.
- The shell does not render app window content. App windows render in iframes or sandboxed containers managed by the Window Manager (spec 08).
- CSS is scoped per component using CSS modules or equivalent. No global CSS bleed.

### 7.3 Communication

- All communication between the Desktop Shell and backend services uses the command bus (spec 10).
- The shell subscribes to events: `app.launched`, `app.stopped`, `app.crashed`, `wm.focus.changed`, `wm.workspace.changed`, `notification.created`, `notification.dismissed`, `notification.read`, `notification.all_read`, `settings.changed`, `os.shutdown.initiated`, `tray.icon.registered`, `tray.icon.unregistered`.
- The shell sends commands: `app.launch`, `app.list`, `settings.get`, `settings.set`, `wm.focus`, `wm.workspace.switch`, `command_palette.toggle`.

## 8. Data Model

### 8.1 DesktopIcon

```typescript
interface DesktopIcon {
  id: string;             // UUID v4
  app_id: string | null;  // app manifest id, null if file shortcut
  file_path: string | null; // virtual filesystem path, null if app shortcut
  label: string;           // display name
  icon_url: string;        // icon asset URL
  position: GridPosition;  // grid cell position
  created_at: string;      // ISO 8601 datetime
}
```

### 8.2 GridPosition

```typescript
interface GridPosition {
  row: number;    // 0-indexed from top
  column: number; // 0-indexed from left
}
```

### 8.3 TaskbarState

```typescript
interface TaskbarState {
  pinned_apps: string[];      // ordered list of app IDs
  auto_hide: boolean;         // default: false
  height: 40 | 48 | 56;      // pixels, default: 48
  running_apps: RunningAppInfo[];
}

interface RunningAppInfo {
  app_id: string;
  instance_id: string;
  window_count: number;
  has_focus: boolean;
}
```

### 8.4 ShellSettings

```typescript
interface ShellSettings {
  desktop: {
    background: BackgroundConfig;
    icon_layout: DesktopIcon[];
    grid_size: number; // cell size in px, default: 96
  };
  taskbar: {
    pinned_apps: string[];
    auto_hide: boolean;
    height: 40 | 48 | 56;
  };
  clock: {
    format: "12h" | "24h";
  };
  command_palette: {
    shortcut: string; // default: "Ctrl+Space"
  };
}

type BackgroundConfig =
  | { type: "solid"; color: string }          // CSS color
  | { type: "gradient"; from: string; to: string; direction: string }
  | { type: "image"; path: string };          // virtual filesystem path
```

### 8.5 TrayIconRegistration

```typescript
interface TrayIconRegistration {
  id: string;               // unique identifier
  app_id: string;           // registering app's manifest id
  icon_url: string;         // 20x20px icon asset URL
  tooltip: string;          // status text for hover
  panel_component: string;  // component id to render in dropdown, or null
}
```

## 9. Public Interfaces

### 9.1 Command Bus Commands Emitted by Shell

| Command | Payload | Response |
|---------|---------|----------|
| `app.launch` | `{ app_id: string, args?: Record<string, string> }` | `{ instance_id: string }` |
| `app.list` | `{}` | `{ apps: AppManifest[] }` |
| `settings.get` | `{ namespace: "shell", keys: string[] }` | `{ values: Record<string, unknown> }` |
| `settings.set` | `{ namespace: "shell", values: Record<string, unknown> }` `{}` |
| `wm.focus` | `{ window_id: string }` | `{}` |
| `wm.workspace.switch` | `{ workspace_id: string }` | `{}` |
| `ui.command_palette.toggle` | `{}` | `{}` |
| `shortcut.launcher.open` | `{}` | `{}` |
| `shortcut.launcher.close` | `{}` | `{}` |

### 9.2 Command Bus Events Subscribed by Shell

| Event | Payload |
|-------|---------|
| `app.launched` | `{ app_id: string, instance_id: string }` |
| `app.stopped` | `{ app_id: string, instance_id: string }` |
| `app.crashed` | `{ app_id: string, instance_id: string, error: string }` |
| `wm.focus.changed` | `{ window_id: string, app_id: string }` |
| `wm.workspace.changed` | `{ workspace_id: string, workspace_index: number, total_workspaces: number }` |
| `notification.created` | `{ notification_id: string, app_id: string, title: string }` |
| `notification.dismissed` | `{ notification_id: string }` |
| `notification.read` | `{ notification_id: string }` |
| `notification.all_read` | `{}` |
| `settings.changed` | `{ namespace: string, keys: string[] }` |
| `os.shutdown.initiated` | `{}` |
| `tray.icon.registered` | `TrayIconRegistration` |
| `tray.icon.unregistered` | `{ id: string }` |

### 9.3 Settings Namespace

The shell owns the `shell` settings namespace. All shell settings keys are prefixed with `shell.`. See section 8.4 for the full schema.

## 10. Internal Interfaces

### 10.1 ShellController (internal singleton)

```typescript
interface ShellController {
  initialize(): Promise<void>;
  getInstalledApps(): AppManifest[];
  getTaskbarState(): TaskbarState;
  getDesktopIcons(): DesktopIcon[];
  getShellSettings(): ShellSettings;
  updateSetting(key: string, value: unknown): Promise<void>;
  launchApp(appId: string, args?: Record<string, string>): Promise<string>;
  focusWindow(windowId: string): Promise<void>;
  openLauncher(): void;
  closeLauncher(): void;
  toggleCommandPalette(): void;
  openNotificationCenter(): void;
  shutdown(): Promise<void>;
}
```

### 10.2 IconLayoutManager (internal)

```typescript
interface IconLayoutManager {
  getIcons(): DesktopIcon[];
  addIcon(icon: Omit<DesktopIcon, "id" | "created_at">): DesktopIcon;
  removeIcon(id: string): void;
  moveIcon(id: string, position: GridPosition): void;
  findNearestEmptyCell(preferred: GridPosition): GridPosition;
  sortIcons(by: "name" | "created_at"): void;
  persist(): Promise<void>;
}
```

### 10.3 TrayManager (internal)

```typescript
interface TrayManager {
  getDefaultIcons(): TrayIconRegistration[];
  getRegisteredIcons(): TrayIconRegistration[];
  getAllIcons(): TrayIconRegistration[];
  handleIconClick(iconId: string): void;
}
```

## 11. State Management

### 11.1 Client-Side State

The shell maintains the following state in browser memory:

- `TaskbarState`: current running apps, focus state, pinned apps.
- `DesktopIcon[]`: current icon layout.
- `ShellSettings`: resolved settings (synced from settings service).
- `launcherOpen: boolean`: whether the app launcher overlay is visible.
- `commandPaletteOpen: boolean`: whether the command palette overlay is visible.
- `notificationCenterOpen: boolean`: whether the notification center is visible.
- `contextMenu: ContextMenuState | null`: current context menu position and items.
- `activeCalendarDropdown: boolean`: whether the clock calendar dropdown is open.

### 11.2 Server-Side State

The following state is persisted server-side via the settings service:

- `shell.desktop.background`: BackgroundConfig.
- `shell.desktop.icon_layout`: DesktopIcon[] (positions and associations).
- `shell.taskbar.pinned_apps`: string[] (ordered app IDs).
- `shell.taskbar.auto_hide`: boolean.
- `shell.taskbar.height`: 40 | 48 | 56.
- `shell.clock.format`: "12h" | "24h".
- `shell.command_palette.shortcut`: string.

### 11.3 State Synchronization

- On startup, the shell fetches all `shell.*` settings from the settings service.
- On any local state change, the shell persists the change to the settings service.
- The shell subscribes to `settings.changed` events for namespace `shell` and reconciles client state with server state. Server state is authoritative.
- If the settings service is unreachable on startup, the shell uses hardcoded defaults and retries every 5 seconds for up to 30 seconds, then proceeds with defaults and logs a warning.

### 11.4 Invariants

- INV-07-1: The taskbar is always rendered when the shell is active, even if auto-hide is enabled (it is merely off-screen, not unmounted).
- INV-07-2: No two desktop icons occupy the same grid cell.
- INV-07-3: The "Trash" icon is always present on the desktop and cannot be deleted by the user.
- INV-07-4: Only one overlay (launcher, command palette, or notification center) is visible at a time. Opening one closes others.
- INV-07-5: All shell settings changes are persisted to the server before the local state is considered committed.
- INV-07-6: The shell never directly accesses the virtual filesystem or policy engine. All access goes through command bus commands.

## 12. Failure Modes and Error Handling

### 12.1 Command Bus Connection Loss

- **Detection**: WebSocket connection closed or heartbeat timeout (5 seconds with no response).
- **Behavior**: The shell displays a non-dismissable banner at the top of the screen: "Connection lost. Attempting to reconnect..."
- **Recovery**: Automatic reconnection with exponential backoff (1s, 2s, 4s, 8s, max 30s). On reconnect, the shell re-fetches all state and resubscribes to events.
- **User action**: None required. The banner dismisses automatically on reconnect.

### 12.2 Settings Service Unreachable

- **Detection**: `settings.get` or `settings.set` command returns an error or times out (10 seconds).
- **Behavior**: The shell uses hardcoded defaults for the unavailable settings. A subtle warning icon appears in the system tray.
- **Recovery**: Retry settings fetch every 5 seconds for 30 seconds on startup. Subsequent failures are retried every 30 seconds.
- **User action**: The user can continue using the shell with defaults. Customizations made during outage are queued and persisted when the service recovers.

### 12.3 App Launch Failure

- **Detection**: `app.launch` command returns an error.
- **Behavior**: The shell shows a toast notification: "Could not launch [App Name]: [error message]."
- **Recovery**: The user can retry by clicking the icon again. The shell does not auto-retry.
- **Logging**: The error is logged to the observability service via `log.error`.

### 12.4 Clock Sync Failure

- **Detection**: Server time request fails or returns a value more than 60 seconds different from local time after correction.
- **Behavior**: The shell falls back to local browser time.
- **Recovery**: Retry server time sync every 60 seconds.

### 12.5 Rendering Performance Degradation

- **Detection**: Frame rate drops below 30 fps for 3 consecutive seconds (measured via `requestAnimationFrame` timestamps).
- **Behavior**: The shell disables non-essential animations (launcher transitions, icon hover effects) until frame rate recovers.
- **Recovery**: Automatic. Animations re-enable after 5 consecutive seconds above 45 fps.

## 13. Security and Permissions

### 13.1 Principle

The Desktop Shell runs as a trusted first-party component. It does not receive elevated privileges beyond what any other app receives for user-facing actions, but it does have implicit access to shell-specific settings and rendering controls.

### 13.2 Specific Rules

- SEC-07-1: The shell cannot directly read or write files. It uses the virtual filesystem service via the command bus.
- SEC-07-2: The shell cannot bypass permission prompts. If an app needs permissions, the permission dialog (spec 04) is rendered by the shell as a modal, but the decision is enforced server-side.
- SEC-07-3: The shell does not store credentials or tokens. All authentication is handled by the identity service (spec 03).
- SEC-07-4: Keyboard shortcut registration is first-come-first-served. The shell registers its shortcuts on startup. Apps cannot override `Meta` or `Ctrl+Space` if the shell has already claimed them.
- SEC-07-5: The shell validates all `TrayIconRegistration` payloads. Icons with missing required fields or invalid URLs are rejected and logged.
- SEC-07-6: The shell renders app content in sandboxed containers (iframes with sandbox attribute or equivalent). The shell cannot inject scripts into app contexts.

## 14. Performance Requirements

| Metric | Requirement |
|--------|-------------|
| Shell initial render (time to interactive) | Less than 1.5 seconds on a broadband connection |
| Taskbar render after startup | Less than 200ms |
| App launcher open animation | 200ms total duration |
| App launcher close animation | 150ms total duration |
| Desktop icon drag latency | Less than 16ms per frame (60 fps) |
| Clock update interval | 1 second |
| Context menu appear | Less than 100ms after click |
| Settings change to visual update | Less than 100ms |
| Memory usage (shell alone, idle) | Less than 50 MB |
| Maximum desktop icons | 100 (hard limit, enforced) |
| WebSocket message processing latency | Less than 5ms per message |

## 15. Accessibility Requirements

- ACC-07-1: All shell elements are navigable via keyboard (Tab, Arrow keys, Enter, Escape).
- ACC-07-2: All interactive elements have visible focus indicators (2px outline, theme-defined color).
- ACC-07-3: All icons have `aria-label` attributes with descriptive text.
- ACC-07-4: The taskbar supports `role="menubar"` semantics. Taskbar items support `role="menuitem"`.
- ACC-07-5: The app launcher overlay traps focus when open. Tab cycles within the launcher. Escape closes it.
- ACC-07-6: The clock meets WCAG 2.1 AA contrast ratio (minimum 4.5:1 for text on background).
- ACC-07-7: Context menus support arrow-key navigation. Selected item is visually distinct.
- ACC-07-8: Screen reader announcements for state changes (app launched, app crashed, connection lost) via ARIA live regions.
- ACC-07-9: Minimum touch target size for all interactive elements: 44x44px (WCAG 2.1 SC 2.5.5).
- ACC-07-10: System tray icon panels are dismissed by Escape key and by clicking outside.

## 16. Observability and Logging

### 16.1 Log Events

The shell emits the following structured log events to the observability service:

| Event | Level | Fields |
|-------|-------|--------|
| `shell.started` | info | `session_id`, `timestamp`, `settings_loaded: boolean` |
| `shell.stopped` | info | `session_id`, `timestamp` |
| `shell.app_launch_requested` | info | `app_id`, `source: "launcher" \| "desktop_icon" \| "taskbar"`, `timestamp` |
| `shell.app_launch_failed` | error | `app_id`, `error`, `timestamp` |
| `shell.settings_changed` | info | `key`, `timestamp` |
| `shell.connection_lost` | warn | `timestamp` |
| `shell.connection_restored` | info | `timestamp`, `downtime_ms` |
| `shell.performance_degraded` | warn | `fps`, `duration_ms`, `timestamp` |
| `shell.icon_added` | info | `icon_id`, `app_id`, `position`, `timestamp` |
| `shell.icon_removed` | info | `icon_id`, `timestamp` |
| `shell.shortcut_conflict` | warn | `shortcut`, `conflicting_app_id`, `timestamp` |

### 16.2 Metrics

The shell reports the following metrics:

- `shell.render_fps`: Gauge, updated every 5 seconds.
- `shell.memory_mb`: Gauge, updated every 30 seconds.
- `shell.launcher_open_count`: Counter, incremented per open.
- `shell.app_launch_count`: Counter, incremented per launch, labeled by `source`.
- `shell.connection_disconnects`: Counter.

## 17. Testing Requirements

### 17.1 Unit Tests

- IconLayoutManager: add, remove, move, findNearestEmptyCell, sort, collision prevention.
- GridPosition calculation for various viewport widths.
- BackgroundConfig validation (valid types, valid color strings, valid paths).
- TrayManager: registration, deduplication, default icons.
- Clock: format 12h/24h, date display.
- Context menu item generation for desktop and icons.

### 17.2 Integration Tests

- Shell startup sequence: verify all settings fetched, all events subscribed, all components rendered.
- App launch flow: click icon -> command sent -> response received -> running app reflected in taskbar.
- Settings change flow: change setting -> persist to server -> receive `settings.changed` event -> update UI.
- Command bus disconnect/reconnect: verify banner, verify state reconciliation after reconnect.
- Launcher open/close with keyboard and mouse.

### 17.3 End-to-End Tests

- Full desktop session: login, see desktop, launch app, switch virtual desktops, open command palette, receive notification, logout.
- Desktop icon drag-and-drop: drag icon to new position, verify persistence, reload page, verify position restored.
- Right-click context menu: verify all menu items present and functional.
- Taskbar auto-hide: enable, verify hide/show behavior on hover and keyboard.
- Accessibility: tab navigation through all taskbar elements, launcher keyboard navigation, screen reader announcements.

### 17.4 Performance Tests

- Measure shell time-to-interactive on simulated 10 Mbps connection. Must be under 1.5 seconds.
- Measure frame rate during icon drag with 100 icons. Must maintain 30+ fps.
- Measure memory after 1-hour idle session. Must remain under 75 MB.

## 18. Acceptance Criteria

- [ ] AC-07-1: Desktop background renders correctly with solid color, gradient, and image modes.
- [ ] AC-07-2: Taskbar displays all zones with correct content and responds to clicks.
- [ ] AC-07-3: App launcher opens on `Meta` key or button click, shows all installed apps, filters by search, and launches apps.
- [ ] AC-07-4: Clock displays correct time and updates every second.
- [ ] AC-07-5: Notification bell shows unread count badge.
- [ ] AC-07-6: `Ctrl+Space` opens command palette overlay.
- [ ] AC-07-7: Desktop icons are draggable, snap to grid, persist positions across reloads.
- [ ] AC-07-8: Right-click context menus appear with correct items for desktop background and icons.
- [ ] AC-07-9: System tray shows default icons and registered app icons.
- [ ] AC-07-10: All shell settings persist to server and restore on reload.
- [ ] AC-07-11: Connection loss banner appears and dismisses on reconnect.
- [ ] AC-07-12: All interactive elements have keyboard navigation and focus indicators.
- [ ] AC-07-13: Virtual desktop indicator reflects current desktop and allows switching.
- [ ] AC-07-14: Auto-hide taskbar slides down and reappears on hover/Meta key.
- [ ] AC-07-15: Shell renders correctly at 1024px and 3840px viewport widths.
- [ ] AC-07-16: No more than 100 desktop icons are allowed (enforced on add).
- [ ] AC-07-17: Only one overlay (launcher, palette, notification center) is visible at a time.

## 19. Build Order and Dependencies
**Layer 7**. Depends on: 01, 02, 05 (settings), 08 (window manager), 09 (app runtime), 10 (command bus), 16 (theme tokens)

### 19.1 Prerequisites

The Desktop Shell requires the following subsystems to be functional before it can be fully implemented:

1. **Spec 10 - System Command Bus**: Must be operational for all shell-to-backend communication.
2. **Spec 05 - Settings Service**: Must be operational for shell settings persistence.
3. **Spec 09 - App Runtime**: Must provide `app.list` and `app.launch` commands.
4. **Spec 02 - Core Architecture**: Defines the client-server communication pattern and API conventions.
5. **Spec 16 - Theme/Design Tokens**: Must provide resolved theme tokens for rendering.
6. **Spec 04 - Permissions**: Must provide permission prompt rendering.

### 19.2 Build Order

1. Build spec 10 (Command Bus) and spec 05 (Settings Service) first.
2. Build spec 16 (Theme) design token definitions.
3. Build Desktop Shell core structure: root component, taskbar layout, desktop background rendering.
4. Build clock, system tray (default icons only), and notification bell.
5. Build desktop icon grid with drag-and-drop.
6. Build app launcher overlay with search.
7. Build context menus.
8. Integrate command palette overlay container.
9. Integrate notification center container.
10. Build virtual desktop indicator.
11. Add accessibility attributes and keyboard navigation.
12. Add performance monitoring and connection loss handling.
13. Write unit, integration, and e2e tests.

### 19.3 What Can Be Stubbed Initially

- Command palette overlay content (can be an empty container until spec 12 is built).
- Notification center panel content (can be empty until spec 13 is built).
- Third-party tray icons (can show only default icons initially).
- Background image loading (can support solid/gradient only initially, add image later).

### 19.4 What Must Be Real in v1

- Taskbar with all three zones.
- App launcher with search and category filtering.
- Desktop icons with drag-and-drop and persistence.
- Clock with both 12h and 24h formats.
- System tray with default icons and tooltip panels.
- Keyboard shortcuts (`Meta`, `Ctrl+Space`, `Escape`).
- Context menus.
- Settings persistence.
- Connection loss handling.

## 20. Non-Goals and Anti-Patterns

### 20.1 Non-Goals

- Desktop widgets or gadgets (clock, weather, etc. as free-floating desktop items).
- Animated desktop backgrounds (video or particle effects).
- Multiple taskbars (top, side, or floating).
- Desktop file operations (copy, paste, move files directly on desktop). File operations go through the File Manager app.
- Desktop wallpaper slideshow or timed rotation.
- Drag-and-drop files between desktop icons and app windows.

### 20.2 Anti-Patterns

- AP-07-1: The shell must never directly call backend REST endpoints. All communication goes through the command bus.
- AP-07-2: The shell must never store security-sensitive data (passwords, tokens, API keys) in client-side state.
- AP-07-3: The shell must never render app content outside of sandboxed containers.
- AP-07-4: The shell must never make authorization decisions. It renders UI based on server-provided state only.
- AP-07-5: The shell must never use `window.alert()`, `window.confirm()`, or `window.prompt()`. All dialogs are custom components.
- AP-07-6: The shell must never directly modify DOM of app windows or other components outside its ownership.
- AP-07-7: The shell must never hardcode color values. All colors come from theme tokens.
- AP-07-8: The shell must not implement its own event system outside the command bus for inter-service communication.

## 21. Implementation Instructions for Claude Code / Codex

### 21.1 Subsystem Ownership

The Desktop Shell owns the browser-rendered desktop environment surface. It does not own window management, notification logic, search indexing, filesystem operations, or AI functionality.

### 21.2 Recommended Placement

- Frontend app: `apps/desktop-shell/`
- Entry point: `apps/desktop-shell/src/main.ts`
- Root component: `apps/desktop-shell/src/components/DesktopShell.tsx`
- Sub-components: `apps/desktop-shell/src/components/` (Taskbar, DesktopIconGrid, AppLauncher, etc.)
- State management: `apps/desktop-shell/src/state/`
- Command bus client: `apps/desktop-shell/src/bus/`
- Styles: `apps/desktop-shell/src/styles/` (CSS modules, consuming theme tokens)

### 21.3 What Can Be Stubbed

- Command palette overlay content: render an empty panel that receives a child component from spec 12.
- Notification center panel: render an empty container that receives a child component from spec 13.
- Third-party tray icon panels: render a "No panel registered" placeholder.
- Background image loading: implement solid and gradient, stub image loading with a TODO.

### 21.4 What Must Be Real in v1

- Taskbar rendering with all three zones (left/center/right).
- App launcher with real app list from `app.list` command and search filtering.
- Desktop icon grid with drag-and-drop using pointer events (not HTML5 drag API, for better control).
- Clock component with server time sync.
- System tray with default icons and dropdown panels.
- Context menus with all items described in section 5.8.
- Settings read/write via command bus.
- Keyboard shortcut registration and handling.
- Connection loss detection and banner.

### 21.5 What Cannot Be Inferred

- The exact theme token names (must consume from spec 16). If spec 16 is not yet implemented, use a placeholder token interface that can be replaced.
- The exact iframe sandboxing attributes for app windows (must align with spec 08 and spec 09).
- The exact `app.list` response schema (must align with spec 09 AppManifest).
- The exact `settings.get`/`settings.set` wire protocol (must align with spec 05 and spec 10).

### 21.6 Stop Conditions

The Desktop Shell subsystem is considered done when:

1. All acceptance criteria in section 18 pass.
2. Unit test coverage for IconLayoutManager, TrayManager, and clock formatting is at least 90%.
3. Integration tests for startup sequence, app launch flow, and settings sync pass.
4. E2E test for a full desktop session passes.
5. Accessibility audit (keyboard nav, ARIA attributes, focus management) shows zero violations.
6. Performance metrics meet the requirements in section 14.
7. No hardcoded colors remain (all consumed from theme tokens).
8. All command bus communication uses typed interfaces, no `any` types.

### 21.7 Testing Gates

Before marking the Desktop Shell as complete:

- Run `pnpm --filter desktop-shell test` and verify all unit tests pass.
- Run `pnpm --filter desktop-shell test:integration` and verify all integration tests pass.
- Run `pnpm run e2e --spec desktop-shell` and verify all e2e tests pass.
- Run `pnpm run lint --filter desktop-shell` and verify zero lint errors.
- Run `pnpm run a11y --filter desktop-shell` and verify zero accessibility violations.
