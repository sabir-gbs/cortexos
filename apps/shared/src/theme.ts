/**
 * CortexOS Design Token System (Spec 16).
 *
 * Provides complete theme definitions with semantic token names,
 * theme resolver with accessibility overrides, and CSS custom property
 * application.
 */

// ── Token Definitions ────────────────────────────────────────────────────────

/** All design token names following --cortex-{category}-{variant}-{state} convention. */
export interface ThemeTokens {
  // Colors
  '--cortex-color-primary': string;
  '--cortex-color-primary-hover': string;
  '--cortex-color-primary-active': string;
  '--cortex-color-primary-disabled': string;
  '--cortex-color-secondary': string;
  '--cortex-color-surface': string;
  '--cortex-color-surface-raised': string;
  '--cortex-color-error': string;
  '--cortex-color-warning': string;
  '--cortex-color-success': string;
  '--cortex-color-text': string;
  '--cortex-color-text-secondary': string;
  '--cortex-color-text-disabled': string;
  '--cortex-color-border': string;
  '--cortex-color-background': string;

  // Spacing
  '--cortex-spacing-xs': string;
  '--cortex-spacing-sm': string;
  '--cortex-spacing-md': string;
  '--cortex-spacing-lg': string;
  '--cortex-spacing-xl': string;
  '--cortex-spacing-2xl': string;

  // Typography
  '--cortex-typography-font-family': string;
  '--cortex-typography-size-xs': string;
  '--cortex-typography-size-sm': string;
  '--cortex-typography-size-md': string;
  '--cortex-typography-size-lg': string;
  '--cortex-typography-size-xl': string;
  '--cortex-typography-size-2xl': string;
  '--cortex-typography-size-3xl': string;
  '--cortex-typography-size-4xl': string;
  '--cortex-typography-weight-normal': string;
  '--cortex-typography-weight-medium': string;
  '--cortex-typography-weight-semibold': string;
  '--cortex-typography-weight-bold': string;
  '--cortex-typography-line-height-tight': string;
  '--cortex-typography-line-height-normal': string;
  '--cortex-typography-line-height-relaxed': string;

  // Border Radius
  '--cortex-border-radius-none': string;
  '--cortex-border-radius-sm': string;
  '--cortex-border-radius-md': string;
  '--cortex-border-radius-lg': string;
  '--cortex-border-radius-full': string;

  // Shadows
  '--cortex-shadow-none': string;
  '--cortex-shadow-sm': string;
  '--cortex-shadow-md': string;
  '--cortex-shadow-lg': string;

  // Transitions
  '--cortex-transition-duration-fast': string;
  '--cortex-transition-duration-normal': string;
  '--cortex-transition-duration-slow': string;
  '--cortex-transition-easing': string;

  // Z-Index
  '--cortex-z-index-base': string;
  '--cortex-z-index-dropdown': string;
  '--cortex-z-index-sticky': string;
  '--cortex-z-index-fixed': string;
  '--cortex-z-index-modal': string;
  '--cortex-z-index-popover': string;
  '--cortex-z-index-tooltip': string;
}

// ── Theme Definitions ────────────────────────────────────────────────────────

export const LIGHT_TOKENS: ThemeTokens = {
  '--cortex-color-primary': '#4F46E5',
  '--cortex-color-primary-hover': '#4338CA',
  '--cortex-color-primary-active': '#3730A3',
  '--cortex-color-primary-disabled': '#A5B4FC',
  '--cortex-color-secondary': '#7C3AED',
  '--cortex-color-surface': '#FFFFFF',
  '--cortex-color-surface-raised': '#F9FAFB',
  '--cortex-color-error': '#DC2626',
  '--cortex-color-warning': '#F59E0B',
  '--cortex-color-success': '#10B981',
  '--cortex-color-text': '#111827',
  '--cortex-color-text-secondary': '#6B7280',
  '--cortex-color-text-disabled': '#9CA3AF',
  '--cortex-color-border': '#E5E7EB',
  '--cortex-color-background': '#F9FAFB',

  '--cortex-spacing-xs': '4px',
  '--cortex-spacing-sm': '8px',
  '--cortex-spacing-md': '16px',
  '--cortex-spacing-lg': '24px',
  '--cortex-spacing-xl': '32px',
  '--cortex-spacing-2xl': '48px',

  '--cortex-typography-font-family': "system-ui, -apple-system, sans-serif",
  '--cortex-typography-size-xs': '0.75rem',
  '--cortex-typography-size-sm': '0.875rem',
  '--cortex-typography-size-md': '1rem',
  '--cortex-typography-size-lg': '1.125rem',
  '--cortex-typography-size-xl': '1.25rem',
  '--cortex-typography-size-2xl': '1.5rem',
  '--cortex-typography-size-3xl': '1.875rem',
  '--cortex-typography-size-4xl': '2.25rem',
  '--cortex-typography-weight-normal': '400',
  '--cortex-typography-weight-medium': '500',
  '--cortex-typography-weight-semibold': '600',
  '--cortex-typography-weight-bold': '700',
  '--cortex-typography-line-height-tight': '1.25',
  '--cortex-typography-line-height-normal': '1.5',
  '--cortex-typography-line-height-relaxed': '1.75',

  '--cortex-border-radius-none': '0',
  '--cortex-border-radius-sm': '4px',
  '--cortex-border-radius-md': '8px',
  '--cortex-border-radius-lg': '12px',
  '--cortex-border-radius-full': '9999px',

  '--cortex-shadow-none': 'none',
  '--cortex-shadow-sm': '0 1px 2px rgba(0,0,0,0.05)',
  '--cortex-shadow-md': '0 4px 6px rgba(0,0,0,0.07)',
  '--cortex-shadow-lg': '0 10px 15px rgba(0,0,0,0.1)',

  '--cortex-transition-duration-fast': '150ms',
  '--cortex-transition-duration-normal': '250ms',
  '--cortex-transition-duration-slow': '350ms',
  '--cortex-transition-easing': 'ease-in-out',

  '--cortex-z-index-base': '0',
  '--cortex-z-index-dropdown': '100',
  '--cortex-z-index-sticky': '200',
  '--cortex-z-index-fixed': '300',
  '--cortex-z-index-modal': '400',
  '--cortex-z-index-popover': '500',
  '--cortex-z-index-tooltip': '600',
};

export const DARK_TOKENS: ThemeTokens = {
  '--cortex-color-primary': '#818CF8',
  '--cortex-color-primary-hover': '#6366F1',
  '--cortex-color-primary-active': '#4F46E5',
  '--cortex-color-primary-disabled': '#4338CA',
  '--cortex-color-secondary': '#A78BFA',
  '--cortex-color-surface': '#1A1A2E',
  '--cortex-color-surface-raised': '#16213E',
  '--cortex-color-error': '#F87171',
  '--cortex-color-warning': '#FBBF24',
  '--cortex-color-success': '#34D399',
  '--cortex-color-text': '#F9FAFB',
  '--cortex-color-text-secondary': '#D1D5DB',
  '--cortex-color-text-disabled': '#6B7280',
  '--cortex-color-border': '#374151',
  '--cortex-color-background': '#111827',

  '--cortex-spacing-xs': '4px',
  '--cortex-spacing-sm': '8px',
  '--cortex-spacing-md': '16px',
  '--cortex-spacing-lg': '24px',
  '--cortex-spacing-xl': '32px',
  '--cortex-spacing-2xl': '48px',

  '--cortex-typography-font-family': "system-ui, -apple-system, sans-serif",
  '--cortex-typography-size-xs': '0.75rem',
  '--cortex-typography-size-sm': '0.875rem',
  '--cortex-typography-size-md': '1rem',
  '--cortex-typography-size-lg': '1.125rem',
  '--cortex-typography-size-xl': '1.25rem',
  '--cortex-typography-size-2xl': '1.5rem',
  '--cortex-typography-size-3xl': '1.875rem',
  '--cortex-typography-size-4xl': '2.25rem',
  '--cortex-typography-weight-normal': '400',
  '--cortex-typography-weight-medium': '500',
  '--cortex-typography-weight-semibold': '600',
  '--cortex-typography-weight-bold': '700',
  '--cortex-typography-line-height-tight': '1.25',
  '--cortex-typography-line-height-normal': '1.5',
  '--cortex-typography-line-height-relaxed': '1.75',

  '--cortex-border-radius-none': '0',
  '--cortex-border-radius-sm': '4px',
  '--cortex-border-radius-md': '8px',
  '--cortex-border-radius-lg': '12px',
  '--cortex-border-radius-full': '9999px',

  '--cortex-shadow-none': 'none',
  '--cortex-shadow-sm': '0 1px 2px rgba(0,0,0,0.3)',
  '--cortex-shadow-md': '0 4px 6px rgba(0,0,0,0.4)',
  '--cortex-shadow-lg': '0 10px 15px rgba(0,0,0,0.5)',

  '--cortex-transition-duration-fast': '150ms',
  '--cortex-transition-duration-normal': '250ms',
  '--cortex-transition-duration-slow': '350ms',
  '--cortex-transition-easing': 'ease-in-out',

  '--cortex-z-index-base': '0',
  '--cortex-z-index-dropdown': '100',
  '--cortex-z-index-sticky': '200',
  '--cortex-z-index-fixed': '300',
  '--cortex-z-index-modal': '400',
  '--cortex-z-index-popover': '500',
  '--cortex-z-index-tooltip': '600',
};

// ── Theme Definition ─────────────────────────────────────────────────────────

export interface ThemeDefinition {
  id: string;
  name: string;
  is_dark: boolean;
  tokens: Partial<ThemeTokens>;
}

// ── Accessibility Overrides ──────────────────────────────────────────────────

export interface AccessibilitySettings {
  high_contrast: boolean;
  reduce_motion: boolean;
  font_scale: number; // 100-200
  focus_indicator_color: string;
}

export const DEFAULT_ACCESSIBILITY: AccessibilitySettings = {
  high_contrast: false,
  reduce_motion: false,
  font_scale: 100,
  focus_indicator_color: '#4F46E5',
};

// ── Theme Resolver ───────────────────────────────────────────────────────────

/**
 * Resolve the final set of tokens by merging:
 * 1. Base theme tokens (light or dark)
 * 2. Custom theme overrides (if any)
 * 3. Accessibility overrides (high contrast, reduce motion)
 */
export function resolveTokens(
  baseTokens: ThemeTokens,
  customOverrides?: Partial<ThemeTokens>,
  accessibility?: AccessibilitySettings,
): ThemeTokens {
  // Start with base
  const resolved = { ...baseTokens };

  // Apply custom overrides
  if (customOverrides) {
    for (const [key, value] of Object.entries(customOverrides)) {
      if (value !== undefined) {
        (resolved as Record<string, string>)[key] = value;
      }
    }
  }

  // Apply accessibility overrides
  if (accessibility) {
    if (accessibility.high_contrast) {
      const isDark = baseTokens === DARK_TOKENS || baseTokens['--cortex-color-background'] === '#111827';
      resolved['--cortex-color-text'] = isDark ? '#FFFFFF' : '#000000';
      resolved['--cortex-color-background'] = isDark ? '#000000' : '#FFFFFF';
      resolved['--cortex-color-surface'] = isDark ? '#000000' : '#FFFFFF';
      resolved['--cortex-color-surface-raised'] = isDark ? '#111111' : '#F5F5F5';
      resolved['--cortex-color-border'] = isDark ? '#FFFFFF' : '#000000';
      resolved['--cortex-color-text-secondary'] = isDark ? '#CCCCCC' : '#333333';
    }

    if (accessibility.reduce_motion) {
      resolved['--cortex-transition-duration-fast'] = '0ms';
      resolved['--cortex-transition-duration-normal'] = '0ms';
      resolved['--cortex-transition-duration-slow'] = '0ms';
    }
  }

  return resolved;
}

/**
 * Apply resolved tokens to the document root as CSS custom properties.
 */
export function applyTokens(tokens: ThemeTokens, root: HTMLElement = document.documentElement): void {
  for (const [key, value] of Object.entries(tokens)) {
    root.style.setProperty(key, value);
  }
}

/**
 * Apply font scale as root font-size percentage.
 */
export function applyFontScale(scale: number, root: HTMLElement = document.documentElement): void {
  root.style.fontSize = `${scale / 100}rem`;
  // Override: since 1rem = 16px default, we want scale% of 16px
  // Actually: set percentage on root so all rem units scale
  root.style.fontSize = `${scale}%`;
}

// ── Custom Theme Validation ──────────────────────────────────────────────────

/** Patterns that are NOT allowed in custom theme values (CSS injection prevention). */
const FORBIDDEN_PATTERNS = [
  /url\s*\(/i,
  /expression\s*\(/i,
  /javascript\s*:/i,
  /@import/i,
  /<\/?script/i,
  /behavior\s*:/i,
  /-moz-binding/i,
];

/**
 * Validate a custom theme's token values.
 * Returns an array of error messages (empty if valid).
 */
export function validateCustomTheme(tokens: Partial<ThemeTokens>): string[] {
  const errors: string[] = [];

  for (const [key, value] of Object.entries(tokens)) {
    if (value === undefined) continue;

    // Check key is a valid token name
    if (!key.startsWith('--cortex-')) {
      errors.push(`Invalid token name: ${key}`);
      continue;
    }

    if (typeof value !== 'string') {
      errors.push(`Token ${key} must be a string value`);
      continue;
    }

    // Check for forbidden patterns
    for (const pattern of FORBIDDEN_PATTERNS) {
      if (pattern.test(value)) {
        errors.push(`Token ${key} contains forbidden pattern: ${pattern.source}`);
      }
    }
  }

  return errors;
}
