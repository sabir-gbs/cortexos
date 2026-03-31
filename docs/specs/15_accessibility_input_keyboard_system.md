# 15 — Accessibility, Input, and Keyboard System

## 1. Purpose
Ensure CortexOS is fully usable with keyboard-only navigation, screen readers, and other assistive technologies. Define the keyboard shortcuts registry, focus management, and accessibility settings.

## 2. Scope
- Full keyboard navigation for all UI elements
- Keyboard shortcuts registry (global + per-app)
- Shortcut conflict resolution
- ARIA roles, labels, and live regions
- High contrast mode enforcement
- Reduced motion support
- Font size scaling
- Focus indicators
- Tab order management
- Accessibility settings

## 3. Out of Scope
- Screen reader implementation (relies on browser's built-in)
- Custom input methods (relies on browser IME)
- Voice control

## 4. Objectives
1. Every interactive element reachable via keyboard.
2. All dynamic content changes announced to screen readers.
3. Minimum WCAG AA compliance (4.5:1 contrast, keyboard operable).
4. Accessibility settings take effect immediately.

## 5. User-Visible Behavior

| Action | Outcome |
|---|---|
| User presses Tab | Focus moves to next interactive element |
| User presses Enter/Space | Focused element activated |
| User presses Escape | Modal dismissed, menu closed |
| User enables high contrast | Colors switch to high-contrast palette |
| User enables reduce motion | Animations replaced with instant transitions |
| User increases font scale | All text scales, UI reflows without overflow |
| User presses app shortcut | Shortcut executes if no conflict with system |

## 6. System Behavior

### 6.1 Keyboard Navigation
- Tab: move focus to next interactive element (DOM order)
- Shift+Tab: move focus backward
- Enter/Space: activate focused element (buttons, links, menu items)
- Escape: close topmost modal/menu/dialog
- Arrow keys: navigate within composite widgets (lists, menus, tabs)
- Home/End: jump to first/last item in list

### 6.2 Keyboard Shortcuts Registry
```typescript
interface KeyboardShortcut {
  id: string;
  key_combo: string;         // e.g. "Ctrl+S", "Ctrl+Space"
  description: string;       // Human-readable: "Save file"
  owner: string;             // "system" or app_id
  scope: "global" | "app";
  when_focused?: string;     // Only active when this element focused
}

// Registration:
// System shortcuts registered at startup (always win conflicts)
// App shortcuts registered via API when app launches
// If app shortcut conflicts with system → app warned, system shortcut used
```

### 6.3 ARIA Requirements
- All interactive elements have `role` and `aria-label`
- Dynamic content uses `aria-live="polite"` for updates, `"assertive"` for alerts
- Dialogs have `role="dialog"`, `aria-modal="true"`, `aria-labelledby`
- Tabs: `role="tablist"`, `role="tab"`, `role="tabpanel"`
- Loading states: `aria-busy="true"`
- Disabled elements: `aria-disabled="true"`
- Expanded/collapsed: `aria-expanded`

### 6.4 High Contrast Mode
- Minimum 4.5:1 contrast ratio for all text (WCAG AA)
- Minimum 3:1 for large text (18px+ bold, 24px+)
- Focus indicators: minimum 3:1 contrast against background
- Enforced via theme tokens (spec 16): high-contrast token values used when enabled

### 6.5 Reduced Motion
- When enabled: all CSS transitions set to `0ms`
- Animation replaced with instant state changes
- No auto-playing animations
- Loading spinners replaced with static "Loading..." text
- Respects `prefers-reduced-motion` media query by default

### 6.6 Font Scaling
- Range: 100% to 200% in 10% steps
- All text uses `rem` units (root font-size scaled)
- UI reflows without horizontal overflow
- No text truncation that hides content

### 6.7 Focus Indicators
- Visible 2px outline on all focused elements
- Color: configurable via `accessibility.focus_indicator_color` (default: system accent)
- Focus ring offset: 2px from element
- Never use `outline: none` without replacement

### 6.8 Tab Order
- Follows DOM order by default
- Skip links: "Skip to main content" link at top of each page
- Focus trap in modals: Tab cycles within modal, Escape closes
- Negative tabindex never used for interactive elements

## 7. Architecture
Accessibility is not a separate service — it's cross-cutting behavior enforced through:
- Theme tokens (spec 16) for high contrast and font scaling
- UI component library for ARIA compliance
- Keyboard shortcuts as a shared registry module
- Settings for user preferences

## 8. Data Model
```typescript
interface AccessibilitySettings {
  keyboard_nav_enabled: boolean;       // Default: true
  high_contrast: boolean;              // Default: false
  reduce_motion: boolean;              // Default: false
  font_scale: number;                  // Default: 100, Range: 100-200, Step: 10
  focus_indicator_color: string;       // Default: "#4F46E5"
  screen_reader_announcements: boolean;// Default: true
}

// Setting keys (in accessibility.* namespace):
// accessibility.keyboard_nav_enabled
// accessibility.high_contrast
// accessibility.reduce_motion
// accessibility.font_scale
// accessibility.focus_indicator_color
// accessibility.screen_reader_announcements
```

## 9. Public Interfaces
- No dedicated REST API (reads/writes via settings API in spec 05)
- Keyboard shortcut registration via command bus

## 10. Internal Interfaces
- Theme system (spec 16) reads accessibility settings for token values
- All UI components must accept and apply focus, ARIA attributes
- Settings changes propagated via WebSocket to all clients

## 11. State Management
- Settings stored in cortex-settings (persisted)
- Keyboard shortcuts registered in memory, cleared on app stop
- Font scale applied as CSS root font-size
- High contrast applied as theme override

## 12. Failure Modes and Error Handling
| Failure | Handling |
|---|---|
| Font scale causes overflow | UI must reflow, never overflow |
| Shortcut registration fails | Log warning, app continues without shortcut |
| ARIA live region update fails | Fallback: update element text content |

## 13. Security and Permissions
- Accessibility settings are user-level, no special permissions needed
- App keyboard shortcuts only active when app focused (unless global)

## 14. Performance Requirements
| Metric | Target |
|---|---|
| Shortcut registration | < 1ms |
| Font scale change rendering | < 100ms |
| Focus indicator rendering | 0ms (CSS only) |

## 15. Accessibility Requirements
This IS the accessibility spec. Self-referential compliance:
- This spec itself must be keyboard-operable when implemented
- Settings for accessibility must be accessible (keyboard-navigable settings app)

## 16. Observability and Logging
- Accessibility setting changes logged at INFO
- Shortcut conflicts logged at WARN
- Font scale overflow warnings at WARN

## 17. Testing Requirements
- E2E: navigate entire OS using only keyboard
- E2E: enable high contrast → verify 4.5:1 ratios
- E2E: enable reduce motion → verify no animations
- E2E: font scale to 200% → verify no overflow
- Unit: shortcut conflict resolution logic
- Accessibility audit with browser tools (axe, Lighthouse)

## 18. Acceptance Criteria
- [ ] Every interactive element reachable via Tab
- [ ] Enter/Space activates focused element
- [ ] Escape closes modals/menus
- [ ] High contrast meets 4.5:1 ratio
- [ ] Reduce motion disables all animations
- [ ] Font scaling to 200% works without overflow
- [ ] Focus indicators visible on all elements
- [ ] ARIA labels on all interactive elements
- [ ] Screen reader announces dynamic content changes
- [ ] System shortcuts always win conflicts

## 19. Build Order and Dependencies
**Layer 10**. Depends on: 01, 02, 05 (settings), 16 (theme tokens)

## 20. Non-Goals and Anti-Patterns
- No custom screen reader (rely on browser)
- No voice control (v1)
- No gesture support (v1 — browser-rendered)
- NEVER use `outline: none` without visible replacement
- NEVER rely on color alone to convey information
- NEVER use `tabindex` > 0 (causes jump disorder)
- NEVER auto-focus elements that surprise users

## 21. Implementation Instructions for Claude Code / Codex
1. Implement keyboard shortcuts registry in shared UI module.
2. Add global keyboard event handler for system shortcuts.
3. Ensure all UI components have proper ARIA attributes.
4. Implement high contrast theme tokens (spec 16 integration).
5. Implement font scaling via CSS root font-size.
6. Implement focus trap for modals.
7. Write E2E tests: keyboard-only navigation of full OS.
8. Run accessibility audit (axe-core) and fix all violations.
