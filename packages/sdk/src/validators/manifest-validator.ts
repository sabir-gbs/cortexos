/**
 * Manifest validation logic for CortexOS app manifests.
 *
 * Every function is pure (no side effects) and can be tested in isolation.
 * The main entry point is {@link validateManifest} which accepts an
 * `unknown` value and produces a full {@link ValidationResult}.
 */

import {
  MANIFEST_SCHEMA_VERSION,
  MAX_MANIFEST_ID_LENGTH,
  MAX_APP_NAME_LENGTH,
  MAX_DESCRIPTION_LENGTH,
  MAX_PERMISSION_REASON_LENGTH,
  MAX_AUTHOR_NAME_LENGTH,
  KNOWN_PERMISSIONS,
  ValidationSeverity,
} from '../types/manifest';
import type {
  AppManifest,
  AppPermission,
  AppCapabilities,
  AuthorInfo,
  ValidationError,
  ValidationResult,
} from '../types/manifest';

// ---------------------------------------------------------------------------
// Primitive validators
// ---------------------------------------------------------------------------

const APP_ID_REGEX = /^[a-z0-9]+(\.[a-z0-9]+)*\.[a-z0-9-]+$/;
const SEMVER_REGEX = /^\d+\.\d+\.\d+$/;
const ICON_EXTENSION_REGEX = /\.(png|svg|ico)$/i;

/**
 * Validate an app ID string against the reverse-domain format.
 *
 * Rules (from SPEC 21):
 * - Must match `^[a-z0-9]+(\.[a-z0-9]+)*\.[a-z0-9-]+$`
 * - Maximum 128 characters
 */
export function validateAppId(id: string): boolean {
  if (id.length === 0 || id.length > MAX_MANIFEST_ID_LENGTH) return false;
  return APP_ID_REGEX.test(id);
}

/**
 * Validate a semver version string.
 *
 * Must match `^\d+\.\d+\.\d+$` (no pre-release or build metadata in v1).
 */
export function validateVersion(version: string): boolean {
  return SEMVER_REGEX.test(version);
}

/**
 * Validate that every permission name in a list is a known permission.
 */
export function validatePermissions(permissions: string[]): boolean {
  return permissions.every((p) => KNOWN_PERMISSIONS.includes(p));
}

// ---------------------------------------------------------------------------
// Object-shape checks (type narrowing helpers)
// ---------------------------------------------------------------------------

function isString(v: unknown): v is string {
  return typeof v === 'string';
}

function isBoolean(v: unknown): v is boolean {
  return typeof v === 'boolean';
}

function isNumber(v: unknown): v is number {
  return typeof v === 'number' && Number.isFinite(v);
}

function isObject(v: unknown): v is Record<string, unknown> {
  return typeof v === 'object' && v !== null && !Array.isArray(v);
}

// ---------------------------------------------------------------------------
// Field-level validators (return errors directly)
// ---------------------------------------------------------------------------

function checkRequiredString(
  value: unknown,
  field: string,
  maxLength: number,
  errors: ValidationError[],
): asserts value is string {
  if (!isString(value)) {
    errors.push({
      field,
      message: `"${field}" must be a string`,
      severity: ValidationSeverity.Error,
    });
    return;
  }
  if (value.length === 0) {
    errors.push({
      field,
      message: `"${field}" must not be empty`,
      severity: ValidationSeverity.Error,
    });
  } else if (value.length > maxLength) {
    errors.push({
      field,
      message: `"${field}" exceeds maximum length of ${maxLength} (got ${value.length})`,
      severity: ValidationSeverity.Error,
    });
  }
}

function checkAppId(value: unknown, errors: ValidationError[]): void {
  if (!isString(value)) {
    errors.push({
      field: 'id',
      message: '"id" must be a string',
      severity: ValidationSeverity.Error,
    });
    return;
  }
  if (value.length === 0) {
    errors.push({
      field: 'id',
      message: '"id" must not be empty',
      severity: ValidationSeverity.Error,
    });
    return;
  }
  if (value.length > MAX_MANIFEST_ID_LENGTH) {
    errors.push({
      field: 'id',
      message: `"id" exceeds maximum length of ${MAX_MANIFEST_ID_LENGTH}`,
      severity: ValidationSeverity.Error,
    });
  }
  if (!APP_ID_REGEX.test(value)) {
    errors.push({
      field: 'id',
      message:
        '"id" must be in reverse-domain format (e.g. "com.example.my-app")',
      severity: ValidationSeverity.Error,
    });
  }
}

function checkVersion(
  value: unknown,
  field: string,
  errors: ValidationError[],
): void {
  if (!isString(value)) {
    errors.push({
      field,
      message: `"${field}" must be a string`,
      severity: ValidationSeverity.Error,
    });
    return;
  }
  if (!SEMVER_REGEX.test(value)) {
    errors.push({
      field,
      message: `"${field}" must be semver (MAJOR.MINOR.PATCH), got "${value}"`,
      severity: ValidationSeverity.Error,
    });
  }
}

function checkIcon(value: unknown, errors: ValidationError[], warnings: ValidationError[]): void {
  if (!isString(value)) {
    errors.push({
      field: 'icon',
      message: '"icon" must be a string',
      severity: ValidationSeverity.Error,
    });
    return;
  }
  if (value.length === 0) {
    errors.push({
      field: 'icon',
      message: '"icon" must not be empty',
      severity: ValidationSeverity.Error,
    });
    return;
  }
  if (!ICON_EXTENSION_REGEX.test(value)) {
    errors.push({
      field: 'icon',
      message: '"icon" must be a PNG, SVG, or ICO file',
      severity: ValidationSeverity.Error,
    });
  }
  // Warn if path looks absolute (should be relative within package)
  if (value.startsWith('/')) {
    warnings.push({
      field: 'icon',
      message: '"icon" should be a relative path within the package',
      severity: ValidationSeverity.Warning,
    });
  }
}

function checkAuthor(value: unknown, errors: ValidationError[]): void {
  if (!isObject(value)) {
    errors.push({
      field: 'author',
      message: '"author" must be an object',
      severity: ValidationSeverity.Error,
    });
    return;
  }
  checkRequiredString(value.name, 'author.name', MAX_AUTHOR_NAME_LENGTH, errors);
  if (value.email !== undefined && !isString(value.email)) {
    errors.push({
      field: 'author.email',
      message: '"author.email" must be a string when present',
      severity: ValidationSeverity.Error,
    });
  }
  if (value.url !== undefined && !isString(value.url)) {
    errors.push({
      field: 'author.url',
      message: '"author.url" must be a string when present',
      severity: ValidationSeverity.Error,
    });
  }
}

function checkPermissionsArray(
  value: unknown,
  errors: ValidationError[],
  warnings: ValidationError[],
): void {
  if (!Array.isArray(value)) {
    errors.push({
      field: 'permissions',
      message: '"permissions" must be an array',
      severity: ValidationSeverity.Error,
    });
    return;
  }

  value.forEach((perm, i) => {
    const prefix = `permissions[${i}]`;
    if (!isObject(perm)) {
      errors.push({
        field: prefix,
        message: `Permission entry must be an object`,
        severity: ValidationSeverity.Error,
      });
      return;
    }

    // permission name
    if (!isString(perm.permission)) {
      errors.push({
        field: `${prefix}.permission`,
        message: '"permission" must be a string',
        severity: ValidationSeverity.Error,
      });
    } else if (!KNOWN_PERMISSIONS.includes(perm.permission)) {
      errors.push({
        field: `${prefix}.permission`,
        message: `Unknown permission "${perm.permission}"`,
        severity: ValidationSeverity.Error,
      });
    }

    // required flag
    if (perm.required !== undefined && !isBoolean(perm.required)) {
      errors.push({
        field: `${prefix}.required`,
        message: '"required" must be a boolean when present',
        severity: ValidationSeverity.Error,
      });
    }

    // reason
    if (!isString(perm.reason)) {
      errors.push({
        field: `${prefix}.reason`,
        message: '"reason" must be a string',
        severity: ValidationSeverity.Error,
      });
    } else if (perm.reason.length === 0) {
      warnings.push({
        field: `${prefix}.reason`,
        message: '"reason" should not be empty',
        severity: ValidationSeverity.Warning,
      });
    } else if (perm.reason.length > MAX_PERMISSION_REASON_LENGTH) {
      errors.push({
        field: `${prefix}.reason`,
        message: `"reason" exceeds maximum length of ${MAX_PERMISSION_REASON_LENGTH}`,
        severity: ValidationSeverity.Error,
      });
    }
  });
}

function checkCapabilities(value: unknown, errors: ValidationError[]): void {
  if (!isObject(value)) {
    errors.push({
      field: 'capabilities',
      message: '"capabilities" must be an object',
      severity: ValidationSeverity.Error,
    });
    return;
  }

  const boolFields: Array<keyof AppCapabilities> = [
    'ai',
    'background_execution',
    'autostart',
    'system_tray',
  ];
  for (const key of boolFields) {
    if (value[key] !== undefined && !isBoolean(value[key])) {
      errors.push({
        field: `capabilities.${key}`,
        message: `"capabilities.${key}" must be a boolean`,
        severity: ValidationSeverity.Error,
      });
    }
  }

  if (value.max_file_size !== undefined && !isNumber(value.max_file_size)) {
    errors.push({
      field: 'capabilities.max_file_size',
      message: '"capabilities.max_file_size" must be a number',
      severity: ValidationSeverity.Error,
    });
  }

  if (value.file_handlers !== undefined) {
    if (!Array.isArray(value.file_handlers)) {
      errors.push({
        field: 'capabilities.file_handlers',
        message: '"capabilities.file_handlers" must be an array',
        severity: ValidationSeverity.Error,
      });
    } else {
      (value.file_handlers as unknown[]).forEach((handler, i) => {
        const prefix = `capabilities.file_handlers[${i}]`;
        if (!isObject(handler)) {
          errors.push({
            field: prefix,
            message: 'File handler must be an object',
            severity: ValidationSeverity.Error,
          });
          return;
        }
        checkRequiredString(handler.extension, `${prefix}.extension`, 32, errors);
        checkRequiredString(handler.mime_type, `${prefix}.mime_type`, 128, errors);
        checkRequiredString(handler.label, `${prefix}.label`, 64, errors);
      });
    }
  }
}

// ---------------------------------------------------------------------------
// Top-level validator
// ---------------------------------------------------------------------------

/**
 * Validate an unknown value as a CortexOS app manifest.
 *
 * Returns a {@link ValidationResult} containing all errors and warnings
 * found during validation. The `valid` flag is `true` only when the
 * `errors` array is empty (warnings do not affect validity).
 *
 * Unknown top-level keys are reported as warnings per SPEC 21 section 6.2.
 */
export function validateManifest(input: unknown): ValidationResult {
  const errors: ValidationError[] = [];
  const warnings: ValidationError[] = [];

  if (!isObject(input)) {
    errors.push({
      field: '',
      message: 'Manifest must be a non-null object',
      severity: ValidationSeverity.Error,
    });
    return { valid: false, errors, warnings };
  }

  // Detect unknown top-level keys (SPEC 21 section 6.2: ignored but logged at warn)
  const knownKeys = new Set([
    'id', 'name', 'version', 'entry_point', 'description', 'icon',
    'permissions', 'capabilities', 'min_os_version', 'author',
    'category', 'homepage_url', 'support_url', 'license', 'window',
  ]);
  for (const key of Object.keys(input)) {
    if (!knownKeys.has(key)) {
      warnings.push({
        field: key,
        message: `Unknown manifest field "${key}" will be ignored`,
        severity: ValidationSeverity.Warning,
      });
    }
  }

  // --- Required fields ---
  checkAppId(input.id, errors);
  checkRequiredString(input.name, 'name', MAX_APP_NAME_LENGTH, errors);
  checkVersion(input.version, 'version', errors);
  checkRequiredString(input.entry_point, 'entry_point', 256, errors);
  checkRequiredString(input.description, 'description', MAX_DESCRIPTION_LENGTH, errors);
  checkIcon(input.icon, errors, warnings);
  checkVersion(input.min_os_version, 'min_os_version', errors);
  checkAuthor(input.author, errors);
  checkPermissionsArray(input.permissions, errors, warnings);
  checkCapabilities(input.capabilities, errors);

  // --- Optional fields ---
  if (input.homepage_url !== undefined && !isString(input.homepage_url)) {
    errors.push({
      field: 'homepage_url',
      message: '"homepage_url" must be a string when present',
      severity: ValidationSeverity.Error,
    });
  }
  if (input.support_url !== undefined && !isString(input.support_url)) {
    errors.push({
      field: 'support_url',
      message: '"support_url" must be a string when present',
      severity: ValidationSeverity.Error,
    });
  }
  if (input.license !== undefined && !isString(input.license)) {
    errors.push({
      field: 'license',
      message: '"license" must be a string when present',
      severity: ValidationSeverity.Error,
    });
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings,
  };
}
