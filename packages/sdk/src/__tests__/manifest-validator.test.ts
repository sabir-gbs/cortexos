import { describe, it, expect } from 'vitest';
import {
  validateAppId,
  validateVersion,
  validatePermissions,
  validateManifest,
} from '../validators/manifest-validator';
import {
  PermissionType,
  ValidationSeverity,
} from '../types/manifest';
import type { AppManifest, ValidationResult } from '../types/manifest';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Build a minimal valid manifest for testing. */
function validManifest(overrides?: Partial<AppManifest>): AppManifest {
  return {
    id: 'com.example.test-app',
    name: 'Test App',
    version: '1.0.0',
    entry_point: 'index.html',
    description: 'A test application',
    icon: 'icon.png',
    permissions: [],
    capabilities: {
      ai: false,
      background_execution: false,
      autostart: false,
      max_file_size: 104_857_600,
      system_tray: false,
      file_handlers: [],
    },
    min_os_version: '0.1.0',
    author: {
      name: 'Test Author',
    },
    ...overrides,
  };
}

// ---------------------------------------------------------------------------
// validateAppId
// ---------------------------------------------------------------------------

describe('validateAppId', () => {
  it('accepts valid reverse-domain IDs', () => {
    expect(validateAppId('com.example.myapp')).toBe(true);
    expect(validateAppId('io.cortexos.clock')).toBe(true);
    expect(validateAppId('org.mycompany.app-name')).toBe(true);
    expect(validateAppId('dev.app123.tool')).toBe(true);
  });

  it('rejects IDs without dots', () => {
    expect(validateAppId('myapp')).toBe(false);
  });

  it('rejects IDs with spaces', () => {
    expect(validateAppId('com.example.my app')).toBe(false);
  });

  it('rejects empty IDs', () => {
    expect(validateAppId('')).toBe(false);
  });

  it('rejects IDs that start or end with a dot', () => {
    expect(validateAppId('.com.example')).toBe(false);
    expect(validateAppId('com.example.')).toBe(false);
  });

  it('rejects IDs with uppercase letters', () => {
    expect(validateAppId('Com.Example.MyApp')).toBe(false);
  });

  it('rejects IDs with special characters', () => {
    expect(validateAppId('com.example/app')).toBe(false);
    expect(validateAppId('com.example@App')).toBe(false);
  });

  it('rejects IDs exceeding 128 characters', () => {
    const longId = 'com.' + 'a'.repeat(130);
    expect(validateAppId(longId)).toBe(false);
  });

  it('rejects IDs with consecutive dots', () => {
    expect(validateAppId('com..example')).toBe(false);
  });

  it('rejects hyphens in non-final segments', () => {
    expect(validateAppId('org.my-company.lastsegment')).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// validateVersion
// ---------------------------------------------------------------------------

describe('validateVersion', () => {
  it('accepts valid semver versions', () => {
    expect(validateVersion('1.0.0')).toBe(true);
    expect(validateVersion('0.0.1')).toBe(true);
    expect(validateVersion('99.99.99')).toBe(true);
    expect(validateVersion('10.20.30')).toBe(true);
  });

  it('rejects versions with less than 3 parts', () => {
    expect(validateVersion('1.0')).toBe(false);
    expect(validateVersion('1')).toBe(false);
  });

  it('rejects versions with more than 3 parts', () => {
    expect(validateVersion('1.0.0.0')).toBe(false);
  });

  it('rejects non-numeric version parts', () => {
    expect(validateVersion('1.0.a')).toBe(false);
    expect(validateVersion('v1.0.0')).toBe(false);
  });

  it('rejects empty string', () => {
    expect(validateVersion('')).toBe(false);
  });

  it('rejects versions with pre-release tags', () => {
    expect(validateVersion('1.0.0-alpha')).toBe(false);
    expect(validateVersion('1.0.0-beta.1')).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// validatePermissions
// ---------------------------------------------------------------------------

describe('validatePermissions', () => {
  it('accepts known permissions', () => {
    expect(validatePermissions([PermissionType.FilesRead, PermissionType.Network])).toBe(true);
  });

  it('accepts an empty array', () => {
    expect(validatePermissions([])).toBe(true);
  });

  it('rejects unknown permissions', () => {
    expect(validatePermissions(['files.read', 'unknown.permission'])).toBe(false);
  });

  it('rejects totally invalid permission strings', () => {
    expect(validatePermissions(['not-a-permission'])).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// validateManifest -- top-level integration
// ---------------------------------------------------------------------------

describe('validateManifest', () => {
  it('accepts a fully valid manifest', () => {
    const result = validateManifest(validManifest());
    expect(result.valid).toBe(true);
    expect(result.errors).toHaveLength(0);
  });

  // --- Type-level checks ---

  it('rejects non-object input', () => {
    expect(validateManifest(null).valid).toBe(false);
    expect(validateManifest(undefined).valid).toBe(false);
    expect(validateManifest('string').valid).toBe(false);
    expect(validateManifest(42).valid).toBe(false);
    expect(validateManifest(true).valid).toBe(false);
    expect(validateManifest([]).valid).toBe(false);
  });

  // --- id ---

  it('rejects missing id', () => {
    const { id, ...noId } = validManifest();
    const result = validateManifest(noId);
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.field === 'id')).toBe(true);
  });

  it('rejects invalid id format', () => {
    const result = validateManifest(validManifest({ id: 'invalid' }));
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.field === 'id')).toBe(true);
  });

  it('rejects id exceeding max length', () => {
    const result = validateManifest(validManifest({ id: 'com.' + 'a'.repeat(130) }));
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.field === 'id')).toBe(true);
  });

  // --- name ---

  it('rejects empty name', () => {
    const result = validateManifest(validManifest({ name: '' }));
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.field === 'name')).toBe(true);
  });

  it('rejects name exceeding max length', () => {
    const result = validateManifest(validManifest({ name: 'x'.repeat(65) }));
    expect(result.valid).toBe(false);
  });

  // --- version ---

  it('rejects invalid version', () => {
    const result = validateManifest(validManifest({ version: '1.0' }));
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.field === 'version')).toBe(true);
  });

  // --- entry_point ---

  it('rejects empty entry_point', () => {
    const result = validateManifest(validManifest({ entry_point: '' }));
    expect(result.valid).toBe(false);
  });

  // --- description ---

  it('rejects empty description', () => {
    const result = validateManifest(validManifest({ description: '' }));
    expect(result.valid).toBe(false);
  });

  it('rejects description exceeding max length', () => {
    const result = validateManifest(validManifest({ description: 'x'.repeat(513) }));
    expect(result.valid).toBe(false);
  });

  // --- icon ---

  it('rejects empty icon', () => {
    const result = validateManifest(validManifest({ icon: '' }));
    expect(result.valid).toBe(false);
  });

  it('rejects non-png/svg/ico icon', () => {
    const result = validateManifest(validManifest({ icon: 'icon.jpg' }));
    expect(result.valid).toBe(false);
  });

  it('warns on absolute icon path', () => {
    const result = validateManifest(validManifest({ icon: '/absolute/icon.png' }));
    expect(result.valid).toBe(true);
    expect(result.warnings.some((w) => w.field === 'icon')).toBe(true);
  });

  it('accepts svg icon', () => {
    const result = validateManifest(validManifest({ icon: 'icon.svg' }));
    expect(result.valid).toBe(true);
  });

  it('accepts ico icon', () => {
    const result = validateManifest(validManifest({ icon: 'favicon.ico' }));
    expect(result.valid).toBe(true);
  });

  // --- min_os_version ---

  it('rejects invalid min_os_version', () => {
    const result = validateManifest(validManifest({ min_os_version: 'bad' }));
    expect(result.valid).toBe(false);
  });

  // --- author ---

  it('rejects missing author', () => {
    const { author, ...noAuthor } = validManifest();
    const result = validateManifest(noAuthor);
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.field === 'author')).toBe(true);
  });

  it('rejects author with empty name', () => {
    const result = validateManifest(validManifest({ author: { name: '' } }));
    expect(result.valid).toBe(false);
  });

  it('accepts author with optional fields', () => {
    const result = validateManifest(
      validManifest({
        author: { name: 'Test Author', email: 'test@example.com', url: 'https://example.com' },
      }),
    );
    expect(result.valid).toBe(true);
  });

  // --- permissions ---

  it('accepts valid permissions', () => {
    const result = validateManifest(
      validManifest({
        permissions: [
          { permission: PermissionType.FilesRead, required: true, reason: 'Read files' },
          { permission: PermissionType.Network, required: false, reason: 'Fetch data' },
        ],
      }),
    );
    expect(result.valid).toBe(true);
  });

  it('rejects unknown permission name', () => {
    const result = validateManifest(
      validManifest({
        permissions: [
          { permission: 'not.real', required: true, reason: 'Bad' },
        ],
      }),
    );
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.field.includes('permissions['))).toBe(true);
  });

  it('rejects permission with missing reason', () => {
    const result = validateManifest(
      validManifest({
        permissions: [
          { permission: PermissionType.FilesRead, required: true, reason: '' },
        ],
      }),
    );
    // Empty reason is a warning, not an error
    expect(result.valid).toBe(true);
    expect(result.warnings.some((w) => w.field.includes('reason'))).toBe(true);
  });

  it('rejects permission reason exceeding max length', () => {
    const result = validateManifest(
      validManifest({
        permissions: [
          { permission: PermissionType.FilesRead, required: true, reason: 'x'.repeat(257) },
        ],
      }),
    );
    expect(result.valid).toBe(false);
  });

  // --- capabilities ---

  it('rejects capabilities as non-object', () => {
    const result = validateManifest(validManifest({ capabilities: 'yes' as unknown as any }));
    expect(result.valid).toBe(false);
  });

  it('rejects boolean capability fields that are not boolean', () => {
    const result = validateManifest(
      validManifest({
        capabilities: {
          ai: 'true' as unknown as boolean,
          background_execution: false,
          autostart: false,
          max_file_size: 100,
          system_tray: false,
          file_handlers: [],
        },
      }),
    );
    expect(result.valid).toBe(false);
  });

  it('rejects invalid file_handlers', () => {
    const result = validateManifest(
      validManifest({
        capabilities: {
          ai: false,
          background_execution: false,
          autostart: false,
          max_file_size: 100,
          system_tray: false,
          file_handlers: 'not-array' as unknown as any[],
        },
      }),
    );
    expect(result.valid).toBe(false);
  });

  // --- unknown fields produce warnings ---

  it('warns on unknown top-level fields', () => {
    const input = { ...validManifest(), unknown_field: 'surprise' };
    const result = validateManifest(input);
    expect(result.valid).toBe(true);
    expect(result.warnings.some((w) => w.field === 'unknown_field')).toBe(true);
  });

  // --- multiple errors at once ---

  it('collects multiple errors simultaneously', () => {
    const result = validateManifest({
      id: 'bad',
      name: '',
      version: 'nope',
      entry_point: '',
      description: '',
      icon: 'file.jpg',
      permissions: 'not-array',
      capabilities: null,
      min_os_version: 'x',
      author: null,
    });
    expect(result.valid).toBe(false);
    // Should have many errors, not just the first one
    expect(result.errors.length).toBeGreaterThanOrEqual(5);
  });

  // --- optional fields ---

  it('accepts manifests without optional fields', () => {
    const result = validateManifest(validManifest());
    expect(result.valid).toBe(true);
  });

  it('accepts optional fields when provided correctly', () => {
    const result = validateManifest(
      validManifest({
        homepage_url: 'https://example.com',
        support_url: 'support@example.com',
        license: 'MIT',
        category: 'utilities' as any,
      }),
    );
    expect(result.valid).toBe(true);
  });

  it('rejects optional fields with wrong types', () => {
    const result = validateManifest(
      validManifest({
        homepage_url: 123 as unknown as string,
      }),
    );
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.field === 'homepage_url')).toBe(true);
  });
});
