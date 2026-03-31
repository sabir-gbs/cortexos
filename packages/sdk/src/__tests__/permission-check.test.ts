import { describe, it, expect, beforeEach } from 'vitest';
import {
  checkPermission,
  getGrantedPermissions,
  getPendingPermissions,
} from '../sandbox/permission-check';
import { AppRegistry } from '../registry/app-registry';
import {
  PermissionType,
  InstallSource,
} from '../types/manifest';
import type { AppManifest } from '../types/manifest';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function validManifest(overrides?: Partial<AppManifest>): AppManifest {
  return {
    id: 'com.example.test-app',
    name: 'Test App',
    version: '1.0.0',
    entry_point: 'index.html',
    description: 'A test application',
    icon: 'icon.png',
    permissions: [
      { permission: PermissionType.FilesRead, required: true, reason: 'Read files' },
      { permission: PermissionType.FilesWrite, required: true, reason: 'Write files' },
      { permission: PermissionType.Network, required: false, reason: 'Fetch data' },
    ],
    capabilities: {
      ai: false,
      background_execution: false,
      autostart: false,
      max_file_size: 104_857_600,
      system_tray: false,
      file_handlers: [],
    },
    min_os_version: '0.1.0',
    author: { name: 'Author' },
    ...overrides,
  };
}

function makeSetup(): { registry: AppRegistry } {
  const registry = new AppRegistry();
  registry.registerApp(
    validManifest(),
    { grantedPermissions: [PermissionType.FilesRead, PermissionType.Network] },
  );
  return { registry };
}

// ---------------------------------------------------------------------------
// checkPermission
// ---------------------------------------------------------------------------

describe('checkPermission', () => {
  let registry: AppRegistry;

  beforeEach(() => {
    ({ registry } = makeSetup());
  });

  it('allows a declared and granted permission', () => {
    const result = checkPermission(registry, 'com.example.test-app', PermissionType.FilesRead);
    expect(result.allowed).toBe(true);
    expect(result.reason).toBeUndefined();
  });

  it('denies when app is not installed', () => {
    const result = checkPermission(registry, 'com.example.nonexistent', PermissionType.FilesRead);
    expect(result.allowed).toBe(false);
    expect(result.reason).toContain('not installed');
  });

  it('denies an unknown permission string', () => {
    const result = checkPermission(registry, 'com.example.test-app', 'totally.fake');
    expect(result.allowed).toBe(false);
    expect(result.reason).toContain('Unknown permission');
  });

  it('denies when permission is not declared in manifest', () => {
    // The manifest declares files.read, files.write, network -- not clipboard
    const result = checkPermission(registry, 'com.example.test-app', PermissionType.Clipboard);
    expect(result.allowed).toBe(false);
    expect(result.reason).toContain('has not declared');
  });

  it('denies when permission is declared but not granted', () => {
    // FilesWrite is declared but not in granted_permissions
    const result = checkPermission(registry, 'com.example.test-app', PermissionType.FilesWrite);
    expect(result.allowed).toBe(false);
    expect(result.reason).toContain('not been granted');
  });

  it('allows network permission (declared and granted)', () => {
    const result = checkPermission(registry, 'com.example.test-app', PermissionType.Network);
    expect(result.allowed).toBe(true);
  });

  it('denies after permission is revoked', () => {
    registry.revokePermission('com.example.test-app', PermissionType.Network);
    const result = checkPermission(registry, 'com.example.test-app', PermissionType.Network);
    expect(result.allowed).toBe(false);
  });

  it('allows after permission is re-granted', () => {
    registry.revokePermission('com.example.test-app', PermissionType.FilesRead);
    registry.grantPermission('com.example.test-app', PermissionType.FilesRead);
    const result = checkPermission(registry, 'com.example.test-app', PermissionType.FilesRead);
    expect(result.allowed).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// getGrantedPermissions
// ---------------------------------------------------------------------------

describe('getGrantedPermissions', () => {
  let registry: AppRegistry;

  beforeEach(() => {
    ({ registry } = makeSetup());
  });

  it('returns granted permissions that are also declared', () => {
    const perms = getGrantedPermissions(registry, 'com.example.test-app');
    // Granted: files.read, network. Both are declared.
    expect(perms).toContain(PermissionType.FilesRead);
    expect(perms).toContain(PermissionType.Network);
    // FilesWrite is declared but not granted
    expect(perms).not.toContain(PermissionType.FilesWrite);
  });

  it('returns empty array for unknown app', () => {
    const perms = getGrantedPermissions(registry, 'com.example.nonexistent');
    expect(perms).toEqual([]);
  });

  it('excludes granted permissions that are not in manifest', () => {
    // Artificially grant a permission not in the manifest
    registry.grantPermission('com.example.test-app', PermissionType.Clipboard);
    const perms = getGrantedPermissions(registry, 'com.example.test-app');
    // Clipboard is not declared in the manifest, so it should be excluded
    expect(perms).not.toContain(PermissionType.Clipboard);
  });

  it('updates when permissions are revoked', () => {
    registry.revokePermission('com.example.test-app', PermissionType.Network);
    const perms = getGrantedPermissions(registry, 'com.example.test-app');
    expect(perms).not.toContain(PermissionType.Network);
  });
});

// ---------------------------------------------------------------------------
// getPendingPermissions
// ---------------------------------------------------------------------------

describe('getPendingPermissions', () => {
  let registry: AppRegistry;

  beforeEach(() => {
    ({ registry } = makeSetup());
  });

  it('returns declared permissions that are not yet granted', () => {
    const pending = getPendingPermissions(registry, 'com.example.test-app');
    // Declared: files.read, files.write, network
    // Granted: files.read, network
    // Pending: files.write
    expect(pending).toEqual([PermissionType.FilesWrite]);
  });

  it('returns empty when all declared permissions are granted', () => {
    registry.grantPermission('com.example.test-app', PermissionType.FilesWrite);
    const pending = getPendingPermissions(registry, 'com.example.test-app');
    expect(pending).toEqual([]);
  });

  it('returns empty array for unknown app', () => {
    const pending = getPendingPermissions(registry, 'com.example.nonexistent');
    expect(pending).toEqual([]);
  });

  it('re-adds a revoked permission to pending', () => {
    registry.revokePermission('com.example.test-app', PermissionType.FilesRead);
    const pending = getPendingPermissions(registry, 'com.example.test-app');
    expect(pending).toContain(PermissionType.FilesRead);
  });
});
