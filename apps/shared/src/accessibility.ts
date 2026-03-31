/**
 * CortexOS Accessibility and Keyboard System (Spec 15).
 *
 * Provides keyboard shortcut registry, focus trap utility,
 * ARIA announcement helper, and accessibility settings types.
 */

// ── Keyboard Shortcut Registry ────────────────────────────────────────────────

export interface KeyboardShortcut {
  id: string;
  key_combo: string;         // e.g. "Ctrl+S", "Ctrl+Space"
  description: string;       // Human-readable: "Save file"
  owner: string;             // "system" or app_id
  scope: 'global' | 'app';
  when_focused?: string;     // Only active when this element focused
  handler: (e: KeyboardEvent) => void;
}

/** System shortcut constants from spec 07/15. */
export const SHORTCUTS = {
  commandPalette: 'Ctrl+Space',
  aiAssistant: 'Ctrl+Shift+A',
  appLauncher: 'Meta+A',
  settings: 'Meta+,',
  closeWindow: 'Meta+W',
  toggleFullscreen: 'F11',
} as const;

type ShortcutMap = Map<string, KeyboardShortcut>;

class ShortcutRegistry {
  private shortcuts: ShortcutMap = new Map();

  /** Register a keyboard shortcut. Returns false if a conflict exists. */
  register(shortcut: KeyboardShortcut): boolean {
    const existing = this.shortcuts.get(shortcut.key_combo);
    if (existing) {
      // System shortcuts always win
      if (existing.owner === 'system' && shortcut.owner !== 'system') {
        return false;
      }
    }
    this.shortcuts.set(shortcut.key_combo, shortcut);
    return true;
  }

  /** Unregister a shortcut by key combo. */
  unregister(key_combo: string): void {
    this.shortcuts.delete(key_combo);
  }

  /** Unregister all shortcuts for a given owner. */
  unregisterByOwner(owner: string): void {
    for (const [key, shortcut] of this.shortcuts) {
      if (shortcut.owner === owner) {
        this.shortcuts.delete(key);
      }
    }
  }

  /** Handle a keyboard event, executing matching shortcut. */
  handleEvent(event: KeyboardEvent): boolean {
    for (const shortcut of this.shortcuts.values()) {
      if (matchesShortcut(event, shortcut.key_combo)) {
        shortcut.handler(event);
        return true;
      }
    }
    return false;
  }

  /** Get all registered shortcuts. */
  getAll(): KeyboardShortcut[] {
    return Array.from(this.shortcuts.values());
  }

  /** Clear all shortcuts. */
  clear(): void {
    this.shortcuts.clear();
  }
}

/** Global shortcut registry singleton. */
export const shortcutRegistry = new ShortcutRegistry();

/** Check if a keyboard event matches a shortcut pattern. */
export function matchesShortcut(event: KeyboardEvent, shortcut: string): boolean {
  const parts = shortcut.split('+');
  const requiresCtrl = parts.includes('Ctrl');
  const requiresShift = parts.includes('Shift');
  const requiresMeta = parts.includes('Meta');
  const requiresAlt = parts.includes('Alt');
  let key = parts.find(p => !['Ctrl', 'Shift', 'Meta', 'Alt'].includes(p));

  // Normalize: shortcut uses "Space" but KeyboardEvent.key is " "
  if (key === 'Space') key = ' ';

  const keyMatch = !key ||
    event.key === key ||
    event.key.toLowerCase() === key.toLowerCase();

  return (
    event.ctrlKey === requiresCtrl &&
    event.shiftKey === requiresShift &&
    event.metaKey === requiresMeta &&
    event.altKey === requiresAlt &&
    keyMatch
  );
}

// ── Focus Trap ────────────────────────────────────────────────────────────────

export interface FocusTrapOptions {
  /** Restore focus to previously focused element on deactivate. */
  restoreFocus?: boolean;
  /** Initial element to focus when trap activates. */
  initialFocus?: HTMLElement | string;
}

/**
 * Create a focus trap on a container element.
 *
 * When active, Tab/Shift+Tab cycles only through focusable elements
 * within the container. Escape calls onEscape if provided.
 *
 * Returns deactivate function.
 */
export function createFocusTrap(
  container: HTMLElement,
  onEscape?: () => void,
  options: FocusTrapOptions = {},
): () => void {
  let active = true;
  let previouslyFocused: HTMLElement | null = document.activeElement as HTMLElement;

  const getFocusableElements = (): HTMLElement[] => {
    const selector = [
      'a[href]',
      'button:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      'textarea:not([disabled])',
      '[tabindex]:not([tabindex="-1"])',
      '[role="tab"]',
      '[role="option"]',
      '[role="menuitem"]',
    ].join(', ');
    return Array.from(container.querySelectorAll<HTMLElement>(selector))
      .filter(el => !el.hasAttribute('disabled') && el.tabIndex >= 0);
  };

  // Focus initial element
  const focusInitial = () => {
    if (options.initialFocus) {
      const el = typeof options.initialFocus === 'string'
        ? container.querySelector<HTMLElement>(options.initialFocus)
        : options.initialFocus;
      el?.focus();
    } else {
      const first = getFocusableElements()[0];
      first?.focus();
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (!active) return;

    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      onEscape?.();
      return;
    }

    if (e.key === 'Tab') {
      const focusable = getFocusableElements();
      if (focusable.length === 0) {
        e.preventDefault();
        return;
      }

      const first = focusable[0];
      const last = focusable[focusable.length - 1];

      if (e.shiftKey) {
        if (document.activeElement === first) {
          e.preventDefault();
          last.focus();
        }
      } else {
        if (document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    }
  };

  container.addEventListener('keydown', handleKeyDown);
  focusInitial();

  const deactivate = () => {
    if (!active) return;
    active = false;
    container.removeEventListener('keydown', handleKeyDown);
    if (options.restoreFocus !== false && previouslyFocused) {
      previouslyFocused.focus();
    }
  };

  return deactivate;
}

// ── ARIA Announcements ────────────────────────────────────────────────────────

/**
 * Announce a message via ARIA live region.
 * Creates a temporary element for screen readers.
 */
export function announce(message: string, priority: 'polite' | 'assertive' = 'polite'): void {
  const el = document.createElement('div');
  el.setAttribute('role', 'status');
  el.setAttribute('aria-live', priority);
  el.setAttribute('aria-atomic', 'true');
  el.className = 'sr-only';
  el.textContent = message;
  document.body.appendChild(el);
  setTimeout(() => {
    if (el.parentNode) {
      document.body.removeChild(el);
    }
  }, 1000);
}

// ── Accessibility Settings ────────────────────────────────────────────────────

export interface AccessibilitySettings {
  keyboard_nav_enabled: boolean;       // Default: true
  high_contrast: boolean;              // Default: false
  reduce_motion: boolean;              // Default: false
  font_scale: number;                  // Default: 100, Range: 100-200, Step: 10
  focus_indicator_color: string;       // Default: "#4F46E5"
  screen_reader_announcements: boolean;// Default: true
}

export const DEFAULT_ACCESSIBILITY_SETTINGS: AccessibilitySettings = {
  keyboard_nav_enabled: true,
  high_contrast: false,
  reduce_motion: false,
  font_scale: 100,
  focus_indicator_color: '#4F46E5',
  screen_reader_announcements: true,
};

/**
 * Detect OS-level reduce-motion preference.
 */
export function prefersReducedMotion(): boolean {
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

/**
 * Detect OS-level high-contrast preference.
 */
export function prefersHighContrast(): boolean {
  return window.matchMedia('(prefers-contrast: more)').matches;
}

/**
 * Detect OS-level forced-colors (Windows High Contrast mode).
 */
export function usesForcedColors(): boolean {
  return window.matchMedia('(forced-colors: active)').matches;
}
