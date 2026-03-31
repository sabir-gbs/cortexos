/**
 * In-memory app registry for CortexOS.
 *
 * The registry is the single source of truth for which apps are installed,
 * their manifests, and their current lifecycle state. In the full system
 * this is backed by a persistent store; this module provides the in-memory
 * layer used by the TypeScript SDK for validation and lookups.
 */

import {
  AppState,
  InstallSource,
} from '../types/manifest';
import type {
  AppManifest,
  InstalledApp,
} from '../types/manifest';
import { validateManifest } from '../validators/manifest-validator';

/**
 * Thrown when an app registry operation violates an invariant.
 */
export class RegistryError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'RegistryError';
  }
}

/**
 * In-memory app registry.
 *
 * Stores manifests and metadata for installed apps. Provides lookup,
 * listing, and lifecycle state management. All mutations are synchronous
 * and in-memory -- persistence is the caller's responsibility.
 */
export class AppRegistry {
  private apps = new Map<string, InstalledApp>();

  /**
   * Register (install) a new app.
   *
   * The manifest is validated before insertion. If validation fails,
   * a {@link RegistryError} is thrown with the validation errors.
   * If an app with the same `id` already exists, a {@link RegistryError}
   * is thrown (use `updateApp` to replace).
   *
   * @param manifest - The validated app manifest.
   * @param options  - Optional metadata for the registry entry.
   */
  registerApp(
    manifest: AppManifest,
    options?: {
      installPath?: string;
      dataPath?: string;
      installSource?: InstallSource;
      grantedPermissions?: string[];
    },
  ): void {
    const result = validateManifest(manifest);
    if (!result.valid) {
      const details = result.errors
        .map((e) => `${e.field}: ${e.message}`)
        .join('; ');
      throw new RegistryError(
        `Manifest validation failed for "${manifest.id ?? '<unknown>'}": ${details}`,
      );
    }

    if (this.apps.has(manifest.id)) {
      throw new RegistryError(
        `App with id "${manifest.id}" is already installed`,
      );
    }

    const now = new Date().toISOString();
    const app: InstalledApp = {
      manifest,
      install_path: options?.installPath ?? '',
      data_path: options?.dataPath ?? '',
      installed_at: now,
      updated_at: now,
      install_source: options?.installSource ?? InstallSource.Sideload,
      granted_permissions: options?.grantedPermissions ?? [],
      state: AppState.Installed,
    };

    this.apps.set(manifest.id, app);
  }

  /**
   * Unregister (uninstall) an app by ID.
   *
   * @throws {RegistryError} if no app with the given ID exists.
   */
  unregisterApp(id: string): void {
    if (!this.apps.has(id)) {
      throw new RegistryError(`App "${id}" is not installed`);
    }
    this.apps.delete(id);
  }

  /**
   * Retrieve a single installed app by ID.
   *
   * Returns `undefined` if no such app is installed.
   */
  getApp(id: string): InstalledApp | undefined {
    return this.apps.get(id);
  }

  /**
   * List all installed apps.
   *
   * Returns a shallow copy of the registry contents.
   */
  listApps(): InstalledApp[] {
    return Array.from(this.apps.values());
  }

  /**
   * Check whether an app with the given ID is installed.
   */
  isInstalled(id: string): boolean {
    return this.apps.has(id);
  }

  /**
   * Update the state of an installed app.
   *
   * @throws {RegistryError} if no app with the given ID exists.
   */
  setAppState(id: string, state: AppState): void {
    const app = this.apps.get(id);
    if (!app) {
      throw new RegistryError(`App "${id}" is not installed`);
    }
    app.state = state;
    app.updated_at = new Date().toISOString();
  }

  /**
   * Replace the manifest of an installed app (used during updates).
   *
   * The new manifest must pass validation. The app's `updated_at`
   * timestamp is refreshed.
   *
   * @throws {RegistryError} if no app with the given ID exists, or
   *         if the new manifest fails validation.
   */
  updateManifest(id: string, newManifest: AppManifest): void {
    const result = validateManifest(newManifest);
    if (!result.valid) {
      const details = result.errors
        .map((e) => `${e.field}: ${e.message}`)
        .join('; ');
      throw new RegistryError(
        `Manifest validation failed for update of "${id}": ${details}`,
      );
    }

    const app = this.apps.get(id);
    if (!app) {
      throw new RegistryError(`App "${id}" is not installed`);
    }

    // If the ID changed, remove old entry and add new one
    if (newManifest.id !== id) {
      this.apps.delete(id);
      this.apps.set(newManifest.id, {
        ...app,
        manifest: newManifest,
        updated_at: new Date().toISOString(),
      });
    } else {
      app.manifest = newManifest;
      app.updated_at = new Date().toISOString();
    }
  }

  /**
   * Grant a permission to an installed app.
   *
   * @throws {RegistryError} if the app is not installed.
   */
  grantPermission(id: string, permission: string): void {
    const app = this.apps.get(id);
    if (!app) {
      throw new RegistryError(`App "${id}" is not installed`);
    }
    if (!app.granted_permissions.includes(permission)) {
      app.granted_permissions.push(permission);
    }
  }

  /**
   * Revoke a permission from an installed app.
   *
   * @throws {RegistryError} if the app is not installed.
   */
  revokePermission(id: string, permission: string): void {
    const app = this.apps.get(id);
    if (!app) {
      throw new RegistryError(`App "${id}" is not installed`);
    }
    app.granted_permissions = app.granted_permissions.filter(
      (p) => p !== permission,
    );
  }

  /**
   * Clear all entries (useful for testing).
   */
  clear(): void {
    this.apps.clear();
  }

  /**
   * Number of installed apps.
   */
  get size(): number {
    return this.apps.size;
  }
}
