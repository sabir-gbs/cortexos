import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  SHORTCUTS,
  matchesShortcut,
  shortcutRegistry,
  createFocusTrap,
  announce,
  DEFAULT_ACCESSIBILITY_SETTINGS,
  type KeyboardShortcut,
} from '../accessibility';

// ── matchesShortcut ──────────────────────────────────────────────────────────

describe('matchesShortcut', () => {
  function makeEvent(overrides: Partial<KeyboardEvent> = {}): KeyboardEvent {
    return {
      ctrlKey: false,
      shiftKey: false,
      metaKey: false,
      altKey: false,
      key: '',
      ...overrides,
    } as KeyboardEvent;
  }

  it('matches Ctrl+Space', () => {
    const e = makeEvent({ ctrlKey: true, key: ' ' });
    expect(matchesShortcut(e, 'Ctrl+Space')).toBe(true);
  });

  it('does not match if Ctrl not pressed', () => {
    const e = makeEvent({ key: ' ' });
    expect(matchesShortcut(e, 'Ctrl+Space')).toBe(false);
  });

  it('matches Ctrl+Shift+A', () => {
    const e = makeEvent({ ctrlKey: true, shiftKey: true, key: 'a' });
    expect(matchesShortcut(e, 'Ctrl+Shift+A')).toBe(true);
  });

  it('matches F11', () => {
    const e = makeEvent({ key: 'F11' });
    expect(matchesShortcut(e, 'F11')).toBe(true);
  });

  it('matches Meta+A', () => {
    const e = makeEvent({ metaKey: true, key: 'a' });
    expect(matchesShortcut(e, 'Meta+A')).toBe(true);
  });

  it('does not match wrong key', () => {
    const e = makeEvent({ ctrlKey: true, key: 'a' });
    expect(matchesShortcut(e, 'Ctrl+Space')).toBe(false);
  });
});

// ── ShortcutRegistry ─────────────────────────────────────────────────────────

describe('ShortcutRegistry', () => {
  beforeEach(() => {
    shortcutRegistry.clear();
  });

  it('registers and handles a shortcut', () => {
    const handler = vi.fn();
    shortcutRegistry.register({
      id: 'test',
      key_combo: 'Ctrl+K',
      description: 'Test shortcut',
      owner: 'system',
      scope: 'global',
      handler,
    });

    const e = { ctrlKey: true, shiftKey: false, metaKey: false, altKey: false, key: 'k' } as KeyboardEvent;
    shortcutRegistry.handleEvent(e);
    expect(handler).toHaveBeenCalled();
  });

  it('system shortcuts win over app shortcuts', () => {
    const systemHandler = vi.fn();
    const appHandler = vi.fn();

    shortcutRegistry.register({
      id: 'sys',
      key_combo: 'Ctrl+K',
      description: 'System',
      owner: 'system',
      scope: 'global',
      handler: systemHandler,
    });

    const registered = shortcutRegistry.register({
      id: 'app',
      key_combo: 'Ctrl+K',
      description: 'App',
      owner: 'my-app',
      scope: 'app',
      handler: appHandler,
    });

    expect(registered).toBe(false);
    // System shortcut still active
    const all = shortcutRegistry.getAll();
    expect(all.find(s => s.id === 'sys')).toBeTruthy();
  });

  it('unregisters by owner', () => {
    shortcutRegistry.register({
      id: 'test',
      key_combo: 'Ctrl+T',
      description: 'Test',
      owner: 'my-app',
      scope: 'app',
      handler: () => {},
    });

    shortcutRegistry.unregisterByOwner('my-app');
    expect(shortcutRegistry.getAll()).toHaveLength(0);
  });
});

// ── createFocusTrap ──────────────────────────────────────────────────────────

describe('createFocusTrap', () => {
  let container: HTMLDivElement;
  let deactivate: () => void;

  beforeEach(() => {
    container = document.createElement('div');
    container.innerHTML = `
      <button id="first">First</button>
      <input id="middle" />
      <button id="last">Last</button>
    `;
    document.body.appendChild(container);
  });

  afterEach(() => {
    deactivate?.();
    document.body.removeChild(container);
  });

  it('focuses first element on activation', () => {
    deactivate = createFocusTrap(container);
    expect(document.activeElement?.id).toBe('first');
  });

  it('cycles focus on Tab at last element', () => {
    deactivate = createFocusTrap(container);
    const last = container.querySelector('#last') as HTMLButtonElement;
    last.focus();

    const event = new KeyboardEvent('keydown', { key: 'Tab', bubbles: true });
    container.dispatchEvent(event);

    expect(document.activeElement?.id).toBe('first');
  });

  it('calls onEscape when Escape pressed', () => {
    const onEscape = vi.fn();
    deactivate = createFocusTrap(container, onEscape);

    const event = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true });
    container.dispatchEvent(event);

    expect(onEscape).toHaveBeenCalled();
  });

  it('restores focus on deactivate', () => {
    const outside = document.createElement('button');
    outside.id = 'outside';
    document.body.appendChild(outside);
    outside.focus();

    deactivate = createFocusTrap(container, undefined, { restoreFocus: true });
    expect(document.activeElement?.id).toBe('first');

    deactivate();
    expect(document.activeElement?.id).toBe('outside');

    document.body.removeChild(outside);
  });
});

// ── announce ─────────────────────────────────────────────────────────────────

describe('announce', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('creates an ARIA live region element', () => {
    announce('Test message');
    const el = document.querySelector('[aria-live="polite"]');
    expect(el).toBeTruthy();
    expect(el?.textContent).toBe('Test message');

    vi.advanceTimersByTime(1100);
    expect(document.querySelector('[aria-live="polite"]')).toBeNull();
  });

  it('uses assertive priority when specified', () => {
    announce('Alert!', 'assertive');
    const el = document.querySelector('[aria-live="assertive"]');
    expect(el).toBeTruthy();

    vi.advanceTimersByTime(1100);
  });
});

// ── DEFAULT_ACCESSIBILITY_SETTINGS ───────────────────────────────────────────

describe('DEFAULT_ACCESSIBILITY_SETTINGS', () => {
  it('has expected defaults', () => {
    expect(DEFAULT_ACCESSIBILITY_SETTINGS.keyboard_nav_enabled).toBe(true);
    expect(DEFAULT_ACCESSIBILITY_SETTINGS.high_contrast).toBe(false);
    expect(DEFAULT_ACCESSIBILITY_SETTINGS.reduce_motion).toBe(false);
    expect(DEFAULT_ACCESSIBILITY_SETTINGS.font_scale).toBe(100);
    expect(DEFAULT_ACCESSIBILITY_SETTINGS.screen_reader_announcements).toBe(true);
  });

  it('font_scale is within valid range', () => {
    expect(DEFAULT_ACCESSIBILITY_SETTINGS.font_scale).toBeGreaterThanOrEqual(100);
    expect(DEFAULT_ACCESSIBILITY_SETTINGS.font_scale).toBeLessThanOrEqual(200);
  });
});
