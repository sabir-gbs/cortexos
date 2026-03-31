import { describe, it, expect, beforeEach } from 'vitest';
import { AppRegistry, RegistryError } from '../registry/app-registry';
import {
  PermissionType,
  AppState,
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

function makeRegistry(): AppRegistry {
  return new AppRegistry();
}

// ---------------------------------------------------------------------------
// registerApp
// ---------------------------------------------------------------------------

describe('AppRegistry.registerApp', () => {
  let registry: AppRegistry;

  beforeEach(() => {
    registry = makeRegistry();
  });

  it('registers a valid app', () => {
    registry.registerApp(validManifest());
    expect(registry.isInstalled('com.example.test-app')).toBe(true);
    expect(registry.size).toBe(1);
  });

  it('sets the state to Installed by default', () => {
    registry.registerApp(validManifest());
    const app = registry.getApp('com.example.test-app');
    expect(app?.state).toBe(AppState.Installed);
  });

  it('stores the provided install path', () => {
    registry.registerApp(validManifest(), { installPath: '/apps/test' });
    expect(registry.getApp('com.example.test-app')?.install_path).toBe('/apps/test');
  });

  it('stores the provided data path', () => {
    registry.registerApp(validManifest(), { dataPath: '/data/test' });
    expect(registry.getApp('com.example.test-app')?.data_path).toBe('/data/test');
  });

  it('stores the provided install source', () => {
    registry.registerApp(validManifest(), { installSource: InstallSource.AppStore });
    expect(registry.getApp('com.example.test-app')?.install_source).toBe(InstallSource.AppStore);
  });

  it('stores granted permissions', () => {
    registry.registerApp(validManifest(), {
      grantedPermissions: [PermissionType.FilesRead],
    });
    expect(registry.getApp('com.example.test-app')?.granted_permissions).toEqual([
      PermissionType.FilesRead,
    ]);
  });

  it('rejects an invalid manifest', () => {
    expect(() =>
      registry.registerApp(validManifest({ id: 'bad' })),
    ).toThrow(RegistryError);
  });

  it('rejects a manifest missing required fields', () => {
    const bad = { ...validManifest(), version: '' };
    expect(() => registry.registerApp(bad)).toThrow(RegistryError);
  });

  it('rejects duplicate app IDs', () => {
    registry.registerApp(validManifest());
    expect(() =>
      registry.registerApp(
        validManifest({ version: '2.0.0', description: 'Updated' }),
      ),
    ).toThrow(RegistryError);
    expect(registry.size).toBe(1);
  });
});

// ---------------------------------------------------------------------------
// unregisterApp
// ---------------------------------------------------------------------------

describe('AppRegistry.unregisterApp', () => {
  let registry: AppRegistry;

  beforeEach(() => {
    registry = makeRegistry();
  });

  it('removes an installed app', () => {
    registry.registerApp(validManifest());
    registry.unregisterApp('com.example.test-app');
    expect(registry.isInstalled('com.example.test-app')).toBe(false);
    expect(registry.size).toBe(0);
  });

  it('throws when app is not installed', () => {
    expect(() => registry.unregisterApp('com.example.nonexistent')).toThrow(
      RegistryError,
    );
  });
});

// ---------------------------------------------------------------------------
// getApp
// ---------------------------------------------------------------------------

describe('AppRegistry.getApp', () => {
  let registry: AppRegistry;

  beforeEach(() => {
    registry = makeRegistry();
  });

  it('returns the installed app', () => {
    registry.registerApp(validManifest());
    const app = registry.getApp('com.example.test-app');
    expect(app).toBeDefined();
    expect(app?.manifest.name).toBe('Test App');
  });

  it('returns undefined for unknown app', () => {
    expect(registry.getApp('com.example.nonexistent')).toBeUndefined();
  });
});

// ---------------------------------------------------------------------------
// listApps
// ---------------------------------------------------------------------------

describe('AppRegistry.listApps', () => {
  let registry: AppRegistry;

  beforeEach(() => {
    registry = makeRegistry();
  });

  it('returns an empty array when no apps are installed', () => {
    expect(registry.listApps()).toEqual([]);
  });

  it('lists all installed apps', () => {
    registry.registerApp(validManifest());
    registry.registerApp(
      validManifest({
        id: 'com.example.another',
        name: 'Another App',
        description: 'Second app',
      }),
    );
    const apps = registry.listApps();
    expect(apps).toHaveLength(2);
    const ids = apps.map((a) => a.manifest.id).sort();
    expect(ids).toEqual(['com.example.another', 'com.example.test-app']);
  });

  it('returns a copy (mutations do not affect registry)', () => {
    registry.registerApp(validManifest());
    const list = registry.listApps();
    list.pop();
    expect(registry.size).toBe(1);
  });
});

// ---------------------------------------------------------------------------
// isInstalled
// ---------------------------------------------------------------------------

describe('AppRegistry.isInstalled', () => {
  let registry: AppRegistry;

  beforeEach(() => {
    registry = makeRegistry();
  });

  it('returns true for installed apps', () => {
    registry.registerApp(validManifest());
    expect(registry.isInstalled('com.example.test-app')).toBe(true);
  });

  it('returns false for unknown apps', () => {
    expect(registry.isInstalled('com.example.nothing')).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// setAppState
// ---------------------------------------------------------------------------

describe('AppRegistry.setAppState', () => {
  let registry: AppRegistry;

  beforeEach(() => {
    registry = makeRegistry();
    registry.registerApp(validManifest());
  });

  it('updates the state to Running', () => {
    registry.setAppState('com.example.test-app', AppState.Running);
    expect(registry.getApp('com.example.test-app')?.state).toBe(AppState.Running);
  });

  it('updates the state to Suspended', () => {
    registry.setAppState('com.example.test-app', AppState.Suspended);
    expect(registry.getApp('com.example.test-app')?.state).toBe(AppState.Suspended);
  });

  it('updates the state to Stopped', () => {
    registry.setAppState('com.example.test-app', AppState.Stopped);
    expect(registry.getApp('com.example.test-app')?.state).toBe(AppState.Stopped);
  });

  it('throws for unknown app', () => {
    expect(() =>
      registry.setAppState('com.example.nonexistent', AppState.Running),
    ).toThrow(RegistryError);
  });
});

// ---------------------------------------------------------------------------
// updateManifest
// ---------------------------------------------------------------------------

describe('AppRegistry.updateManifest', () => {
  let registry: AppRegistry;

  beforeEach(() => {
    registry = makeRegistry();
    registry.registerApp(validManifest());
  });

  it('replaces the manifest with a valid new one', () => {
    const newManifest = validManifest({ version: '2.0.0', description: 'Updated!' });
    registry.updateManifest('com.example.test-app', newManifest);
    expect(registry.getApp('com.example.test-app')?.manifest.version).toBe('2.0.0');
  });

  it('rejects an invalid new manifest', () => {
    const bad = validManifest({ version: 'not-semver' });
    expect(() =>
      registry.updateManifest('com.example.test-app', bad),
    ).toThrow(RegistryError);
  });

  it('throws for unknown app', () => {
    expect(() =>
      registry.updateManifest('com.example.nonexistent', validManifest()),
    ).toThrow(RegistryError);
  });

  it('handles ID change during update', () => {
    const newManifest = validManifest({ id: 'com.example.renamed' });
    registry.updateManifest('com.example.test-app', newManifest);
    expect(registry.isInstalled('com.example.test-app')).toBe(false);
    expect(registry.isInstalled('com.example.renamed')).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// grantPermission / revokePermission
// ---------------------------------------------------------------------------

describe('AppRegistry permission management', () => {
  let registry: AppRegistry;

  beforeEach(() => {
    registry = makeRegistry();
    registry.registerApp(validManifest(), {
      grantedPermissions: [PermissionType.FilesRead],
    });
  });

  it('grants a new permission', () => {
    registry.grantPermission('com.example.test-app', PermissionType.Network);
    const app = registry.getApp('com.example.test-app');
    expect(app?.granted_permissions).toContain(PermissionType.Network);
  });

  it('does not duplicate already-granted permission', () => {
    registry.grantPermission('com.example.test-app', PermissionType.FilesRead);
    const app = registry.getApp('com.example.test-app');
    const count = app?.granted_permissions.filter(
      (p) => p === PermissionType.FilesRead,
    ).length;
    expect(count).toBe(1);
  });

  it('revokes a permission', () => {
    registry.revokePermission('com.example.test-app', PermissionType.FilesRead);
    const app = registry.getApp('com.example.test-app');
    expect(app?.granted_permissions).not.toContain(PermissionType.FilesRead);
  });

  it('grant throws for unknown app', () => {
    expect(() =>
      registry.grantPermission('com.example.nonexistent', PermissionType.Network),
    ).toThrow(RegistryError);
  });

  it('revoke throws for unknown app', () => {
    expect(() =>
      registry.revokePermission('com.example.nonexistent', PermissionType.Network),
    ).toThrow(RegistryError);
  });
});

// ---------------------------------------------------------------------------
// clear
// ---------------------------------------------------------------------------

describe('AppRegistry.clear', () => {
  it('removes all entries', () => {
    const registry = makeRegistry();
    registry.registerApp(validManifest());
    registry.registerApp(
      validManifest({ id: 'com.example.second', description: 'Second' }),
    );
    expect(registry.size).toBe(2);
    registry.clear();
    expect(registry.size).toBe(0);
    expect(registry.listApps()).toEqual([]);
  });
});
