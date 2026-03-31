# 18c. Snake

## 1. Purpose

The Snake game provides a classic arcade experience for CortexOS, featuring a snake that grows as it eats food on a 20x20 grid. The player steers the snake with arrow keys, avoiding walls and its own body. It serves as the reference implementation for a real-time, tick-based game within the game framework, validating game loop timing, progressive difficulty via speed increases, and persistent high score storage.

## 2. Scope

- Classic snake on a fixed 20x20 grid.
- Arrow key controls for direction (up, down, left, right).
- Snake grows by one segment each time it eats food.
- Death on collision with walls or the snake's own body.
- Speed increases every 5 food items eaten (tick interval decreases).
- Score equals the number of food items eaten.
- High score persisted across sessions.
- Pause/resume with Space bar.
- App location: `apps/games/snake`.

## 3. Out of Scope

- Diagonal movement.
- Wrap-around walls (snake does not pass through edges to the opposite side).
- Multiple food items on the grid simultaneously.
- Power-ups, obstacles, or bonus items.
- AI-controlled snake or multiplayer modes.
- Custom grid sizes.
- Sound effects.
- Snake skin themes or custom appearance.
- Obstacle placement or maze layouts.

## 4. Objectives

1. Implement a fully playable snake game with correct collision detection, growth mechanics, and progressive speed.
2. Validate the game framework's real-time tick-based loop with configurable tick intervals.
3. Demonstrate persistent high score storage using the framework's state persistence layer.
4. Provide responsive keyboard controls with input buffering to prevent missed direction changes between ticks.
5. Implement clean grid rendering at varying tick speeds without frame-rate dependency.

## 5. User-Visible Behavior

### 5.1 Game Layout

The game area consists of:

1. **Toolbar**: Score display (left), high score display (center-left), current speed level (center-right), pause button (right).
2. **Grid**: A 20x20 grid centered in the game area. Each cell is approximately 24x24 px at default window size, scaling proportionally with window resize. The grid has a visible border indicating the wall boundary.
3. **Overlay slots**: Pause overlay, game-over overlay, and help overlay render on top of the grid.

### 5.2 Cell Representation

- **Empty cell**: Background color derived from `--game-cell-bg` token. Alternating subtle shade on even/odd cells to create a checkerboard pattern for visual clarity.
- **Snake head**: Filled cell using `--game-piece-primary` token with a slightly lighter border. Two small "eye" indicators facing the current direction (rendered as two small dots using `--game-cell-bg` token).
- **Snake body**: Filled cell using `--game-piece-primary` token, slightly darker than the head, with a subtle inner border giving a segmented appearance.
- **Food**: Filled cell using `--game-piece-danger` token (red dot) with a subtle pulse animation (scale 1.0 to 1.15 over 600 ms, repeating). Respects reduced-motion preference (static dot if reduced motion is enabled).

### 5.3 Keyboard Controls

| Key | Action |
|-----|--------|
| `Arrow Up` or `W` | Change direction to up (if not currently moving down). |
| `Arrow Down` or `S` | Change direction to down (if not currently moving up). |
| `Arrow Left` or `A` | Change direction to left (if not currently moving right). |
| `Arrow Right` or `D` | Change direction to right (if not currently moving left). |
| `Space` | Pause or resume the game. |
| `Enter` | Start a new game (from game-over screen or idle state). |
| `Escape` | Pause game (if playing). |
| `Ctrl+N` | New game (resets score, grid, and snake). |
| `H` or `?` | Toggle help overlay. |

### 5.4 Mouse Controls

Mouse input is secondary for Snake (keyboard is primary). The following mouse interactions are provided:

| Action | Behavior |
|--------|----------|
| Click on the grid | Focus the game area (enables keyboard capture). |
| Click "New Game" button | Start a new game. |
| Click pause button | Pause or resume. |
| Click help button | Toggle help overlay. |

### 5.5 Direction Restriction

The snake cannot reverse direction in a single tick. Specifically:
- If moving up, the player cannot set direction to down.
- If moving down, the player cannot set direction to up.
- If moving left, the player cannot set direction to right.
- If moving right, the player cannot set direction to left.

This prevents the snake from immediately colliding with its own neck. An attempt to reverse direction is silently ignored.

### 5.6 Input Buffering

The game buffers one direction change per tick. If the player presses multiple direction keys between ticks, only the last valid direction change is applied. This ensures responsive control at high speeds without allowing illegal moves.

### 5.7 Win and Loss

**Loss**: The game ends when:
- The snake's head collides with a wall cell (the grid boundary).
- The snake's head collides with any segment of its own body (including the tail segment that has not yet moved).

There is no explicit win condition. The game continues until loss.

**Game Over screen**: Displays "Game Over", final score, high score, and a "Play Again" button. The snake's final position and the food remain visible on the grid.

## 6. System Behavior

### 6.1 Game Loop

Snake uses a tick-based game loop (not frame-based). The snake advances one cell per tick. The tick interval starts at 200 ms and decreases as the snake eats food.

**Tick cycle (per tick):**
1. Consume buffered direction input.
2. Calculate the new head position: current head + one cell in the current direction.
3. Check collision:
   a. If new head is outside grid bounds -> death (wall collision).
   b. If new head overlaps any body segment -> death (self collision).
4. If alive:
   a. Add new head cell to the front of the snake.
   b. Check if new head is on food:
      - If yes: increment score, generate new food, do NOT remove tail (snake grows by 1).
      - If no: remove the last tail cell (snake moves forward without growing).
5. Update state and re-render.

### 6.2 Speed Progression

The tick interval decreases every 5 food items eaten:

| Food Eaten | Speed Level | Tick Interval (ms) |
|------------|-------------|---------------------|
| 0-4        | 1           | 200                 |
| 5-9        | 2           | 175                 |
| 10-14      | 3           | 150                 |
| 15-19      | 4           | 130                 |
| 20-24      | 5           | 115                 |
| 25-29      | 6           | 100                 |
| 30-34      | 7           | 90                  |
| 35-39      | 8           | 82                  |
| 40-44      | 9           | 75                  |
| 45+        | 10          | 70                  |

Speed level is calculated as: `Math.min(10, Math.floor(score / 5) + 1)`.

The tick interval is calculated as: `Math.max(70, 200 - (speedLevel - 1) * 15)` with clamping at 70 ms minimum.

### 6.3 Food Generation

Food is placed at a random empty cell on the grid. "Empty" means the cell is not occupied by any segment of the snake.

**Algorithm:**
1. Collect all cells not occupied by the snake into a candidate list.
2. If the candidate list is empty (snake fills the entire grid -- theoretical win), the game continues without food (this scenario is virtually impossible on a 20x20 grid with a single snake but is handled).
3. Select a random cell from the candidate list using a seeded PRNG.
4. Place food at the selected cell.

Food is generated immediately after the previous food is eaten, before the next tick.

### 6.4 Collision Detection

Collision checks occur during the tick cycle, after the new head position is calculated:

- **Wall collision**: `newHeadRow < 0 || newHeadRow >= 20 || newHeadCol < 0 || newHeadCol >= 20`.
- **Self collision**: `snake.segments` contains a segment with the same `(row, col)` as the new head. The check is performed against all current body segments BEFORE the tail is removed (this prevents the edge case where the tail moves away from the collision point in the same tick).

### 6.5 Timer Behavior

- A game timer starts when the first tick occurs (game starts).
- Timer pauses when the game is paused.
- Timer stops on game over.
- Display format: MM:SS in the toolbar.
- Timer is for display only; it does not affect scoring or speed.

### 6.6 Pause Behavior

- When paused, the tick loop stops. No ticks are processed.
- The current tick interval timer is canceled.
- A "PAUSED" overlay appears over the grid.
- When resumed, the tick loop restarts at the current tick interval.
- Direction buffer is cleared on resume to prevent accidental immediate direction changes.

## 7. Architecture

```
apps/games/snake/
  manifest.json
  package.json
  src/
    main.ts                    # Registers game with runtime
    SnakeGame.tsx              # Root component
    components/
      GameBoard.tsx            # Grid layout and cell rendering
      SnakeCell.tsx            # Single cell rendering (all states)
      GameToolbar.tsx          # Score, high score, speed, pause
      GameOverOverlay.tsx      # Game over screen
      PauseOverlay.tsx         # Pause screen
      HelpOverlay.tsx          # Rules and controls
    engine/
      SnakeEngine.ts           # Game rules, tick, collision, growth
      SnakeState.ts            # Full game state definition
      FoodGenerator.ts         # Random food placement on empty cells
      SpeedManager.ts          # Tick interval calculation
      CollisionDetector.ts     # Wall and self collision checks
      DirectionBuffer.ts       # Input buffering for direction changes
    services/
      StateSerializer.ts       # Serialize/deserialize snake state
      HighScoreService.ts      # Persist and retrieve high scores
    hooks/
      useTickLoop.ts           # Tick-based game loop with variable interval
      useKeyboardInput.ts      # Arrow key and WASD input capture
    types.ts
  tests/
    unit/
      engine.test.ts           # Tick cycle, collision, growth, direction
      food_generator.test.ts   # Food placement on empty cells, no-snake overlap
      speed_manager.test.ts    # Speed level and tick interval calculation
      collision.test.ts        # Wall collision, self collision, edge cases
      direction_buffer.test.ts # Input buffering, direction restriction
      state.test.ts            # State serialization round-trip
    integration/
      gameplay.test.ts         # Full game play-through: eat food, speed up, die
      keyboard.test.ts         # Keyboard-only play-through
      pause_resume.test.ts     # Pause and resume preserves state
      high_score.test.ts       # High score persistence across sessions
      theme.test.ts            # Theme rendering
```

## 8. Data Model

### 8.1 Grid Position

```typescript
interface Position {
  row: number;                  // 0-19 (top to bottom)
  col: number;                  // 0-19 (left to right)
}
```

### 8.2 Snake State

```typescript
interface SnakeState {
  schemaVersion: 1;
  status: "idle" | "playing" | "paused" | "game_over";

  // Snake
  snake: {
    segments: Position[];        // Head is index 0, tail is last index
    direction: Direction;
    nextDirection: Direction;    // Buffered direction for next tick
  };

  // Food
  food: Position;

  // Score
  score: number;                // Number of food items eaten
  highScore: number;

  // Speed
  speedLevel: number;           // 1-10
  tickIntervalMs: number;       // Current tick interval in milliseconds

  // Grid
  gridRows: 20;
  gridCols: 20;

  // Timer
  timerElapsed: number;
  timerStarted: boolean;

  // Meta
  startedAt: string;
  lastSaved: string;

  // Undo (single-step: restart from last tick before death)
  lastTickSnapshot: string | null;  // JSON of state before last tick
}

type Direction = "up" | "down" | "left" | "right";

const OPPOSITE_DIRECTION: Record<Direction, Direction> = {
  up: "down",
  down: "up",
  left: "right",
  right: "left",
};

const DIRECTION_DELTA: Record<Direction, Position> = {
  up:    { row: -1, col: 0 },
  down:  { row: 1,  col: 0 },
  left:  { row: 0,  col: -1 },
  right: { row: 0,  col: 1 },
};
```

### 8.3 Move Record

```typescript
interface SnakeMove {
  tickIndex: number;
  timestamp: number;
  direction: Direction;
  newHead: Position;
  ateFood: boolean;
  collision: "none" | "wall" | "self" | null;
  snakeLength: number;
  stateSnapshot: string;            // JSON of full state before tick
}
```

### 8.4 Manifest

```typescript
{
  id: "com.cortexos.games.snake",
  name: "Snake",
  version: "1.0.0",
  description: "Classic snake arcade game",
  firstParty: true,
  bundled: true,
  category: "games",
  entry: { frontend: "src/main.ts" },
  window: {
    defaultWidth: 560,
    defaultHeight: 620,
    minWidth: 400,
    minHeight: 480,
    resizable: true,
    singleInstance: true
  },
  permissions: {
    required: ["runtime.state", "runtime.lifecycle"],
    optional: []
  },
  game: {
    supportsPause: true,
    supportsUndo: false,
    supportsSave: true,
    supportsTimer: true,
    maxPlayers: 1,
    difficulties: [],
    defaultDifficulty: null
  },
  accessibility: {
    highContrastSupport: true,
    screenReaderSupport: true,
    keyboardNavigation: true
  }
}
```

## 9. Public Interfaces

### 9.1 Game Engine Interface

```typescript
interface ISnakeEngine extends IGameEngine {
  // Initialization
  createInitialState(): SnakeState;

  // Tick
  tick(state: SnakeState): SnakeState;

  // Direction
  setDirection(state: SnakeState, direction: Direction): SnakeState;

  // Collision
  checkWallCollision(head: Position): boolean;
  checkSelfCollision(head: Position, segments: Position[]): boolean;

  // Food
  generateFood(segments: Position[], gridRows: number, gridCols: number): Position;

  // Speed
  calculateSpeedLevel(score: number): number;
  calculateTickInterval(speedLevel: number): number;
}
```

### 9.2 Cell Display Interface

```typescript
interface SnakeCellDisplayProps {
  cellType: "empty" | "snake_head" | "snake_body" | "food";
  direction: Direction | null;       // For head eye rendering
  isCheckerDark: boolean;            // Alternating cell shade
}
```

## 10. Internal Interfaces

### 10.1 Direction Buffer

```typescript
interface DirectionBuffer {
  buffer(direction: Direction, currentDirection: Direction): void;
  consume(): Direction | null;
  clear(): void;
}

interface DirectionBufferState {
  pendingDirection: Direction | null;
}
```

### 10.2 Tick Loop

```typescript
interface TickLoop {
  start(intervalMs: number, callback: () => void): void;
  stop(): void;
  setInterval(intervalMs: number): void;
  isRunning(): boolean;
}
```

## 11. State Management

### 11.1 State Layers

1. **Ephemeral**: Current animation frame for food pulse, hover effects on toolbar buttons.
2. **Session**: Full `SnakeState` object. Auto-saved every 5 ticks (debounced). Saved immediately on pause and game over.
3. **Persistent**: High score (single highest score), stored via the framework's state persistence layer.

### 11.2 State Serialization

The full `SnakeState` is serialized to JSON for save/load. Snake segments, food position, direction, score, speed level, and tick interval are all included.

On deserialization:
- Validate `schemaVersion`. If it does not match `1`, start a new game and log a warning.
- Validate that all snake segments are within grid bounds. If any segment is out of bounds, start a new game and log an error.
- Validate that food position is within grid bounds and not on a snake segment. If invalid, regenerate food and log a warning.
- Restore timer state.

### 11.3 High Score Persistence

A single high score is stored, representing the best score across all sessions:

```typescript
interface SnakeHighScore extends HighScoreEntry {
  score: number;
  date: string;
}
```

The high score is updated when the game ends and the final score exceeds the stored high score. Displayed in the toolbar during gameplay and on the game-over screen.

## 12. Failure Modes and Error Handling

| Failure | Detection | Recovery |
|---------|-----------|----------|
| State load failure (corrupt JSON) | Deserialization returns null or schema version mismatch | Start new game. Show toast: "Could not restore previous game." Log warning. |
| Snake segments out of bounds on load | Validation finds segment with row/col outside 0-19 | Start new game. Log error with out-of-bounds segment. |
| Food on snake segment on load | Validation finds food position overlaps a snake segment | Regenerate food at a valid empty cell. Log warning. |
| Empty candidate list for food | Snake fills entire 20x20 grid (400 segments) | Game continues without placing new food. Display "Perfect Game!" message. This is a theoretical edge case. |
| Tick loop interval calculation underflow | Speed level exceeds expected range | Clamp tick interval to minimum 70 ms. Log warning. |
| Window too small for grid | ResizeObserver reports below minWidth/minHeight | Show message: "Please resize the window to play." Do not render grid. |

## 13. Security and Permissions

- **Required**: `runtime.state`, `runtime.lifecycle` -- for game state persistence and lifecycle management.
- **Optional**: none.
- No filesystem access needed.
- No network access needed.
- Game state JSON is treated as untrusted input during deserialization (validated before use).
- No `eval()` or dynamic code execution.
- No access to user data or other app states.

## 14. Performance Requirements

- Tick processing (collision check, movement, food generation): under 1 ms per tick.
- Grid re-render after each tick: under 8 ms (single frame at 120 fps).
- Food generation (scanning 400 cells for empty candidates): under 1 ms.
- Full grid re-render (after load): under 16 ms (single frame at 60 fps).
- Memory usage must not exceed 20 MB (400 cells + snake state + history).
- Startup first meaningful paint: under 400 ms.
- No frame drops during food pulse animation.

## 15. Accessibility Requirements

- **Keyboard navigation**: Full game playable via keyboard. Arrow keys or WASD for direction. Space for pause. Enter for new game. All controls documented in help overlay.
- **Screen reader**: Score changes announced via ARIA live region. Game-over announcement: "Game Over. Score: X. High score: Y." Pause/resume announced. Speed level changes announced ("Speed level 2").
- **High contrast**: Snake and food are distinguishable by shape and pattern, not color alone. Snake head has eye indicators. Food has a distinct circular shape with a border. Empty cells use a subtle checkerboard pattern.
- **Focus indicators**: Game grid has a visible focus ring when focused (required for keyboard input capture).
- **Reduced motion**: Food pulse animation is disabled (static dot). No other continuous animations in the game.
- **Text sizing**: Score, high score, speed level, and timer text remain readable at 200% browser zoom.

## 16. Observability and Logging

### 16.1 Required Log Events

- `snake.launched` (info) -- Game opened.
- `snake.new_game` (info) -- New game started.
- `snake.direction_change` (debug) -- Direction changed. Includes direction and tick index.
- `snake.food_eaten` (debug) -- Food eaten. Includes score and speed level.
- `snake.speed_up` (info) -- Speed level increased. Includes new level and tick interval.
- `snake.pause` (info) -- Game paused. Includes score and tick index.
- `snake.resume` (info) -- Game resumed.
- `snake.game_over` (info) -- Game ended. Includes score, high score, snake length, collision type, and time played.
- `snake.high_score` (info) -- New high score achieved. Includes score.
- `snake.gave_up` (info) -- New game started before game over in current game.
- `snake.error` (warn) -- Engine error (error type and recovery action).

No PII is logged.

## 17. Testing Requirements

### 17.1 Unit Tests

- **Tick cycle**: Snake advances one cell per tick in the current direction. Tail is removed when no food is eaten. Tail is not removed when food is eaten (snake grows).
- **Direction restriction**: Cannot reverse direction. Illegal direction changes are silently ignored.
- **Direction buffer**: Only one direction change is applied per tick. Last valid direction wins when multiple keys are pressed between ticks.
- **Wall collision**: Snake dies when head moves outside grid bounds (all four edges tested).
- **Self collision**: Snake dies when head moves onto a body segment. Edge case: snake of length 3+ with a U-turn body layout.
- **Food generation**: Food is always placed on an empty cell. Food never overlaps snake segments. Food is generated within grid bounds.
- **Speed calculation**: Speed level and tick interval are correct for all score values 0-50+. Clamped at maximum speed level 10.
- **State serialization**: Round-trip test -- serialize, deserialize, verify snake segments, food position, score, and direction match.

### 17.2 Integration Tests

- **Full game play-through**: Start game, eat food, verify growth, verify speed-up at score 5, 10, 15, etc., die on wall collision.
- **Self-collision death**: Steer snake into its own body, verify game over.
- **Pause/resume**: Pause mid-game, resume, verify state is preserved and tick loop restarts.
- **Direction buffer timing**: Press direction key, verify change is applied on next tick, not immediately.
- **High score persistence**: Achieve a score, close game, reopen, verify high score is displayed.
- **Save/load**: Play 20 ticks, save, close, reopen, load. Verify exact state (snake position, food, score, speed).
- **Theme switch**: Verify snake color, food color, grid colors in all three themes.

### 17.3 Accessibility Tests

- AX tree validation: grid cells have correct ARIA labels (empty, snake, food).
- Keyboard navigation: game is fully playable with keyboard alone.
- Screen reader: score and game-over announcements are correct.

## 18. Acceptance Criteria

- [ ] 20x20 grid renders correctly with checkerboard pattern.
- [ ] Snake starts at center of grid, length 3, moving right.
- [ ] Arrow keys and WASD control direction correctly.
- [ ] Direction reversal is blocked (cannot go opposite of current direction).
- [ ] Snake grows by one segment when eating food.
- [ ] Snake dies on wall collision (all four edges tested).
- [ ] Snake dies on self collision.
- [ ] Speed increases every 5 food items eaten (10 speed levels).
- [ ] Tick interval decreases from 200 ms to 70 ms across speed levels.
- [ ] Score equals number of food items eaten.
- [ ] High score persists across sessions.
- [ ] Pause (Space) stops the tick loop and shows overlay.
- [ ] Resume restarts the tick loop at current speed.
- [ ] Input buffering prevents missed direction changes between ticks.
- [ ] Game-over screen shows score, high score, and "Play Again" button.
- [ ] Save/load restores exact game state across app restarts.
- [ ] Help overlay shows complete rules and controls.
- [ ] All three themes render correctly with no hardcoded colors.
- [ ] Game is fully playable via keyboard alone.
- [ ] Screen reader announces score changes and game over.
- [ ] Food pulse animation respects reduced-motion preference.
- [ ] Unit test coverage >= 85%.
- [ ] Engine test coverage >= 95%.

## 19. Build Order and Dependencies
**Layer 12**. Depends on: 09 (app runtime), 16 (theme tokens), 18 (games parent)

### 19.1 Prerequisites

- Spec 18 -- Games Platform Parent (shared game framework).
- `@cortexos/game-framework` (game loop, input, state, chrome).
- `@cortexos/ui-components` (shared UI library).
- `@cortexos/runtime-client` (state persistence).
- `@cortexos/theme` (design token consumer).

### 19.2 Build Position

Snake is the **first** game to build. It validates the game framework's real-time tick-based loop, variable-speed timing, keyboard input handling, and persistent high score storage.

No Rust crate needed. Pure frontend app.

## 20. Non-Goals and Anti-Patterns

### 20.1 Non-Goals

- Wrap-around walls (edges are solid boundaries).
- Multiple food items or power-ups.
- AI snake or multiplayer modes.
- Custom grid sizes or difficulty settings.
- Replay recording or playback.
- Statistics tracking beyond high score.

### 20.2 Anti-Patterns

- **Frame-rate-dependent movement**: Tick interval is real-time based (ms), not frame-count based. Snake speed must be consistent regardless of display refresh rate.
- **Recursive or complex algorithms for food placement**: Use simple candidate-list randomization. The grid is only 400 cells.
- **Mutating snake segments in place**: State changes produce new segment arrays. No in-place mutation.
- **setInterval for tick loop**: Use `setTimeout` with recalculated interval on each tick, or the framework's provided tick loop primitive. This allows dynamic interval changes without drift.
- **Hardcoded snake/food colors**: Use theme tokens for all visual elements.
- **Storing full tick history without pruning**: Only store the last tick snapshot for potential undo. Do not accumulate tick history.

## 21. Implementation Instructions for Claude Code / Codex

### 21.1 Subsystem Ownership

- Snake owns: tick-based game loop, collision detection, direction buffering, food generation, speed management, snake rendering, grid rendering.
- Snake does not own: state persistence (framework), high score storage (framework), theme system (framework), window management (framework), timer display (framework provides timer primitive, Snake displays it).

### 21.2 Recommended Implementation Order

1. Create `manifest.json` and validate against GameManifest schema.
2. Implement `SnakeState.ts` -- define state, initial state factory (snake at center, length 3, moving right). Write unit tests.
3. Implement `DirectionBuffer.ts` -- direction buffering with reversal restriction. Write unit tests.
4. Implement `CollisionDetector.ts` -- wall and self collision checks. Write unit tests with edge cases.
5. Implement `FoodGenerator.ts` -- random empty cell selection with seed support. Write unit tests.
6. Implement `SpeedManager.ts` -- speed level and tick interval calculation. Write unit tests.
7. Implement `SnakeEngine.ts` -- integrate all subsystems into the tick cycle. Write unit tests extensively.
8. Build `SnakeCell.tsx` component with all visual states (empty, snake head with eyes, snake body, food).
9. Build `GameBoard.tsx` grid layout with checkerboard pattern.
10. Build `GameToolbar.tsx` with score, high score, speed level, and pause button.
11. Implement `useTickLoop.ts` hook for tick-based game loop with variable interval.
12. Implement `useKeyboardInput.ts` hook for arrow/WASD input capture.
13. Integrate with game framework for state persistence, high scores, and timer.
14. Build pause overlay, game-over overlay, and help overlay.
15. Implement `StateSerializer.ts` and test round-trip.
16. Accessibility audit and fixes.
17. Theme verification (light, dark, high-contrast).

### 21.3 What Can Be Stubbed Initially

- Food pulse animation can be a static dot initially.
- Snake eye indicators can be omitted initially (solid colored head).
- Checkerboard pattern can be a solid background initially.
- Speed display in toolbar can be a simple number initially.

### 21.4 What Must Be Real in v1

- Complete tick-based game loop with variable tick interval.
- Direction input with buffering and reversal restriction.
- Snake growth on food consumption.
- Wall and self collision detection.
- 10 speed levels with correct tick intervals.
- Food generation on random empty cells.
- Score tracking and persistent high score.
- Pause/resume with state preservation.
- Save/load with full state restoration.
- Game-over screen with score and high score.
- Help overlay with complete rules.
- Theme support (all three themes).
- Accessibility (keyboard, screen reader, high contrast).

### 21.5 What Cannot Be Inferred

- Grid cell size: 24x24 px at default window size, scaled proportionally.
- Snake initial position: center of grid, 3 segments, moving right. Head at (10, 10), body at (10, 9) and (10, 8).
- Grid border: 2 px solid line using `--game-border` token.
- Food pulse animation: scale 1.0 to 1.15 over 600 ms, CSS transform, prefers-reduced-motion respected.
- Snake body segment gap: 1 px visual gap between segments for visual clarity (achieved with a 1 px inner border in the background color).
- Initial tick interval: 200 ms (speed level 1).
- Minimum tick interval: 70 ms (speed level 10).

### 21.6 Stop Conditions

1. All acceptance criteria in section 18 pass.
2. Engine unit test coverage >= 95%.
3. Overall unit test coverage >= 85%.
4. Integration tests for full gameplay, pause/resume, and save/load pass.
5. Accessibility tests pass.
6. No hardcoded colors (all theme tokens).
7. No linter warnings or type errors.

### 21.7 Testing Gates

- Engine, collision, and direction buffer unit tests must pass before UI work begins.
- Speed progression integration test (verify tick interval at each speed level) must pass before merge.
- Keyboard-only play-through must succeed before merge.
- Save/load round-trip must pass before merge.
- High score persistence must be verified across close/reopen cycles.
