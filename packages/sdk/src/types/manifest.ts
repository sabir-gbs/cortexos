/**
 * TypeScript type definitions for the CortexOS app manifest schema.
 *
 * These types mirror the Rust manifest types in cortex-sdk and the
 * schema defined in SPEC 21.
 */

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/** Current manifest schema version. */
export const MANIFEST_SCHEMA_VERSION = '1.0.0' as const;

/** Maximum lengths enforced by the spec. */
export const MAX_MANIFEST_ID_LENGTH = 128;
export const MAX_APP_NAME_LENGTH = 64;
export const MAX_DESCRIPTION_LENGTH = 512;
export const MAX_PERMISSION_REASON_LENGTH = 256;
export const MAX_AUTHOR_NAME_LENGTH = 64;

/** Default max file size an app may create (100 MB). */
export const DEFAULT_MAX_FILE_SIZE = 104_857_600;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/**
 * Permission types that third-party apps can declare.
 *
 * Each maps to a specific capability gate in the SDK routing layer.
 */
export enum PermissionType {
  FilesRead = 'files.read',
  FilesWrite = 'files.write',
  Clipboard = 'clipboard',
  Notifications = 'notifications',
  Ai = 'ai',
  Settings = 'settings',
  Search = 'search',
  Network = 'network',
  HardwareCamera = 'hardware.camera',
  HardwareMicrophone = 'hardware.microphone',
  HardwareLocation = 'hardware.location',
}

/** All known permission type string values for validation. */
export const KNOWN_PERMISSIONS: readonly string[] = Object.values(PermissionType);

/**
 * App categories for store organisation.
 */
export enum AppCategory {
  Productivity = 'productivity',
  Communication = 'communication',
  Development = 'development',
  Media = 'media',
  Education = 'education',
  Finance = 'finance',
  Health = 'health',
  Games = 'games',
  Utilities = 'utilities',
  Other = 'other',
}

/**
 * Install source for an installed app.
 */
export enum InstallSource {
  AppStore = 'app_store',
  Sideload = 'sideload',
  System = 'system',
}

/**
 * Lifecycle state of an installed app.
 */
export enum AppState {
  Installed = 'installed',
  Running = 'running',
  Suspended = 'suspended',
  Stopped = 'stopped',
}

// ---------------------------------------------------------------------------
// Structured types
// ---------------------------------------------------------------------------

/** Information about the app author. */
export interface AuthorInfo {
  /** Author or organisation name. Maximum 64 characters. */
  name: string;
  /** Optional author email. */
  email?: string;
  /** Optional author website URL. */
  url?: string;
}

/** A single permission declaration in a manifest. */
export interface AppPermission {
  /** Permission name -- must be one of {@link PermissionType}. */
  permission: string;
  /** Whether the permission is mandatory for the app to function. */
  required: boolean;
  /**
   * Human-readable reason shown to the user during install.
   * Maximum 256 characters.
   */
  reason: string;
}

/** A file type handler the app can register. */
export interface FileHandler {
  /** File extension without leading dot (e.g. "txt"). */
  extension: string;
  /** MIME type (e.g. "text/plain"). */
  mime_type: string;
  /** Human-readable label (e.g. "Text Document"). */
  label: string;
}

/** Capability flags and limits declared in the manifest. */
export interface AppCapabilities {
  /** Whether the app uses AI features. Default: false. */
  ai: boolean;
  /** Whether the app needs background execution. Default: false. */
  background_execution: boolean;
  /** Whether the app should run at system startup. Default: false. */
  autostart: boolean;
  /**
   * Maximum file size the app may create in bytes.
   * 0 = unlimited. Default: 100 MB.
   */
  max_file_size: number;
  /** Whether the app provides a system tray icon. Default: false. */
  system_tray: boolean;
  /** File type associations the app can handle. Default: empty. */
  file_handlers: FileHandler[];
}

/** Window / viewport configuration for the app. */
export interface WindowConfig {
  /** Default window width in pixels. */
  width?: number;
  /** Default window height in pixels. */
  height?: number;
  /** Minimum window width in pixels. */
  min_width?: number;
  /** Minimum window height in pixels. */
  min_height?: number;
  /** Whether the window can be resized. Default: true. */
  resizable?: boolean;
  /** Window title override (defaults to app name). */
  title?: string;
}

/**
 * The full app manifest.
 *
 * This is the top-level document that describes a third-party CortexOS
 * application. It is validated at install time and stored in the app
 * registry.
 */
export interface AppManifest {
  /**
   * Unique identifier in reverse-domain format.
   * Pattern: ^[a-z0-9]+(\.[a-z0-9]+)*\.[a-z0-9-]+$
   * Maximum length: 128 characters.
   */
  id: string;

  /**
   * Human-readable app name.
   * Maximum length: 64 characters. Must not be empty.
   */
  name: string;

  /**
   * Semantic version string (MAJOR.MINOR.PATCH).
   * Pattern: ^\d+\.\d+\.\d+$
   */
  version: string;

  /**
   * Entry point: the main HTML file or WASM module to load.
   * Must reference a file within the package.
   */
  entry_point: string;

  /**
   * Human-readable description.
   * Maximum length: 512 characters.
   */
  description: string;

  /**
   * Path to the app icon within the package.
   * Must be PNG, SVG, or ICO. Recommended 128x128 px.
   */
  icon: string;

  /** Permissions requested by the app. */
  permissions: AppPermission[];

  /** Capabilities declared by the app. */
  capabilities: AppCapabilities;

  /**
   * Minimum OS version required.
   * Pattern: ^\d+\.\d+\.\d+$
   */
  min_os_version: string;

  /** Author information. */
  author: AuthorInfo;

  /** Optional category for app store organisation. */
  category?: AppCategory;

  /** Optional homepage URL. */
  homepage_url?: string;

  /** Optional support / contact URL or email. */
  support_url?: string;

  /** Optional license identifier (SPDX format preferred). */
  license?: string;

  /** Optional window configuration. */
  window?: WindowConfig;
}

/**
 * Record of an installed app stored in the registry.
 */
export interface InstalledApp {
  /** The validated manifest. */
  manifest: AppManifest;
  /** Absolute path to the extracted package on disk. */
  install_path: string;
  /** Absolute path to the app sandbox data directory. */
  data_path: string;
  /** ISO-8601 timestamp of when the app was installed. */
  installed_at: string;
  /** ISO-8601 timestamp of the last update. */
  updated_at: string;
  /** How the app was installed. */
  install_source: InstallSource;
  /** Permissions the user has granted for this app. */
  granted_permissions: string[];
  /** Current lifecycle state. */
  state: AppState;
}

// ---------------------------------------------------------------------------
// Validation result types
// ---------------------------------------------------------------------------

/** Severity of a validation issue. */
export enum ValidationSeverity {
  Error = 'error',
  Warning = 'warning',
}

/** A single validation error or warning. */
export interface ValidationError {
  /** Dot-path to the offending field (e.g. "id", "permissions[2].permission"). */
  field: string;
  /** Human-readable description of what is wrong. */
  message: string;
  /** Whether this blocks installation or is merely a warning. */
  severity: ValidationSeverity;
}

/** Result of validating an unknown value as an AppManifest. */
export interface ValidationResult {
  /** True when the manifest is fully valid (no errors; warnings may exist). */
  valid: boolean;
  /** Collected errors (blocks installation). */
  errors: ValidationError[];
  /** Collected warnings (logged but installation proceeds). */
  warnings: ValidationError[];
}
