/** TypeScript types for the CortexOS Desktop Shell. */

// ── Window Manager ──────────────────────────────────────────────────────────────

export interface WindowState {
  id: string;
  instance_id: string;
  user_id: string;
  workspace_id: string;
  title: string;
  state: "normal" | "minimized" | "maximized" | "closed";
  x: number;
  y: number;
  width: number;
  height: number;
  z_index: number;
  focused: boolean;
  created_at: string;
  updated_at: string;
}

export interface WorkspaceState {
  id: string;
  user_id: string;
  name: string;
  index: number;
  active: boolean;
  created_at: string;
}

// ── Desktop Icons ───────────────────────────────────────────────────────────────

export interface DesktopIcon {
  id: string;
  app_id: string | null;
  file_path: string | null;
  label: string;
  icon_url: string;
  position: GridPosition;
  created_at: string;
}

export interface GridPosition {
  row: number;
  column: number;
}

// ── Apps ─────────────────────────────────────────────────────────────────────────

export interface AppManifest {
  id: string;
  name: string;
  description: string;
  category: string;
  icon: string;
  version: string;
  entry_point?: string;
}

export interface AppInstance {
  instance_id: string;
  app_id: string;
  user_id: string;
  state: string;
  window_id: string | null;
  launched_at: string | null;
  stopped_at: string | null;
}

export interface RunningAppInfo {
  app_id: string;
  instance_id: string;
  window_count: number;
  has_focus: boolean;
}

// ── Notifications ───────────────────────────────────────────────────────────────

export interface NotificationInfo {
  notification_id: string;
  user_id: string;
  title: string;
  body: string;
  category: string;
  is_read: boolean;
  created_at: string;
  dismissed_at: string | null;
}

// ── Search ──────────────────────────────────────────────────────────────────────

export interface SearchResult {
  source_id: string;
  content_type: string;
  snippet: string;
  relevance: number;
}

// ── Settings ────────────────────────────────────────────────────────────────────

export interface ShellSettings {
  desktop: {
    background: BackgroundConfig;
    icons: DesktopIcon[];
    grid_size: number;
  };
  taskbar: {
    pinned_apps: string[];
    auto_hide: boolean;
    height: 40 | 48 | 56;
  };
  clock: {
    format: "12h" | "24h";
  };
  theme: {
    mode: "light" | "dark";
  };
}

export type BackgroundConfig =
  | { type: "solid"; color: string }
  | { type: "gradient"; from: string; to: string; direction: string }
  | { type: "image"; path: string };

// ── Context Menu ────────────────────────────────────────────────────────────────

export interface ContextMenuState {
  x: number;
  y: number;
  items: ContextMenuItem[];
}

export interface ContextMenuItem {
  id: string;
  label: string;
  icon?: string;
  disabled?: boolean;
  separator?: boolean;
  action?: () => void;
}

// ── Auth ────────────────────────────────────────────────────────────────────────

export interface LoginResponse {
  session_id: string;
  token: string;
  user_id: string;
  expires_at: string;
}

export interface ProfileResponse {
  user_id: string;
  username: string;
  display_name: string;
  created_at: string;
}

// ── Shell State ─────────────────────────────────────────────────────────────────

export type OverlayType = "launcher" | "palette" | "notifications" | "settings" | null;

// ── API ─────────────────────────────────────────────────────────────────────────

export interface ApiResponse<T> {
  data: T;
  meta?: unknown;
}

export interface ApiErrorResponse {
  error: {
    code: string;
    message: string;
    request_id?: string;
  };
}

// ── Theme ───────────────────────────────────────────────────────────────────────

export interface ThemeTokens {
  colors: {
    bg: string;
    bgSecondary: string;
    bgTertiary: string;
    bgSurface: string;
    text: string;
    textSecondary: string;
    textMuted: string;
    accent: string;
    accentHover: string;
    success: string;
    warning: string;
    error: string;
    border: string;
    borderLight: string;
  };
  spacing: {
    xs: number;
    sm: number;
    md: number;
    lg: number;
    xl: number;
  };
  borderRadius: {
    sm: number;
    md: number;
    lg: number;
  };
  fontSize: {
    xs: number;
    sm: number;
    md: number;
    lg: number;
    xl: number;
  };
}

export const DARK_THEME: ThemeTokens = {
  colors: {
    bg: "#1a1a2e",
    bgSecondary: "#16213e",
    bgTertiary: "#0f3460",
    bgSurface: "#222244",
    text: "#e0e0e0",
    textSecondary: "#a0a0a0",
    textMuted: "#606060",
    accent: "#4fc3f7",
    accentHover: "#29b6f6",
    success: "#66bb6a",
    warning: "#ffa726",
    error: "#ef5350",
    border: "#333355",
    borderLight: "#2a2a44",
  },
  spacing: { xs: 4, sm: 8, md: 16, lg: 24, xl: 32 },
  borderRadius: { sm: 4, md: 8, lg: 12 },
  fontSize: { xs: 11, sm: 13, md: 14, lg: 16, xl: 20 },
};

export const LIGHT_THEME: ThemeTokens = {
  colors: {
    bg: "#f0f0f5",
    bgSecondary: "#e8e8f0",
    bgTertiary: "#d0d0e0",
    bgSurface: "#ffffff",
    text: "#1a1a2e",
    textSecondary: "#555577",
    textMuted: "#888899",
    accent: "#1976d2",
    accentHover: "#1565c0",
    success: "#388e3c",
    warning: "#f57c00",
    error: "#d32f2f",
    border: "#ccccdd",
    borderLight: "#ddddee",
  },
  spacing: { xs: 4, sm: 8, md: 16, lg: 24, xl: 32 },
  borderRadius: { sm: 4, md: 8, lg: 12 },
  fontSize: { xs: 11, sm: 13, md: 14, lg: 16, xl: 20 },
};

// ── Defaults ────────────────────────────────────────────────────────────────────

export const DEFAULT_SETTINGS: ShellSettings = {
  desktop: {
    background: { type: "solid", color: "#1a1a2e" },
    icons: [
      {
        id: "icon-files",
        app_id: "file-manager",
        file_path: null,
        label: "Files",
        icon_url: "\uD83D\uDCC1",
        position: { row: 0, column: 0 },
        created_at: "",
      },
      {
        id: "icon-terminal",
        app_id: "terminal-lite",
        file_path: null,
        label: "Terminal",
        icon_url: "\u25B6\uFE0F",
        position: { row: 1, column: 0 },
        created_at: "",
      },
      {
        id: "icon-settings",
        app_id: "settings-app",
        file_path: null,
        label: "Settings",
        icon_url: "\u2699\uFE0F",
        position: { row: 2, column: 0 },
        created_at: "",
      },
      {
        id: "icon-trash",
        app_id: null,
        file_path: "/tmp/trash",
        label: "Trash",
        icon_url: "\uD83D\uDDD1\uFE0F",
        position: { row: 3, column: 0 },
        created_at: "",
      },
    ],
    grid_size: 96,
  },
  taskbar: {
    pinned_apps: ["file-manager", "terminal-lite", "settings-app", "calculator", "notes"],
    auto_hide: false,
    height: 48,
  },
  clock: {
    format: "24h",
  },
  theme: {
    mode: "dark",
  },
};

export const BUILTIN_APPS: AppManifest[] = [
  {
    id: "calculator",
    name: "Calculator",
    description: "Basic calculator",
    category: "Utilities",
    icon: "\uD83E\uDDEE",
    version: "1.0.0",
    entry_point: "/apps/calculator/index.html",
  },
  {
    id: "text-editor",
    name: "Text Editor",
    description: "Edit text files",
    category: "Productivity",
    icon: "\uD83D\uDCDD",
    version: "1.0.0",
    entry_point: "/apps/text-editor/index.html",
  },
  {
    id: "notes",
    name: "Notes",
    description: "Take notes",
    category: "Productivity",
    icon: "\uD83D\uDCD3",
    version: "1.0.0",
    entry_point: "/apps/notes/index.html",
  },
  {
    id: "file-manager",
    name: "Files",
    description: "Browse files",
    category: "System",
    icon: "\uD83D\uDCC1",
    version: "1.0.0",
    entry_point: "/apps/file-manager/index.html",
  },
  {
    id: "media-viewer",
    name: "Media Viewer",
    description: "View images",
    category: "Media",
    icon: "\uD83C\uDF5B",
    version: "1.0.0",
    entry_point: "/apps/media-viewer/index.html",
  },
  {
    id: "terminal-lite",
    name: "Terminal",
    description: "Command line",
    category: "System",
    icon: "\u25B6\uFE0F",
    version: "1.0.0",
    entry_point: "/apps/terminal-lite/index.html",
  },
  {
    id: "clock-utils",
    name: "Clock",
    description: "Clock and timer",
    category: "Utilities",
    icon: "\u23F0",
    version: "1.0.0",
    entry_point: "/apps/clock-utils/index.html",
  },
  {
    id: "settings-app",
    name: "Settings",
    description: "System settings",
    category: "System",
    icon: "\u2699\uFE0F",
    version: "1.0.0",
    entry_point: "/apps/settings-app/index.html",
  },
  {
    id: "solitaire",
    name: "Solitaire",
    description: "Klondike solitaire",
    category: "Games",
    icon: "\uD83C\uDCCF",
    version: "1.0.0",
    entry_point: "/apps/solitaire/index.html",
  },
  {
    id: "minesweeper",
    name: "Minesweeper",
    description: "Classic minesweeper",
    category: "Games",
    icon: "\uD83D\uDCA3",
    version: "1.0.0",
    entry_point: "/apps/minesweeper/index.html",
  },
  {
    id: "snake",
    name: "Snake",
    description: "Classic snake game",
    category: "Games",
    icon: "\uD83D\uDC0D",
    version: "1.0.0",
    entry_point: "/apps/snake/index.html",
  },
  {
    id: "tetris",
    name: "Tetris",
    description: "Block puzzle game",
    category: "Games",
    icon: "\uD83E\uDDE9",
    version: "1.0.0",
    entry_point: "/apps/tetris/index.html",
  },
  {
    id: "chess",
    name: "Chess",
    description: "Chess game",
    category: "Games",
    icon: "\u265F\uFE0F",
    version: "1.0.0",
    entry_point: "/apps/chess/index.html",
  },
];

export const APP_CATEGORIES = [
  "All",
  "Utilities",
  "Productivity",
  "Media",
  "System",
  "Games",
] as const;
