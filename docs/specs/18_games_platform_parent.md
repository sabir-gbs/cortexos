# 18. Games Platform -- Parent Specification

## 1. Purpose

This specification defines the shared game framework, rendering loop, input handling, state serialization, and consistent UI chrome for all CortexOS games. It serves as the authoritative parent spec for child specs 18a through 18e. Each child spec owns its specific game mechanics; this document owns the shared game infrastructure, conventions, and constraints that unify all first-party games within the CortexOS environment.

## 2. Scope

This spec covers:

- Definition of a "game" as a distinct first-party app category within CortexOS.
- The shared game framework providing a rendering loop, input manager, state serializer, and pause/resume lifecycle.
- Common UI chrome requirements: title bar, pause button, score display, settings gear, help/rules overlay.
- Input contract: every game must support both keyboard and mouse controls.
- State serialization and deserialization for save/load, undo, and hot-reload persistence.
- Scoring, timing, and high-score persistence patterns.
- Permission model: games use the same permission model as all first-party apps. No hidden privileges.
- Theme support: all games must render correctly in light, dark, and high-contrast CortexOS themes.
- Help and rules overlay: every game must include a built-in rules/help screen accessible via toolbar or keyboard shortcut.
- Relationship between games and system services (cortex-runtime, cortex-policy, cortex-settings, cortex-observability).

In scope for this spec:
- apps/games/solitaire
- apps/games/minesweeper
- apps/games/snake
- apps/games/tetris
- apps/games/chess

## 3. Out of Scope

- Network multiplayer in v1 (all games are single-player or local two-player).
- Leaderboards or online high-score sync.
- In-app purchases, monetization, or ad surfaces.
- Game controller / gamepad support (keyboard and mouse only in v1).
- Custom game asset pipelines or sprite sheet tools.
- Third-party game distribution (games are first-party only in v1).
- Tournament or competitive ranking systems.
- Replay recording or playback.

## 4. Objectives

1. Define a shared game framework that all CortexOS games consume, guaranteeing consistent rendering, input, state management, and UI chrome.
2. Ensure every game supports keyboard and mouse controls as mandatory input methods.
3. Mandate save/load, pause/resume, and undo capabilities as framework-provided primitives.
4. Establish scoring and high-score persistence patterns that are uniform across all games.
5. Ensure games follow the same permission model and security posture as other first-party apps -- no elevated privileges.
6. Guarantee that all games are fully playable via keyboard alone, mouse alone, or any combination.
7. Provide a help/rules system accessible from within every game without leaving the game window.

## 5. User-Visible Behavior

### 5.1 Consistent Game Chrome

Every game window must present:

- **Title bar**: Managed by the window manager (spec 08). Contains the game name and standard window controls (minimize, maximize, close).
- **Toolbar**: Below the title bar, containing:
  - Game title or icon (left-aligned).
  - Score display (centered or right of title).
  - Timer display (if applicable, next to score).
  - Pause button (play/pause toggle icon).
  - Settings gear icon (opens game-specific settings such as difficulty).
  - Help/rules button (opens a modal overlay with game rules and controls).
- **Game area**: The central canvas or grid where gameplay occurs.
- **Status bar**: Optional bottom bar showing contextual information (e.g., mine count, level, next piece preview).

### 5.2 Pause and Resume

- Clicking the pause button or pressing `Escape` pauses the game.
- When paused:
  - The game loop stops updating.
  - A semi-transparent overlay covers the game area with "PAUSED" text.
  - The overlay shows buttons: Resume, Restart, Settings, Help, Quit.
  - The game timer freezes.
  - The game area is obscured (prevents cheating in timed/memorization games).
- Resuming restores the game loop and timer from the exact paused state.
- Games with no active play state (e.g., between rounds or before first move) do not require pausing but must not crash if pause is invoked.

### 5.3 Save and Load

- Games auto-save state to session storage on every significant state change (move, score change, timer tick -- debounced to once per second for timer).
- Users can manually save via `Ctrl+S` or the settings menu.
- Users can load a saved game from the settings menu.
- On app relaunch, the last auto-saved state is restored automatically, with a brief "Resume where you left off?" confirmation dialog.
- Multiple save slots are not required in v1 (single save state per game).

### 5.4 Help and Rules

- Every game provides a built-in help overlay accessible via:
  - The help/rules toolbar button.
  - Keyboard shortcut `F1` or `Ctrl+H`.
- The help overlay displays:
  - Game objective and rules in concise plain text.
  - Controls reference table (keyboard and mouse).
  - Scoring rules.
  - Tips for new players.
- The help overlay is dismissible via `Escape`, clicking outside, or a close button.

### 5.5 New Game and Restart

- New game can be started via `Ctrl+N` or from the pause/settings menu.
- If a game is in progress, a confirmation dialog appears: "Start a new game? Current progress will be lost."
- Restart resets all game state to initial conditions based on current difficulty settings.

### 5.6 Game Over and Win States

- Game over: A modal overlay displays the outcome (win/lose), final score, and high-score comparison.
- The overlay offers: Play Again, Close.
- If the player achieves a new high score, the overlay highlights this with a distinct visual treatment and a "New High Score!" message.

## 6. System Behavior

### 6.1 Game Loop Architecture

All games run a requestAnimationFrame-based game loop with three phases per frame:

1. **Process Input**: Read queued keyboard and mouse events. Translate to game actions.
2. **Update State**: Apply game rules, advance simulation, update score, check win/loss conditions.
3. **Render**: Draw the current game state to the canvas or DOM.

The loop runs at the display refresh rate (typically 60 fps) and uses delta-time for frame-rate-independent logic where applicable. The framework provides the loop; games plug in their update and render functions.

### 6.2 Input Manager

The framework input manager:

- Captures keyboard events (keydown, keyup) and maintains a pressed-key set.
- Captures mouse events (click, double-click, mousedown, mouseup, mousemove, contextmenu) relative to the game area.
- Normalizes input into game-specific action events (e.g., "move_left", "rotate_cw").
- Supports input rebinding in the settings panel (future consideration; v1 uses fixed bindings defined in child specs).
- Prevents default browser behavior for game-relevant keys (arrow keys, space) when the game area has focus.

### 6.3 State Serialization

The framework provides a state serialization service:

- Each game defines its serializable state as a JSON-compatible TypeScript interface.
- The framework serializes the state on every significant state change (debounced per game's declared interval).
- Serialized state is stored via `cortex-runtime.apps.state` under the game's namespace.
- On load, the framework validates the serialized state against the game's schema version. If the version mismatches, the game starts fresh and logs a warning.

### 6.4 Scoring Service

The framework provides scoring primitives:

- Current score (number, starts at 0 unless restored from save).
- High scores: an array of `{ score: number, date: string, difficulty: string }`, persisted across sessions.
- Maximum high-score entries: 10 per difficulty level.
- Score update notifications: the framework emits an event when the score changes, allowing the UI chrome to animate score changes.

### 6.5 Timer Service

The framework provides a game timer:

- Elapsed time in seconds, displayed as MM:SS.
- The timer starts on first player action (not on game load).
- The timer pauses when the game is paused.
- The timer stops on game over or win.
- Timer state is included in serialized game state for save/load.

### 6.6 Permission Model

Games use the same permission enforcement as all first-party apps:

- cortex-policy evaluates every permission check without special-casing app identity.
- Games declare required permissions in their manifest.
- No implicit or hidden grants. If a game needs storage, it declares the appropriate permission.
- Permission denials are handled gracefully per section 12.

### 6.7 Theme Integration

Games consume design tokens from the CortexOS theme system:

- Board/grid colors, piece colors, text colors, and chrome colors all derive from theme tokens.
- Games must render correctly in light, dark, and high-contrast themes.
- Games may define additional game-specific color tokens that are derived from the base theme (e.g., a "mine" color that adjusts per theme).
- No hardcoded colors. All visual properties consume CSS custom properties from the theme.

## 7. Architecture

### 7.1 Common Game Architecture

```
apps/games/{game-name}/
  manifest.json           # CortexOS game manifest
  package.json            # Frontend package
  src/
    main.ts               # Entry point, registers game with runtime
    Game.tsx               # Root game component
    components/
      GameArea.tsx         # Main game rendering (canvas or DOM grid)
      GameToolbar.tsx      # Score, timer, pause, settings, help buttons
      GameOverlay.tsx      # Pause, game-over, and win overlays
      HelpOverlay.tsx      # Rules and controls reference
      SettingsPanel.tsx    # Game-specific settings (difficulty, etc.)
      ScoreDisplay.tsx     # Animated score display
      TimerDisplay.tsx     # MM:SS timer
    engine/
      GameEngine.ts        # Implements the game loop interface
      GameState.ts         # Game-specific state class/interface
      InputHandler.ts      # Game-specific input mapping
      Renderer.ts          # Game-specific rendering logic
      ScoreEngine.ts       # Score calculation logic
      AIEngine.ts          # Computer opponent logic (if applicable)
    services/
      StateSerializer.ts   # Serialize/deserialize game state
      HighScoreService.ts  # Persist and retrieve high scores
    hooks/
      useGameLoop.ts       # Hook wrapping requestAnimationFrame loop
      useInput.ts          # Hook for keyboard and mouse input
      useGameState.ts      # Hook for game state with undo support
    types.ts               # Game-specific TypeScript types
  tests/
    unit/
      engine.test.ts       # Game engine logic tests
      scoring.test.ts      # Score calculation tests
      input.test.ts        # Input mapping tests
      state.test.ts        # State serialization tests
    integration/
      lifecycle.test.ts    # Game launch, pause, resume, save, load
      gameplay.test.ts     # Full game play-through scenarios
      theme.test.ts        # Theme rendering validation
```

### 7.2 Shared Game Framework Package

A shared framework package `@cortexos/game-framework` provides:

- `GameLoop`: requestAnimationFrame loop manager with pause/resume.
- `InputManager`: Keyboard and mouse event capture and normalization.
- `StateSerializer`: JSON serialization with schema versioning.
- `TimerService`: Game timer with pause/resume and persistence.
- `ScoreService`: Score tracking and high-score persistence.
- `GameChrome`: Pre-built toolbar and overlay React components.
- `HelpOverlay`: Pre-built rules/help modal component.

Each game imports from this framework and provides game-specific implementations for:
- `IGameEngine`: The game's update/render cycle.
- `IGameState`: The game's serializable state.
- `IInputMap`: The game's keyboard and mouse bindings.
- `IRenderer`: The game's draw logic.

### 7.3 Dependency Graph

Games depend on:
- `@cortexos/game-framework` (shared game framework)
- `@cortexos/ui-components` (shared UI library for chrome)
- `@cortexos/runtime-client` (for state persistence)
- `@cortexos/theme` (design token consumer)

Games must NOT depend on:
- Internal implementation details of system crates.
- Direct filesystem, network, or OS APIs.
- Third-party game engines or rendering libraries (Canvas API and DOM only).

## 8. Data Model

### 8.1 Game Manifest Schema

```typescript
interface GameManifest {
  id: string;                       // e.g., "com.cortexos.games.solitaire"
  name: string;                     // Display name
  version: string;                  // Semver
  description: string;
  firstParty: true;
  bundled: true;
  category: "games";

  entry: {
    frontend: string;               // Path to frontend entry
  };

  window: {
    defaultWidth: number;
    defaultHeight: number;
    minWidth: number;
    minHeight: number;
    resizable: boolean;
    singleInstance: true;           // Games are always single-instance
  };

  permissions: {
    required: string[];
    optional: string[];
  };

  game: {
    supportsPause: true;
    supportsUndo: boolean;          // Declared per game
    supportsSave: true;
    supportsTimer: boolean;         // Declared per game
    maxPlayers: 1 | 2;             // 1 = single-player, 2 = local two-player
    difficulties: string[];         // e.g., ["beginner", "intermediate", "expert"]
    defaultDifficulty: string;
  };

  accessibility: {
    highContrastSupport: true;
    screenReaderSupport: true;
    keyboardNavigation: true;
  };
}
```

### 8.2 Common Game State Base

```typescript
interface GameStateBase {
  schemaVersion: number;            // Incremented on breaking state changes
  status: "idle" | "playing" | "paused" | "game_over" | "won";
  score: number;
  timerElapsed: number;             // Seconds elapsed (0 until first action)
  timerStarted: boolean;            // Whether first action has occurred
  difficulty: string;
  moveHistory: MoveRecord[];        // For undo support
  undoPointer: number;              // Index into moveHistory (-1 = no undo)
  startedAt: string;                // ISO 8601 timestamp
  lastSaved: string;                // ISO 8601 timestamp
}

interface MoveRecord {
  moveIndex: number;
  timestamp: number;                // ms since game start
  action: string;                   // Game-specific action identifier
  stateSnapshot: string;            // JSON string of full state before move
}

interface HighScoreEntry {
  score: number;
  difficulty: string;
  date: string;                     // ISO 8601
  duration: number;                 // Seconds
}
```

### 8.3 High Score Storage

```typescript
interface HighScoreStore {
  entries: HighScoreEntry[];
  maxPerDifficulty: number;         // Default: 10
}
```

High scores are stored in persistent state under `cortex-runtime://app-state/{game-id}/high-scores`.

## 9. Public Interfaces

### 9.1 Game Engine Interface

Each game must implement and export:

```typescript
interface IGameEngine {
  readonly id: string;
  readonly name: string;

  createInitialState(difficulty: string): IGameState;
  update(state: IGameState, deltaTime: number, input: GameInput): IGameState;
  render(state: IGameState, canvas: HTMLCanvasElement | HTMLElement): void;
  checkWinCondition(state: IGameState): boolean;
  checkLossCondition(state: IGameState): boolean;
  calculateScore(state: IGameState, action: string): number;
  getValidActions(state: IGameState): string[];
  serializeState(state: IGameState): string;
  deserializeState(json: string): IGameState | null;
}
```

### 9.2 Game Input Interface

```typescript
interface GameInput {
  keyboard: {
    pressedKeys: Set<string>;
    keyPressedThisFrame: string[];
    keyReleasedThisFrame: string[];
  };
  mouse: {
    position: { x: number; y: number };
    clickedThisFrame: boolean;
    rightClickedThisFrame: boolean;
    doubleClickedThisFrame: boolean;
    dragStart: { x: number; y: number } | null;
    dragEnd: { x: number; y: number } | null;
    isDragging: boolean;
  };
}
```

### 9.3 Game Lifecycle Hooks

```typescript
interface CortexGame {
  mount(container: HTMLElement, runtime: CortexRuntimeClient): void;
  unmount(): Promise<void>;
  getState(): GameStateBase;
  setState(state: Partial<GameStateBase>): void;
  pause(): void;
  resume(): void;
  restart(difficulty?: string): void;
  undo(): boolean;                  // Returns false if undo not possible
  showHelp(): void;
}
```

## 10. Internal Interfaces

### 10.1 Game Framework Internal APIs

`@cortexos/game-framework` provides these internal interfaces:

```typescript
// Game Loop Controller
interface GameLoopController {
  start(): void;
  stop(): void;
  pause(): void;
  resume(): void;
  isRunning(): boolean;
  isPaused(): boolean;
  onFrame(callback: (deltaTime: number) => void): void;
}

// Undo Manager
interface UndoManager<T> {
  push(state: T): void;
  undo(): T | null;
  canUndo(): boolean;
  clear(): void;
  getHistorySize(): number;
}

// State Persistence
interface GameStatePersistence {
  save(gameId: string, state: GameStateBase): Promise<void>;
  load(gameId: string): Promise<GameStateBase | null>;
  clear(gameId: string): Promise<void>;
}

// High Score Manager
interface HighScoreManager {
  getHighScores(gameId: string, difficulty: string): HighScoreEntry[];
  addHighScore(gameId: string, entry: HighScoreEntry): void;
  isHighScore(gameId: string, difficulty: string, score: number): boolean;
}
```

### 10.2 Theme Integration for Games

Games consume theme tokens through CSS custom properties. The framework provides a mapping of semantic game tokens:

```typescript
interface GameThemeTokens {
  // Board/Grid
  '--game-board-bg': string;
  '--game-board-border': string;
  '--game-cell-bg': string;
  '--game-cell-border': string;

  // Pieces/Elements
  '--game-piece-primary': string;
  '--game-piece-secondary': string;
  '--game-piece-accent': string;
  '--game-piece-danger': string;

  // Text
  '--game-text-primary': string;
  '--game-text-secondary': string;
  '--game-text-score': string;

  // Overlays
  '--game-overlay-bg': string;
  '--game-overlay-text': string;

  // Highlights
  '--game-highlight-valid': string;
  '--game-highlight-invalid': string;
  '--game-highlight-selected': string;
}
```

These tokens are derived from the base CortexOS theme tokens and are overridden per theme. Games never reference raw color values.

## 11. State Management

### 11.1 State Layers

Each game manages three layers of state:

1. **Ephemeral state**: React component state for transient UI (hover effects, animation progress, drag position). Lost on unmount.
2. **Session state**: The full game state (position, score, timer, move history). Persisted via cortex-runtime session state so it survives hot-reload. Auto-saved on every significant state change, debounced to a maximum of one write per second.
3. **Persistent state**: High scores, preferred difficulty, user preferences (sound on/off, theme overrides). Persists across sessions via cortex-runtime.

### 11.2 State Sync Rules

- Auto-save debounce: maximum one write per second during active play.
- State is saved immediately on pause, win, or game over (not debounced).
- State is saved immediately on app unmount (before unmount promise resolves).
- On app relaunch, if saved state exists with status "playing" or "paused", the user is prompted to resume.
- If saved state exists with status "idle", the game starts fresh without prompting.
- Schema version mismatches result in a fresh start and a logged warning (no crash, no corrupted state).

### 11.3 State Size Limits

- Game state (serialized JSON): maximum 5 MB per game.
- Move history for undo: maximum 1000 entries. If exceeded, oldest entries are pruned.
- High scores: maximum 10 entries per difficulty level.

## 12. Failure Modes and Error Handling

### 12.1 Common Game Failure Modes

| Failure | Detection | Recovery |
|---------|-----------|----------|
| State load failure (corrupt JSON) | Deserialization returns null | Log warning, start fresh game, notify user: "Could not restore previous game. Starting a new game." |
| State schema version mismatch | Version check in deserializer | Log warning, start fresh game. No data migration in v1. |
| Storage quota exceeded | cortex-runtime returns QuotaExceeded | Warn user via toast. Continue playing in-memory. Attempt save on next significant action after pruning move history. |
| Canvas context unavailable | getContext returns null | Fall back to DOM-based rendering. Log error. |
| Input device disconnected | N/A (keyboard/mouse always available in browser) | No action needed. |
| Game engine throws during update | try/catch in game loop | Pause game. Show error overlay: "An error occurred. Your progress has been saved." Offer restart. Log error with stack trace. |
| Timer precision drift | Delta-time accumulation | Framework recalculates elapsed time from Date.now() delta on each frame, preventing cumulative drift. |
| Window resize during play | ResizeObserver callback | Re-render game area at new dimensions. Scale game content proportionally. Pause if minimum dimensions not met. |

### 12.2 Error Presentation Rules

- Game errors are presented as overlays on the game area (not system dialogs) to maintain immersion.
- Errors include a short description and a "Restart" button.
- Critical errors (engine crash) pause the game automatically.
- Non-critical errors (high score save failure) show a toast notification and do not interrupt gameplay.
- Errors are logged via cortex-observability at the appropriate level.

## 13. Security and Permissions

### 13.1 No Hidden Privileges

Games follow the same permission enforcement as all first-party apps:

- cortex-policy evaluates every permission check without special-casing app identity.
- Game manifests declare all required permissions explicitly.
- The system does not auto-grant permissions to games.
- Admin can revoke game permissions identically to any other app.

### 13.2 Permission Declaration

**Required (all games):**
- `runtime.state` -- Read/write own game state.
- `runtime.lifecycle` -- Access lifecycle hooks.

**Optional (declared per game):**
- `notifications.send` -- Send notifications (e.g., for timed games running in background).

### 13.3 Content Security

- Games must not execute dynamically loaded code.
- Game state JSON is treated as untrusted input during deserialization.
- No network requests in v1 games.
- No access to browser APIs beyond Canvas, DOM events, and cortex-runtime.

## 14. Performance Requirements

### 14.1 Startup

- Game window must render first meaningful paint (game board/grid visible) within 500 ms of launch signal.
- Game must be fully interactive (accepting input) within 1 second of launch signal.
- State restoration must not block initial render.

### 14.2 Runtime

- Game loop must maintain 60 fps during normal gameplay.
- Frame time must not exceed 20 ms for any single frame (allowing 3 ms budget over 16.67 ms target).
- Score animation and overlay transitions must not cause frame drops.
- Full-board re-renders (e.g., minesweeper flood reveal) must complete within one frame.

### 14.3 Memory

- Games must not exceed 100 MB heap memory under normal usage.
- Games must not leak memory over extended play sessions (verified over 10,000 moves in tests).
- Undo history must be pruned to prevent unbounded memory growth.

### 14.4 Bundle Size

- Individual game frontend bundle must not exceed 300 KB gzipped.
- Shared game framework is loaded once by the runtime and not counted against per-game bundle size.

## 15. Accessibility Requirements

All games must meet the following (WCAG 2.1 AA minimum):

- **Keyboard navigation**: All gameplay must be achievable via keyboard alone. Games that primarily use mouse (e.g., solitaire drag-and-drop) must provide keyboard alternatives for every action.
- **Screen reader**: Game status (score, timer, turn indicator) announced via ARIA live regions. Game board state described via ARIA labels on grid cells or regions.
- **High contrast**: Correct rendering in high-contrast theme. Game pieces, board elements, and interactive regions must be distinguishable by more than color alone (use patterns, shapes, labels, or borders).
- **Text sizing**: Score, timer, and help text must remain readable at 200% browser text zoom.
- **Reduced motion**: Respect `prefers-reduced-motion`. Disable animations (piece movement, win celebration) when this setting is active. Static state replaces animated transitions.
- **Focus management**: Focus must be trapped within the game area during active play. Tab key must not escape the game area (use `Escape` to pause, then `Tab` navigates the pause overlay).

## 16. Observability and Logging

### 16.1 Structured Logging

All games log through `cortex-observability.log` with structured entries:

```typescript
{
  app: string;          // Game ID
  level: "debug" | "info" | "warn" | "error";
  event: string;        // Event name (e.g., "game.started", "game.move", "game.paused")
  payload: object;      // Event-specific data (no PII)
  timestamp: string;    // ISO 8601
  sessionId: string;    // Game session ID
}
```

### 16.2 Required Log Events

Each game must log:

- `game.launched` (info) -- Game opened with difficulty.
- `game.started` (info) -- New game started.
- `game.paused` / `game.resumed` (info) -- Pause/resume events.
- `game.move` (debug) -- Each player move (action type only, no full state).
- `game.undone` (info) -- Undo action performed.
- `game.won` (info) -- Game won with score and duration.
- `game.lost` (info) -- Game lost with score and duration.
- `game.saved` / `game.loaded` (info) -- Save/load events.
- `game.error` (warn) -- Game engine error (error type and recovery action).
- `game.high_score` (info) -- New high score achieved.

### 16.3 Telemetry

- Games do not send telemetry independently. All telemetry goes through cortex-observability.
- Games report performance metrics (frame time, startup time) via cortex-observability metrics API.

## 17. Testing Requirements

### 17.1 Unit Tests

- Minimum 85% code coverage per game.
- Game engine logic (rules, scoring, win/loss detection) must have 95% coverage.
- State serialization round-trip tests: serialize, deserialize, verify equality.
- Input mapping tests: verify keyboard and mouse inputs map to correct game actions.
- Edge cases: game start, game end, empty state, maximum undo history, high score pruning.

### 17.2 Integration Tests

- Full game lifecycle: launch, play several moves, pause, resume, save, close, relaunch, load, continue playing, win/lose.
- Undo flow: make moves, undo several, verify correct state restoration, continue playing.
- Theme switch: game renders correctly in all three themes without reload.
- Window resize: game scales and remains playable.
- Keyboard-only play-through: complete a full game using only keyboard input.
- Mouse-only play-through: complete a full game using only mouse input.

### 17.3 Accessibility Tests

- AX tree validation for game area, toolbar, overlays, and help screen.
- Keyboard-only navigation test for all game flows.
- Screen reader announcement test for score, timer, and game status changes.
- High-contrast rendering test.

### 17.4 Performance Tests

- Frame time measurement during intensive gameplay (must not exceed 20 ms per frame).
- Memory usage over extended play session (10,000 moves).
- Startup time verified under 500 ms / 1 s thresholds.

## 18. Acceptance Criteria

A game is accepted when:

- [ ] Manifest validates against the GameManifest schema.
- [ ] All required permissions are declared and handled.
- [ ] Game launches and renders within performance thresholds.
- [ ] Theme switching works across light, dark, and high-contrast without custom color logic.
- [ ] All interactive elements are keyboard-navigable.
- [ ] Game is fully playable using keyboard alone.
- [ ] Game is fully playable using mouse alone.
- [ ] Pause/resume works correctly (timer freezes, state preserved, game area obscured).
- [ ] Save/load works correctly across app restart.
- [ ] Undo works correctly (if supported by the game).
- [ ] Help/rules overlay is accessible and complete.
- [ ] High scores persist across sessions.
- [ ] Score, timer, and status are announced to screen readers.
- [ ] Game renders correctly at 200% text zoom.
- [ ] Reduced motion preference is respected (animations disabled).
- [ ] Error handling follows section 12 conventions.
- [ ] Logging follows section 16 conventions.
- [ ] Unit test coverage >= 85%.
- [ ] Integration tests for lifecycle, theme, and input pass.
- [ ] Accessibility tests pass.
- [ ] Performance tests pass (60 fps, <100 MB memory).
- [ ] No direct filesystem, network, or OS API access.
- [ ] No hidden privileges or permission bypasses.

## 19. Build Order and Dependencies
**Layer 12**. Depends on: 09 (app runtime), 16 (theme tokens)

### 19.1 Prerequisite Specs (must be implemented first)

1. Spec 02 -- Core Architecture and Runtime Boundaries
2. Spec 04 -- Permissions, Policy, Trust Model
3. Spec 08 -- Window Manager
4. Spec 09 -- App Runtime and App Lifecycle
5. Spec 10 -- System Command Bus and Event Model
6. Spec 14 -- Observability, Logging, Telemetry
7. Spec 15 -- Accessibility, Input, Keyboard System
8. Spec 16 -- Theme, Design Tokens, UI System
9. Spec 17 -- First-Party Core Apps Parent (shared patterns and conventions)

### 19.2 Build Order Within Games

Games should be implemented in this order (simpler to more complex):

1. **18c -- Snake**: Simplest game loop, grid-based, validates framework rendering and input.
2. **18b -- Minesweeper**: Grid-based with flood-fill algorithm, validates state serialization and difficulty system.
3. **18a -- Solitaire**: Drag-and-drop input, validates mouse interaction and complex undo.
4. **18d -- Tetris**: Timed game loop with increasing speed, validates timer service and real-time rendering.
5. **18e -- Chess**: Most complex rules engine, validates AI opponent and move validation.

### 19.3 In-Repo Dependencies

```
packages/game-framework/       # Shared game framework (must be built first)
  depends on:
    packages/ui-components/
    packages/runtime-client/
    packages/theme/

apps/games/{game-name}/
  depends on:
    packages/game-framework/
    packages/ui-components/
    packages/runtime-client/
    packages/theme/
```

## 20. Non-Goals and Anti-Patterns

### 20.1 Non-Goals

- Games are not expected to compete with dedicated gaming platforms or apps.
- Network multiplayer is explicitly out of scope for v1.
- Game controller / gamepad support is deferred to v2.
- Leaderboards, achievements, and social features are deferred.
- Custom skin or theme creation for games is not supported in v1.
- Game replay recording and sharing is not supported.

### 20.2 Anti-Patterns

- **Hidden privileges**: Never grant games elevated permissions beyond what is declared in their manifest.
- **Hardcoded colors**: Never hardcode visual properties. Always consume theme tokens.
- **Frame-rate-dependent logic**: Never tie game speed to frame rate. Always use delta-time.
- **Blocking main thread**: Never perform heavy computation on the main thread that causes frame drops. Offload to Web Workers if needed.
- **eval or innerHTML**: Never eval user input or inject untrusted HTML.
- **Direct DOM manipulation in game loop**: Never manipulate DOM directly during the game loop for Canvas-based games. Use the Canvas API exclusively for rendering.
- **Mutable shared state**: Never share mutable state between game engine and UI components. Use immutable state updates or state snapshots.
- **Unbounded history**: Never accumulate undo history or move history without pruning.
- **Ignoring reduced motion**: Never force animations when the user has enabled reduced motion.
- **Single input method**: Never implement a game that requires only keyboard or only mouse. Both must be supported.
- **Custom file save dialogs**: Never build custom save dialogs. Use the game framework's save/load mechanism.

## 21. Implementation Instructions for Claude Code / Codex

### 21.1 Subsystem Ownership

- The game framework (`@cortexos/game-framework`) owns the game loop, input manager, state serializer, timer, score service, and UI chrome components.
- Each game owns its specific rules engine, renderer, input mapping, and state definition.
- This parent spec owns the shared contract, not any game-specific logic.
- Changes to the shared framework require updating this file and verifying all child specs remain compatible.

### 21.2 Recommended Implementation Order

1. Build `@cortexos/game-framework` package with `GameLoop`, `InputManager`, `StateSerializer`, `TimerService`, `ScoreService`.
2. Write framework unit tests (loop control, input capture, serialization round-trip, timer accuracy, score persistence).
3. Build the shared `GameChrome` component (toolbar, pause overlay, game-over overlay, help overlay, settings panel).
4. Implement the first game (Snake) to validate the framework.
5. Iterate on the framework based on Snake integration learnings.
6. Implement remaining games in build order (Minesweeper, Solitaire, Tetris, Chess).

### 21.3 What Can Be Stubbed Initially

- AI opponent logic (Chess) can use a random-move generator initially, upgraded to a real engine later.
- Win animations can be a simple overlay initially, enhanced to particle effects later.
- Sound effects can be omitted entirely in v1 (no audio dependencies).
- Input rebinding UI can be a no-op (fixed bindings in v1).
- Notifications (background timer alerts) can be a no-op until the notification service is integrated.

### 21.4 What Must Be Real in v1

- Full game loop with pause/resume.
- Complete game rules engines (no shortcut or partial implementations).
- Keyboard and mouse input for every action.
- Save/load state across sessions.
- Undo (for games that declare `supportsUndo: true`).
- High score persistence.
- Help/rules overlay with complete rules.
- Theme support (all three themes).
- Accessibility (keyboard navigation, screen reader, high contrast, reduced motion).
- Error handling and logging.

### 21.5 What Cannot Be Inferred

- Exact game board dimensions and cell sizes (defined per game in child specs).
- Specific piece shapes, colors, and visual styles (defined per game, derived from theme tokens).
- Difficulty parameter values (defined per game in child specs).
- Scoring formulas (defined per game in child specs).
- Specific keyboard and mouse bindings (defined per game in child specs).

### 21.6 Stop Conditions

A game subsystem is done when:

1. All acceptance criteria in section 18 pass.
2. Child spec acceptance criteria pass.
3. `manifest.json` validates against GameManifest schema.
4. No linter warnings, no type errors.
5. All tests pass (unit, integration, accessibility, performance).
6. No direct imports of system crate internals.
7. Code review confirms no anti-patterns from section 20.
8. Game is fully playable via keyboard alone and via mouse alone.
9. Save/load round-trip succeeds without data loss.
10. High scores persist across sessions.

### 21.7 Testing Gates

- Pre-merge: unit tests and lint pass. Framework tests pass.
- Post-merge to main: integration tests and accessibility tests pass for all games.
- Pre-release: performance tests pass (60 fps, memory limits).
- Manual QA: full play-through of each game, keyboard-only and mouse-only, theme switch, pause/resume, save/load.
