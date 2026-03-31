/**
 * Permission checker for CortexOS third-party apps.
 *
 * The permission checker is used by the SDK routing layer to verify
 * that an app has been granted a specific permission before allowing
 * an API call to proceed. It queries the app registry for granted
 * permissions and cross-references with the manifest's declared
 * permissions.
 */

import { KNOWN_PERMISSIONS } from '../types/manifest';
import type { AppRegistry } from '../registry/app-registry';

/**
 * Result of a permission check.
 */
export interface PermissionCheckResult {
  /** Whether the permission is allowed. */
  allowed: boolean;
  /** Reason for denial, if applicable. */
  reason?: string;
}

/**
 * Check whether a specific app has been granted a permission.
 *
 * Three conditions must all be true for the permission to be allowed:
 *
 * 1. The app must be installed (present in the registry).
 * 2. The permission must be declared in the app's manifest.
 * 3. The permission must have been explicitly granted by the user
 *    (present in `granted_permissions`).
 *
 * @param registry - The app registry to query.
 * @param appId    - The app's unique identifier.
 * @param permission - The permission string to check.
 * @returns A {@link PermissionCheckResult}.
 */
export function checkPermission(
  registry: AppRegistry,
  appId: string,
  permission: string,
): PermissionCheckResult {
  // 1. App must be installed
  const app = registry.getApp(appId);
  if (!app) {
    return {
      allowed: false,
      reason: `App "${appId}" is not installed`,
    };
  }

  // 2. Permission must be a known permission type
  if (!KNOWN_PERMISSIONS.includes(permission)) {
    return {
      allowed: false,
      reason: `Unknown permission "${permission}"`,
    };
  }

  // 3. Permission must be declared in the manifest
  const declared = app.manifest.permissions.some(
    (p) => p.permission === permission,
  );
  if (!declared) {
    return {
      allowed: false,
      reason: `App "${appId}" has not declared permission "${permission}" in its manifest`,
    };
  }

  // 4. Permission must have been granted by the user
  if (!app.granted_permissions.includes(permission)) {
    return {
      allowed: false,
      reason: `Permission "${permission}" has not been granted for app "${appId}"`,
    };
  }

  return { allowed: true };
}

/**
 * Get the list of permissions that have been granted to an app.
 *
 * Only returns permissions that are both declared in the manifest AND
 * explicitly granted by the user.
 *
 * @param registry - The app registry to query.
 * @param appId    - The app's unique identifier.
 * @returns Array of permission strings, or an empty array if the app
 *          is not installed.
 */
export function getGrantedPermissions(
  registry: AppRegistry,
  appId: string,
): string[] {
  const app = registry.getApp(appId);
  if (!app) {
    return [];
  }

  // Intersect declared permissions with granted permissions
  const declaredSet = new Set(
    app.manifest.permissions.map((p) => p.permission),
  );
  return app.granted_permissions.filter((p) => declaredSet.has(p));
}

/**
 * Get the list of permissions that an app has declared in its manifest
 * but which have not yet been granted by the user.
 *
 * Useful for displaying a permission prompt during app first-run.
 *
 * @param registry - The app registry to query.
 * @param appId    - The app's unique identifier.
 * @returns Array of permission strings awaiting grant.
 */
export function getPendingPermissions(
  registry: AppRegistry,
  appId: string,
): string[] {
  const app = registry.getApp(appId);
  if (!app) {
    return [];
  }

  const grantedSet = new Set(app.granted_permissions);
  return app.manifest.permissions
    .filter((p) => !grantedSet.has(p.permission))
    .map((p) => p.permission);
}
