# 17a. Calculator App

## 1. Purpose

The Calculator app provides a system-calculator for CortexOS, supporting standard arithmetic and scientific-mode calculations with expression input, calculation history, and clipboard integration. It serves as the reference implementation for a lightweight first-party app with no file dependencies.

## 2. Scope

- Standard calculator mode: basic arithmetic operations (+, -, *, /, %, parentheses).
- Scientific calculator mode: trigonometric functions, logarithms, exponents, roots, constants (pi, e), factorial, modulo.
- Expression-based input: user types a full expression, result is evaluated on Enter.
- Calculation history: scrollable list of previous calculations and results.
- Copy result to clipboard.
- Keyboard shortcuts for all operations.
- AI integration: "Explain this calculation" action.
- App location: `apps/calculator-app`.

## 3. Out of Scope

- Graphing or plotting capabilities.
- Unit conversion (belongs in a separate utility if needed).
- Currency conversion (requires network data).
- Programmable or custom functions.
- Persistent save/load of calculations (history is session-scoped only).
- Calculator widgets on the desktop (future consideration).

## 4. Objectives

1. Provide a reliable, fast calculator for everyday and scientific use.
2. Validate the first-party app lifecycle pattern (spec 17) with the simplest non-trivial app.
3. Demonstrate keyboard-first input with a purely frontend, no-file-operations app.
4. Expose AI surface with domain-specific actions (explain calculation, convert notation).

## 5. User-Visible Behavior

### 5.1 Modes

- **Standard mode**: Number pad layout, basic operations, percentage, sign toggle, clear/backspace.
- **Scientific mode**: Extended button panel with sin, cos, tan, log, ln, sqrt, pow, factorial, parentheses, constants (pi, e). Toggled via a mode switch button or `Ctrl+Shift+S`.

### 5.2 Expression Input

- A single-line text input field at the top displays the current expression.
- User can type freely or click buttons to build the expression.
- Pressing Enter evaluates the expression and displays the result below the input.
- The previous expression is replaced by the result for chaining: pressing an operator after a result continues the calculation.
- Pressing Enter on an already-evaluated expression does nothing (no re-evaluation).

### 5.3 History

- A collapsible history panel (below the expression input, above the keypad) shows previous calculations.
- Each history entry shows: expression, equals sign, result.
- Clicking a history entry loads the result into the expression input for continued calculation.
- History is session-scoped: cleared when the app closes. User can clear history manually via button or `Ctrl+Shift+H`.
- Maximum history entries: 100. Oldest entries are pruned.

### 5.4 Clipboard

- Clicking the result copies it to clipboard. A toast confirms the copy.
- Keyboard shortcut: `Ctrl+C` when focus is on the result field copies the result.
- `Ctrl+V` pastes a number from clipboard into the expression input.

### 5.5 Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `0-9`, `.`, `+`, `-`, `*`, `/`, `%`, `(`, `)` | Insert character into expression |
| `Enter` | Evaluate expression |
| `Backspace` | Delete last character |
| `Escape` | Clear expression |
| `Ctrl+C` | Copy result |
| `Ctrl+V` | Paste into expression |
| `Ctrl+Shift+S` | Toggle standard/scientific mode |
| `Ctrl+Shift+H` | Clear history |
| `Ctrl+Shift+A` | Open AI assistant panel |

## 6. System Behavior

### 6.1 Evaluation Engine

- Expression evaluation is performed entirely client-side using a deterministic parser.
- The parser is a recursive-descent expression parser implemented in TypeScript.
- Operator precedence: parentheses > functions > exponentiation > multiplication/division > addition/subtraction.
- Division by zero returns "Error: Division by zero" displayed in the result field.
- Invalid expressions return "Error: Invalid expression".
- Results are displayed with up to 15 significant digits. Trailing zeros are trimmed.
- Very large or very small numbers display in scientific notation (e.g., 1.23e+10).
- Results that are integers display without decimal point.

### 6.2 Scientific Functions

All trigonometric functions accept and return radians by default. A DEG/RAD toggle switches between degree and radian mode.

Supported functions:
- sin, cos, tan, asin, acos, atan
- log (base 10), ln (natural log)
- sqrt, cbrt
- pow(x, y)
- factorial(n) -- integer n only, max 170
- abs
- Constants: pi (3.14159265358979), e (2.71828182845905)

### 6.3 App Lifecycle

- Single-instance app. Launching a second instance brings the existing window to focus.
- No unsaved state concern. History and expression are session-scoped.
- State persisted across hot-reload: current expression, current mode (standard/scientific), DEG/RAD toggle.
- State not persisted across sessions. Fresh start on each launch.

### 6.4 File Associations

None. Calculator does not open or save files.

## 7. Architecture

```
apps/calculator-app/
  manifest.json
  package.json
  src/
    main.ts                  # Registers app with runtime
    App.tsx                  # Root component, mode switcher
    components/
      ExpressionInput.tsx    # Expression text input
      ResultDisplay.tsx      # Result display with copy
      Keypad.tsx             # Button grid (standard)
      ScientificKeypad.tsx   # Extended button grid
      HistoryPanel.tsx       # Collapsible history list
      ModeToggle.tsx         # Standard/Scientific switch
    services/
      evaluator.ts           # Expression parser and evaluator
    hooks/
      useHistory.ts          # History state management
      useExpression.ts       # Expression input and evaluation
    ai/
      context.ts             # Provides current expression + result to AI
      actions.ts             # "Explain calculation" action
    types.ts
  tests/
    unit/
      evaluator.test.ts      # Expression parser tests
      history.test.ts        # History management tests
    integration/
      lifecycle.test.ts      # App launch/unmount
      keyboard.test.ts       # Keyboard input flows
```

No Rust backend crate needed. All logic is client-side TypeScript.

## 8. Data Model

### 8.1 History Entry

```typescript
interface HistoryEntry {
  id: string;              // UUID
  expression: string;      // Raw expression string
  result: string;          // Formatted result string
  timestamp: string;       // ISO 8601
  mode: "standard" | "scientific";
}
```

### 8.2 App State

```typescript
interface CalculatorState {
  expression: string;
  result: string | null;
  mode: "standard" | "scientific";
  angleUnit: "deg" | "rad";
  history: HistoryEntry[];
  historyPanelOpen: boolean;
}
```

### 8.3 Manifest

```typescript
{
  id: "com.cortexos.calculator",
  name: "Calculator",
  version: "1.0.0",
  description: "Standard and scientific calculator",
  firstParty: true,
  bundled: true,
  entry: { frontend: "src/main.ts" },
  window: {
    defaultWidth: 360,
    defaultHeight: 520,
    minWidth: 300,
    minHeight: 400,
    resizable: true,
    singleInstance: true
  },
  permissions: {
    required: ["runtime.state", "runtime.lifecycle"],
    optional: ["clipboard.read", "clipboard.write", "ai.context", "ai.invoke"]
  },
  ai: {
    surfaces: { assistantPanel: true, contextMenu: false, inlineSuggestions: false },
    contextProviders: ["calculator-context"],
    actions: [
      {
        id: "explain-calculation",
        label: "Explain this calculation",
        description: "Provides a step-by-step explanation of the current expression and result",
        confirmationRequired: false,
        destructive: false
      }
    ]
  },
  accessibility: { highContrastSupport: true, screenReaderSupport: true, keyboardNavigation: true },
  category: "utilities"
}
```

## 9. Public Interfaces

### 9.1 AI Context

```typescript
interface CalculatorAIContext {
  expression: string;
  result: string | null;
  mode: "standard" | "scientific";
  angleUnit: "deg" | "rad";
}
```

### 9.2 Commands Exposed

None. Calculator does not expose commands to other apps.

## 10. Internal Interfaces

### 10.1 Evaluator Service

```typescript
interface Evaluator {
  evaluate(expression: string, angleUnit: "deg" | "rad"): EvaluationResult;
}

interface EvaluationResult {
  success: boolean;
  value?: number;
  error?: "division_by_zero" | "invalid_expression" | "overflow" | "undefined_function";
}
```

### 10.2 History Hook

```typescript
interface UseHistory {
  entries: HistoryEntry[];
  addEntry(expression: string, result: string, mode: string): void;
  clearHistory(): void;
  selectEntry(id: string): HistoryEntry | undefined;
}
```

## 11. State Management

- **Ephemeral**: Keypad button hover/active states, history panel open/close.
- **Session**: Current expression, result, mode, angle unit, history. Persisted via cortex-runtime session state so it survives hot-reload.
- **Persistent**: None. Calculator starts fresh each session.
- State key: `com.cortexos.calculator.session`.

## 12. Failure Modes and Error Handling

| Failure | Behavior |
|---------|----------|
| Division by zero | Display "Cannot divide by zero" in result field. Log as info. |
| Invalid expression | Display "Invalid expression" in result field. Log as info. |
| Overflow (result exceeds Number.MAX_VALUE) | Display "Result too large". Log as info. |
| Factorial of non-integer or negative | Display "Invalid input for factorial". |
| Clipboard write failure | Show toast: "Could not copy to clipboard". |
| AI provider unavailable | AI panel shows "AI unavailable" message. Calculator remains fully functional. |

All errors are non-blocking. Calculator never crashes due to evaluation errors.

## 13. Security and Permissions

- No filesystem access needed.
- No network access needed.
- Optional clipboard permissions: user is prompted on first copy/paste attempt. If denied, copy/paste buttons are disabled with a tooltip explaining the permission is needed.
- AI permissions optional: AI panel shows permission request if not granted.
- Expression input is never evaluated via `eval()`. The custom parser prevents code injection.

## 14. Performance Requirements

- Expression evaluation must complete within 1 ms for any valid expression.
- UI must respond to button clicks within 16 ms (one frame).
- History rendering must handle 100 entries without scroll jank.
- Startup first meaningful paint: under 300 ms (simplest first-party app).
- Bundle size: under 100 KB gzipped.

## 15. Accessibility Requirements

- All keypad buttons have accessible labels (e.g., "plus", "multiply", "open parenthesis").
- Result field has `role="status"` with `aria-live="polite"` to announce results to screen readers.
- Expression input has `aria-label="Calculator expression"`.
- History entries are in a list with `role="list"` and `role="listitem"`.
- Mode toggle has `aria-pressed` state.
- Focus management: after evaluation, focus remains on expression input.
- Color is not the sole indicator of mode (standard/scientence differentiated by label and button layout).

## 16. Observability and Logging

Logged events:
- `calc.launched` (info) -- App opened with mode.
- `calc.evaluated` (info) -- Expression evaluated, includes mode and result type (number/error). Does not log the expression or result values.
- `calc.mode_changed` (info) -- Mode switched.
- `calc.ai.explain_invoked` (info) -- AI explain action triggered.
- `calc.error` (warn) -- Evaluation error (error type only, no expression).

No PII is logged. Expressions and results are never included in log payloads.

## 17. Testing Requirements

### 17.1 Unit Tests

- Evaluator: all arithmetic operators, precedence, parentheses, scientific functions, edge cases (division by zero, overflow, invalid syntax).
- History: add, clear, select, max 100 entries pruning.
- Expression hook: input accumulation, backspace, clear, paste.

### 17.2 Integration Tests

- Full calculation flow: type expression, evaluate, verify result, verify history entry.
- Mode switching: toggle between standard and scientific, verify button visibility.
- Clipboard: copy result, verify clipboard content.
- Keyboard-only flow: all operations via keyboard.

### 17.3 Accessibility Tests

- AX tree validation for both modes.
- Keyboard navigation through all buttons and controls.
- Screen reader announcement of results.

## 18. Acceptance Criteria

- [ ] Standard mode evaluates all basic arithmetic correctly.
- [ ] Scientific mode evaluates all listed functions correctly.
- [ ] DEG/RAD toggle affects trigonometric results correctly.
- [ ] History displays up to 100 entries, sorted newest-first.
- [ ] Clicking history entry loads result.
- [ ] Copy to clipboard works with permission granted.
- [ ] Keyboard shortcuts work for all documented actions.
- [ ] Expression errors display user-friendly messages, never raw errors.
- [ ] App launches in under 300 ms.
- [ ] All three themes render correctly.
- [ ] Screen reader announces results.
- [ ] AI panel opens and provides expression context.
- [ ] No use of `eval()` or `Function()` constructor.
- [ ] Unit test coverage >= 90% (evaluator is critical path).

## 19. Build Order and Dependencies
**Layer 11**. Depends on: 09 (app runtime), 16 (theme tokens), 17 (first-party app parent)

### Prerequisites

- Spec 17 parent (first-party app framework).
- `@cortexos/ui-components` (shared UI library).
- `@cortexos/runtime-client` (for state persistence).
- `@cortexos/ai-client` (for AI surface).
- `@cortexos/theme` (design token consumer).

### Build Position

Calculator is the **second** first-party app to build (after Clock Utilities), serving as a validation point for the app lifecycle and AI integration patterns.

No Rust crate needed. Pure frontend app.

## 20. Non-Goals and Anti-Patterns

### Non-Goals

- Graphing calculator capabilities.
- Currency or unit conversion.
- Persistent calculation history across sessions.
- Programmable functions or macros.

### Anti-Patterns

- Using `eval()` or `new Function()` for expression evaluation.
- Storing expressions or results in log payloads (privacy).
- Depending on a math library that is not deterministic across platforms (use a pure JS/TS parser).
- Building a custom UI for copy/paste instead of using the system clipboard API.

## 21. Implementation Instructions for Claude Code / Codex

### Subsystem Ownership

- Calculator owns: expression parser, evaluation engine, UI layout, history management.
- Calculator does not own: clipboard system, AI runtime, theme system, window management.

### Recommended Implementation Order

1. Create `manifest.json` and validate against FirstPartyAppManifest schema.
2. Implement `services/evaluator.ts` -- the expression parser. This is the core logic. Write comprehensive unit tests first (test-driven).
3. Implement `App.tsx` with standard mode keypad and expression input.
4. Add result display with copy functionality.
5. Add history panel.
6. Add scientific mode toggle and scientific keypad.
7. Add DEG/RAD toggle.
8. Integrate `@cortexos/runtime-client` for session state persistence.
9. Integrate `@cortexos/ai-client` for AI surface and "Explain calculation" action.
10. Add keyboard shortcuts.
11. Accessibility audit and fixes.
12. Theme verification (light, dark, high-contrast).

### What Can Be Stubbed Initially

- AI action handler can return a placeholder explanation initially.
- Clipboard integration can use a stub until the system clipboard API is ready.

### What Must Be Real in V1

- Full expression parser (no shortcuts, no eval).
- Both standard and scientific modes.
- History panel with 100-entry limit.
- Copy result to clipboard.
- Keyboard shortcuts.
- Theme support.
- Accessibility (keyboard navigation, screen reader).

### What Cannot Be Inferred

- Exact button layout and sizing (follow design tokens from spec 16).
- Color of error states (consume design token `--color-error`).
- Default window size (360x520 per manifest).

### Stop Conditions

1. All unit tests pass with >= 90% coverage.
2. Integration tests for full calculation flow pass.
3. Accessibility tests pass.
4. Manifest validates.
5. No `eval()` usage confirmed by code review and linter rule.
6. Both modes functional and correctly computing results.
7. AI panel opens and provides context (even if AI returns placeholder response).

### Testing Gates

- Unit tests for evaluator must pass before any UI work begins.
- Keyboard navigation test must pass before merge.
- Performance benchmark: 1000 random expressions evaluated in under 1 second total.
