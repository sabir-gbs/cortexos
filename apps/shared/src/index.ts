// Theme system
export {
  LIGHT_TOKENS,
  DARK_TOKENS,
  resolveTokens,
  applyTokens,
  applyFontScale,
  validateCustomTheme,
  DEFAULT_ACCESSIBILITY,
  type ThemeTokens,
  type ThemeDefinition,
  type AccessibilitySettings as ThemeAccessibilitySettings,
} from './theme';

// Accessibility & keyboard system
export {
  SHORTCUTS,
  matchesShortcut,
  shortcutRegistry,
  createFocusTrap,
  announce,
  prefersReducedMotion,
  prefersHighContrast,
  usesForcedColors,
  DEFAULT_ACCESSIBILITY_SETTINGS,
  type KeyboardShortcut,
  type FocusTrapOptions,
  type AccessibilitySettings,
} from './accessibility';
