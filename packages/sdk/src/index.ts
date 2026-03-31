/**
 * @cortexos/sdk -- CortexOS third-party app SDK for TypeScript.
 *
 * This package provides types, manifest validation, an in-memory app
 * registry, and a permission checker for third-party CortexOS apps.
 * It has no runtime dependencies and is suitable for use in both
 * browser and Node.js environments.
 */

// Types
export {
  MANIFEST_SCHEMA_VERSION,
  MAX_MANIFEST_ID_LENGTH,
  MAX_APP_NAME_LENGTH,
  MAX_DESCRIPTION_LENGTH,
  MAX_PERMISSION_REASON_LENGTH,
  MAX_AUTHOR_NAME_LENGTH,
  DEFAULT_MAX_FILE_SIZE,
  KNOWN_PERMISSIONS,
  PermissionType,
  AppCategory,
  InstallSource,
  AppState,
  ValidationSeverity,
} from './types/manifest';

export type {
  AuthorInfo,
  AppPermission,
  FileHandler,
  AppCapabilities,
  WindowConfig,
  AppManifest,
  InstalledApp,
  ValidationError,
  ValidationResult,
} from './types/manifest';

// Validators
export {
  validateAppId,
  validateVersion,
  validatePermissions,
  validateManifest,
} from './validators/manifest-validator';

// Registry
export { AppRegistry, RegistryError } from './registry/app-registry';

// Permission checking
export {
  checkPermission,
  getGrantedPermissions,
  getPendingPermissions,
} from './sandbox/permission-check';

export type { PermissionCheckResult } from './sandbox/permission-check';
