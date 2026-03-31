# 16 — Theme, Design Tokens, and UI System

## 1. Purpose
Define the design token system that provides consistent theming across CortexOS, supporting light/dark/custom themes with runtime switching and per-app overrides.

## 2. Scope
- Design token categories and values
- Theme definitions (light, dark, custom)
- CSS custom property implementation
- Runtime theme switching
- Per-app token overrides
- Token schema validation

## 3. Out of Scope
- Component library implementation (implementation detail, not spec)
- Animation library (owned by spec 15 reduce-motion integration)
- Icon system (v1 — SVG icons bundled with apps)

## 4. Objectives
1. Every visual property uses a design token — no hardcoded colors/spacing in app code.
2. Theme switching is instant (CSS custom properties, no reload).
3. All tokens have semantic names, not color names (e.g., `--cortex-color-primary` not `--cortex-blue`).

## 5. User-Visible Behavior

| Action | Outcome |
|---|---|
| User selects Light theme | Bright color scheme applied instantly |
| User selects Dark theme | Dark color scheme applied instantly |
| User selects Custom theme | User-defined overrides applied |
| App uses per-app override | App renders with its custom tokens |

## 6. System Behavior

### 6.1 Token Naming Convention
```
--cortex-{category}-{variant}-{state}

Examples:
--cortex-color-primary                    (default state)
--cortex-color-primary-hover              (hover state)
--cortex-color-primary-active             (pressed state)
--cortex-color-primary-disabled           (disabled state)
--cortex-spacing-md                       (no variant)
--cortex-typography-size-lg               (variant)
--cortex-border-radius-md                 (no state)
```

### 6.2 Token Categories

#### Colors
| Token | Light | Dark |
|---|---|---|
| `--cortex-color-primary` | #4F46E5 | #818CF8 |
| `--cortex-color-primary-hover` | #4338CA | #6366F1 |
| `--cortex-color-primary-active` | #3730A3 | #4F46E5 |
| `--cortex-color-primary-disabled` | #A5B4FC | #4338CA |
| `--cortex-color-secondary` | #7C3AED | #A78BFA |
| `--cortex-color-surface` | #FFFFFF | #1A1A2E |
| `--cortex-color-surface-raised` | #F9FAFB | #16213E |
| `--cortex-color-error` | #DC2626 | #F87171 |
| `--cortex-color-warning` | #F59E0B | #FBBF24 |
| `--cortex-color-success` | #10B981 | #34D399 |
| `--cortex-color-text` | #111827 | #F9FAFB |
| `--cortex-color-text-secondary` | #6B7280 | #D1D5DB |
| `--cortex-color-text-disabled` | #9CA3AF | #6B7280 |
| `--cortex-color-border` | #E5E7EB | #374151 |
| `--cortex-color-background` | #F9FAFB | #111827 |

#### Spacing
| Token | Value |
|---|---|
| `--cortex-spacing-xs` | 4px |
| `--cortex-spacing-sm` | 8px |
| `--cortex-spacing-md` | 16px |
| `--cortex-spacing-lg` | 24px |
| `--cortex-spacing-xl` | 32px |
| `--cortex-spacing-2xl` | 48px |

#### Typography
| Token | Value |
|---|---|
| `--cortex-typography-font-family` | system-ui, -apple-system, sans-serif |
| `--cortex-typography-size-xs` | 12px |
| `--cortex-typography-size-sm` | 14px |
| `--cortex-typography-size-md` | 16px |
| `--cortex-typography-size-lg` | 18px |
| `--cortex-typography-size-xl` | 20px |
| `--cortex-typography-size-2xl` | 24px |
| `--cortex-typography-size-3xl` | 30px |
| `--cortex-typography-size-4xl` | 36px |
| `--cortex-typography-weight-normal` | 400 |
| `--cortex-typography-weight-medium` | 500 |
| `--cortex-typography-weight-semibold` | 600 |
| `--cortex-typography-weight-bold` | 700 |
| `--cortex-typography-line-height-tight` | 1.25 |
| `--cortex-typography-line-height-normal` | 1.5 |
| `--cortex-typography-line-height-relaxed` | 1.75 |

#### Border Radius
| Token | Value |
|---|---|
| `--cortex-border-radius-none` | 0 |
| `--cortex-border-radius-sm` | 4px |
| `--cortex-border-radius-md` | 8px |
| `--cortex-border-radius-lg` | 12px |
| `--cortex-border-radius-full` | 9999px |

#### Shadows
| Token | Light | Dark |
|---|---|---|
| `--cortex-shadow-none` | none | none |
| `--cortex-shadow-sm` | 0 1px 2px rgba(0,0,0,0.05) | 0 1px 2px rgba(0,0,0,0.3) |
| `--cortex-shadow-md` | 0 4px 6px rgba(0,0,0,0.07) | 0 4px 6px rgba(0,0,0,0.4) |
| `--cortex-shadow-lg` | 0 10px 15px rgba(0,0,0,0.1) | 0 10px 15px rgba(0,0,0,0.5) |

#### Transitions
| Token | Value |
|---|---|
| `--cortex-transition-duration-fast` | 150ms |
| `--cortex-transition-duration-normal` | 250ms |
| `--cortex-transition-duration-slow` | 350ms |
| `--cortex-transition-easing` | ease-in-out |

#### Z-Index
| Token | Value |
|---|---|
| `--cortex-z-index-base` | 0 |
| `--cortex-z-index-dropdown` | 100 |
| `--cortex-z-index-sticky` | 200 |
| `--cortex-z-index-fixed` | 300 |
| `--cortex-z-index-modal` | 400 |
| `--cortex-z-index-popover` | 500 |
| `--cortex-z-index-tooltip` | 600 |

### 6.3 High Contrast Overrides
When `accessibility.high_contrast = true`:
- Text color forced to #000000 (light) or #FFFFFF (dark)
- Focus indicator: 3px solid #000000 or #FFFFFF
- Minimum 4.5:1 contrast enforced
- Background simplified to pure white/black

### 6.4 Reduce Motion Overrides
When `accessibility.reduce_motion = true`:
- All transition durations set to 0ms
- `--cortex-transition-duration-fast`: 0ms
- `--cortex-transition-duration-normal`: 0ms
- `--cortex-transition-duration-slow`: 0ms

## 7. Architecture
```
┌─────────────────────────────────────┐
│         Theme System                │
│                                     │
│  ┌───────────────────────────────┐  │
│  │  Token Registry (JSON schema) │  │
│  │  (all tokens, types, values)  │  │
│  └───────────────┬───────────────┘  │
│  ┌───────────────┴───────────────┐  │
│  │  Theme Resolver               │  │
│  │  (base theme + user overrides │  │
│  │   + accessibility overrides)  │  │
│  └───────────────┬───────────────┘  │
│  ┌───────────────┴───────────────┐  │
│  │  CSS Custom Properties        │  │
│  │  (applied to :root)           │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

## 8. Data Model
```typescript
interface ThemeTokens {
  [tokenName: string]: string | number;  // "--cortex-color-primary": "#4F46E5"
}

interface ThemeDefinition {
  id: string;                    // "light", "dark", "custom-{uuid}"
  name: string;
  is_dark: boolean;
  tokens: Partial<ThemeTokens>;  // Override base tokens
}

// Default themes:
const LIGHT_THEME: ThemeDefinition = { id: "light", name: "Light", is_dark: false, tokens: { /* all light values */ } };
const DARK_THEME: ThemeDefinition = { id: "dark", name: "Dark", is_dark: true, tokens: { /* all dark values */ } };
```

## 9. Public Interfaces
- No dedicated REST API (theme selection via settings API: `display.theme`)
- Theme tokens applied as CSS custom properties on `:root`
- Per-app overrides applied as CSS custom properties on app container element

## 10. Internal Interfaces
- Reads `display.theme` from settings (spec 05)
- Reads accessibility settings for high-contrast and reduce-motion overrides
- Desktop shell (spec 07) applies tokens to `:root`
- All apps consume tokens via CSS custom properties

## 11. State Management
- Active theme stored in settings (`display.theme`)
- Custom themes stored in settings (`display.custom_themes`)
- Token values resolved at theme-switch time and applied to DOM
- Changes persisted immediately via settings API

## 12. Failure Modes and Error Handling
| Failure | Handling |
|---|---|
| Invalid token value | Fall back to light theme default |
| Missing token | Use light theme default |
| Custom theme parse error | Fall back to light, show warning |

## 13. Security and Permissions
- Theme selection is a user preference, no permission needed
- Custom theme values validated (CSS injection prevention)
- Only CSS custom property names/values allowed (no `url()`, `expression()`, etc.)

## 14. Performance Requirements
| Metric | Target |
|---|---|
| Theme switch rendering | < 100ms (CSS-only, no repaint) |
| Token resolution | < 1ms |
| Custom theme load | < 50ms |

## 15. Accessibility Requirements
- High contrast mode enforced through token overrides
- Font scaling via root font-size (all tokens in rem where applicable)
- Reduced motion via transition duration overrides
- Theme must meet WCAG AA contrast ratios

## 16. Observability and Logging
- Theme changes logged at INFO: {old_theme, new_theme}
- Custom theme validation failures logged at WARN
- Token fallback usage logged at DEBUG

## 17. Testing Requirements
- Unit: token resolution (base → user override → accessibility override)
- Unit: high contrast overrides applied correctly
- Visual: light/dark theme renders correctly across all apps
- E2E: theme switch applies instantly without flicker
- Accessibility: contrast ratios validated

## 18. Acceptance Criteria
- [ ] All tokens defined with light and dark values
- [ ] Theme switching instant (no page reload)
- [ ] Tokens used by all apps (no hardcoded colors)
- [ ] High contrast mode enforces 4.5:1 ratios
- [ ] Reduce motion disables all transitions
- [ ] Font scaling works via root font-size
- [ ] Custom themes validated (no CSS injection)
- [ ] Per-app overrides scoped correctly

## 19. Build Order and Dependencies
**Layer 10**. Depends on: 01, 02, 05 (settings for theme selection)

## 20. Non-Goals and Anti-Patterns
- No component library (v1 — tokens only)
- No theme editor UI (v1 — JSON-based custom themes)
- No animation library
- NEVER hardcode colors, spacing, or typography in component code
- NEVER use color names as token names (use semantic names)
- NEVER allow CSS injection in custom themes

## 21. Implementation Instructions for Claude Code / Codex
1. Define all tokens as JSON schema (light + dark values).
2. Build theme resolver: merge base tokens → user theme → accessibility overrides → per-app.
3. Apply tokens to `:root` as CSS custom properties on theme change.
4. Build theme switch handler in desktop shell.
5. Validate custom themes: check all values are valid CSS, no injection.
6. Write tests: token resolution, contrast ratios, theme switch rendering.
