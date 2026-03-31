import { describe, it, expect } from 'vitest';
import {
  LIGHT_TOKENS,
  DARK_TOKENS,
  resolveTokens,
  validateCustomTheme,
  DEFAULT_ACCESSIBILITY,
  type ThemeTokens,
} from '../theme';

describe('Theme Tokens', () => {
  it('LIGHT_TOKENS has all required color tokens', () => {
    expect(LIGHT_TOKENS['--cortex-color-primary']).toBe('#4F46E5');
    expect(LIGHT_TOKENS['--cortex-color-surface']).toBe('#FFFFFF');
    expect(LIGHT_TOKENS['--cortex-color-text']).toBe('#111827');
    expect(LIGHT_TOKENS['--cortex-color-background']).toBe('#F9FAFB');
  });

  it('DARK_TOKENS has all required color tokens', () => {
    expect(DARK_TOKENS['--cortex-color-primary']).toBe('#818CF8');
    expect(DARK_TOKENS['--cortex-color-surface']).toBe('#1A1A2E');
    expect(DARK_TOKENS['--cortex-color-text']).toBe('#F9FAFB');
    expect(DARK_TOKENS['--cortex-color-background']).toBe('#111827');
  });

  it('both themes have the same token keys', () => {
    const lightKeys = Object.keys(LIGHT_TOKENS).sort();
    const darkKeys = Object.keys(DARK_TOKENS).sort();
    expect(lightKeys).toEqual(darkKeys);
  });

  it('spacing tokens are consistent across themes', () => {
    expect(LIGHT_TOKENS['--cortex-spacing-xs']).toBe(DARK_TOKENS['--cortex-spacing-xs']);
    expect(LIGHT_TOKENS['--cortex-spacing-md']).toBe(DARK_TOKENS['--cortex-spacing-md']);
    expect(LIGHT_TOKENS['--cortex-spacing-2xl']).toBe(DARK_TOKENS['--cortex-spacing-2xl']);
  });

  it('typography tokens are consistent across themes', () => {
    expect(LIGHT_TOKENS['--cortex-typography-font-family']).toBe(DARK_TOKENS['--cortex-typography-font-family']);
    expect(LIGHT_TOKENS['--cortex-typography-size-md']).toBe(DARK_TOKENS['--cortex-typography-size-md']);
  });

  it('z-index tokens are consistent across themes', () => {
    expect(LIGHT_TOKENS['--cortex-z-index-modal']).toBe(DARK_TOKENS['--cortex-z-index-modal']);
    expect(LIGHT_TOKENS['--cortex-z-index-tooltip']).toBe(DARK_TOKENS['--cortex-z-index-tooltip']);
  });
});

describe('resolveTokens', () => {
  it('returns base tokens when no overrides', () => {
    const resolved = resolveTokens(LIGHT_TOKENS);
    expect(resolved['--cortex-color-primary']).toBe(LIGHT_TOKENS['--cortex-color-primary']);
  });

  it('applies custom overrides', () => {
    const resolved = resolveTokens(LIGHT_TOKENS, {
      '--cortex-color-primary': '#FF0000',
    });
    expect(resolved['--cortex-color-primary']).toBe('#FF0000');
    // Other tokens unchanged
    expect(resolved['--cortex-color-surface']).toBe(LIGHT_TOKENS['--cortex-color-surface']);
  });

  it('applies high contrast for light theme', () => {
    const resolved = resolveTokens(LIGHT_TOKENS, undefined, {
      ...DEFAULT_ACCESSIBILITY,
      high_contrast: true,
    });
    expect(resolved['--cortex-color-text']).toBe('#000000');
    expect(resolved['--cortex-color-background']).toBe('#FFFFFF');
    expect(resolved['--cortex-color-border']).toBe('#000000');
  });

  it('applies high contrast for dark theme', () => {
    const resolved = resolveTokens(DARK_TOKENS, undefined, {
      ...DEFAULT_ACCESSIBILITY,
      high_contrast: true,
    });
    expect(resolved['--cortex-color-text']).toBe('#FFFFFF');
    expect(resolved['--cortex-color-background']).toBe('#000000');
    expect(resolved['--cortex-color-border']).toBe('#FFFFFF');
  });

  it('applies reduce motion', () => {
    const resolved = resolveTokens(LIGHT_TOKENS, undefined, {
      ...DEFAULT_ACCESSIBILITY,
      reduce_motion: true,
    });
    expect(resolved['--cortex-transition-duration-fast']).toBe('0ms');
    expect(resolved['--cortex-transition-duration-normal']).toBe('0ms');
    expect(resolved['--cortex-transition-duration-slow']).toBe('0ms');
  });

  it('combines high contrast and reduce motion', () => {
    const resolved = resolveTokens(DARK_TOKENS, undefined, {
      ...DEFAULT_ACCESSIBILITY,
      high_contrast: true,
      reduce_motion: true,
    });
    expect(resolved['--cortex-color-text']).toBe('#FFFFFF');
    expect(resolved['--cortex-transition-duration-fast']).toBe('0ms');
  });

  it('combines custom overrides with accessibility', () => {
    const resolved = resolveTokens(LIGHT_TOKENS, {
      '--cortex-color-secondary': '#00FF00',
    }, {
      ...DEFAULT_ACCESSIBILITY,
      high_contrast: true,
    });
    // Custom override applied
    expect(resolved['--cortex-color-secondary']).toBe('#00FF00');
    // Accessibility override applied
    expect(resolved['--cortex-color-text']).toBe('#000000');
  });
});

describe('validateCustomTheme', () => {
  it('accepts valid custom tokens', () => {
    const errors = validateCustomTheme({
      '--cortex-color-primary': '#FF0000',
      '--cortex-spacing-md': '32px',
    });
    expect(errors).toEqual([]);
  });

  it('rejects invalid token names', () => {
    const errors = validateCustomTheme({
      '--invalid-token': 'red',
    } as any);
    expect(errors.length).toBeGreaterThan(0);
    expect(errors[0]).toContain('Invalid token name');
  });

  it('rejects url() values (CSS injection)', () => {
    const errors = validateCustomTheme({
      '--cortex-color-primary': 'url(evil.com)',
    });
    expect(errors.length).toBeGreaterThan(0);
  });

  it('rejects expression() values', () => {
    const errors = validateCustomTheme({
      '--cortex-color-primary': 'expression(alert(1))',
    });
    expect(errors.length).toBeGreaterThan(0);
  });

  it('rejects javascript: values', () => {
    const errors = validateCustomTheme({
      '--cortex-color-primary': 'javascript:alert(1)',
    });
    expect(errors.length).toBeGreaterThan(0);
  });

  it('accepts empty custom theme', () => {
    const errors = validateCustomTheme({});
    expect(errors).toEqual([]);
  });
});
