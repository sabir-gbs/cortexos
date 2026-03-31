# 18b. Minesweeper

## 1. Purpose

The Minesweeper game provides a classic mine-clearing puzzle experience for CortexOS, featuring a grid of hidden cells where the player must reveal all non-mine cells without detonating any mines. It serves as the reference implementation for a grid-based puzzle game within the game framework, validating flood-fill algorithms, first-click-safe generation, and multi-difficulty state management.

## 2. Scope

- Classic minesweeper on a rectangular grid with three difficulty presets.
- Left-click to reveal a cell. Right-click to place or remove a flag.
- First-click safety: the first reveal never hits a mine; mines are placed after the first click.
- Chord (double-click or middle-click on a numbered cell): auto-reveal all adjacent unflagged cells when the correct number of flags surround it.
- Three difficulty levels:
  - Beginner: 9x9 grid, 10 mines.
  - Intermediate: 16x16 grid, 40 mines.
  - Expert: 30x16 grid, 99 mines.
- Mine counter (total mines minus flags placed, displayed as a negative-capable integer).
- Game timer (starts on first click, stops on win or loss).
- Win condition: all non-mine cells are revealed.
- Loss condition: a mine cell is revealed.
- Question-mark mode (optional): cells can be cycled through hidden, flagged, question-mark, hidden.
- App location: `apps/games/minesweeper`.

## 3. Out of Scope

- Custom grid sizes or mine counts (user cannot define arbitrary dimensions).
- Hexagonal or non-rectangular grid variants.
- Multiplayer minesweeper.
- Mine detection hints or solver.
- Statistics tracking beyond high scores (e.g., win percentage, streaks, 3BV/speed metrics).
- Sound effects.
- Animations on cell reveal beyond the immediate visual update.
- Themeable cell skins or custom mine/flag icons.

## 4. Objectives

1. Implement a fully playable minesweeper with correct flood-fill reveal, mine counting, and first-click-safe generation.
2. Validate the game framework's state serialization with a grid-based game that has large state (expert mode: 480 cells).
3. Demonstrate the difficulty system with three distinct presets affecting grid dimensions and mine count.
4. Provide smooth keyboard navigation of a grid, proving the framework's keyboard input for grid-based games.
5. Implement chord mechanics correctly, including mis-flag chord detonation (if wrong flags are placed, chord can reveal a mine).

## 5. User-Visible Behavior

### 5.1 Game Layout

The game area consists of:

1. **Toolbar**: Mine counter (left), smiley-face status indicator (center), timer (right). The smiley changes expression based on game state: happy (playing), surprised (mid-click), cool sunglasses (won), dead (lost).
2. **Grid**: A rectangular grid of cells. Each cell is a square approximately 28x28 px at default window size, scaling proportionally with window resize. Grid is centered in the game area.

### 5.2 Cell Representation

- **Hidden cell**: Flat, raised appearance suggesting an unrevealed button. Uses `--game-cell-bg` token.
- **Revealed cell (number)**: Sunken appearance. Displays a digit 1-8 in a color unique per number (1=blue, 2=green, 3=red, 4=dark-blue, 5=dark-red, 6=teal, 7=black, 8=gray). Colors derived from theme tokens where possible; numbers use `--game-text-primary` as fallback in high-contrast mode.
- **Revealed cell (empty)**: Sunken, blank.
- **Flagged cell**: Hidden cell appearance with a flag symbol overlaid. Flag uses `--game-piece-danger` token.
- **Question-mark cell**: Hidden cell appearance with a "?" symbol overlaid.
- **Mine (game over)**: Red background with mine symbol. Uses `--game-piece-danger` token for background.
- **Wrong flag (game over)**: Flagged cell shown with an X through it to indicate it was not a mine.

### 5.3 Mouse Controls

| Action | Behavior |
|--------|----------|
| Left-click hidden cell | Reveal the cell. If mine, game over. If empty, flood-fill reveal all connected empty and numbered cells. |
| Left-click revealed cell | No action. |
| Right-click hidden cell | Cycle: hidden -> flagged -> question-mark -> hidden (if question-mark mode enabled) or hidden -> flagged -> hidden (if question-mark mode disabled). |
| Right-click flagged cell | Remove flag (cycle to question-mark or hidden depending on mode). |
| Middle-click or left+right-click on revealed number | Chord: if the number of adjacent flags equals the cell's number, reveal all adjacent unflagged hidden cells. If any of those cells is an unflagged mine, game over. |
| Left-click smiley | Restart game with current difficulty. |
| Mouse down (any) on grid | Smiley shows "surprised" face until mouse up. |

### 5.4 Keyboard Controls

| Key | Action |
|-----|--------|
| `Arrow keys` | Move cursor highlight one cell in the pressed direction. |
| `Enter` or `Space` | Reveal the highlighted cell. |
| `F` | Toggle flag on the highlighted cell. |
| `Q` | Toggle question-mark on the highlighted cell. |
| `C` or `Middle-click equivalent` | Chord on the highlighted revealed cell. |
| `Ctrl+N` | New game (current difficulty). |
| `1` | Set difficulty to Beginner. |
| `2` | Set difficulty to Intermediate. |
| `3` | Set difficulty to Expert. |
| `Escape` | Pause game. |

### 5.5 Game Status Indicator

The smiley-face button in the toolbar reflects game state:
- **Happy**: Game in progress or idle.
- **Surprised**: Mouse button is currently held down on a cell (mid-click anticipation).
- **Sunglasses**: Game won (all non-mine cells revealed).
- **Dead (X-eyes)**: Game lost (mine detonated).

Clicking the smiley at any time starts a new game at the current difficulty.

### 5.6 Win and Loss

**Win**: When every non-mine cell has been revealed, the game is won. Remaining mine cells are auto-flagged. Timer stops. Win overlay displays: "You Win!", time, and difficulty. "Play Again" button appears.

**Loss**: When a mine cell is revealed, the game is lost. All remaining mines are revealed. Incorrectly flagged cells show an X. Timer stops. Loss overlay displays: "Game Over", and "Play Again" button appears.

## 6. System Behavior

### 6.1 Mine Generation

Mines are placed after the player's first left-click reveal, not at game initialization. This guarantees the first clicked cell (and optionally its neighbors) are never mines.

**First-click-safe algorithm:**

1. Player clicks cell (row, col).
2. Generate a set of protected cells: the clicked cell and its 8 neighbors (or fewer at edges/corners). Total protected: up to 9 cells.
3. Randomly place N mines on the grid, excluding all protected cells. If the grid has fewer than N + 9 cells (e.g., beginner 9x9=81, mines=10, 81-10=71 available, 9 protected, 71-9=62 candidates), this always succeeds. For expert, 480 - 99 = 381 available, 9 protected, 372 candidates; always succeeds.
4. After mines are placed, calculate the number for each non-mine cell (count of adjacent mines, 0-8).
5. Reveal the clicked cell. If it is a 0 (no adjacent mines), flood-fill from that cell.

### 6.2 Flood-Fill Reveal

When a cell with zero adjacent mines is revealed, the game automatically reveals all connected cells that also have zero adjacent mines, plus the border of numbered cells surrounding them.

**Algorithm (iterative BFS to avoid stack overflow on large grids):**

1. Start with the revealed cell (row, col).
2. If the cell's number is 0, push it onto a queue.
3. While the queue is not empty:
   a. Dequeue a cell.
   b. For each of its 8 neighbors that is hidden and not flagged:
      - Reveal the neighbor.
      - If the neighbor's number is 0, enqueue it.
4. All cells enqueued and their revealed numbered neighbors are revealed.

Flood-fill must complete in a single frame (under 16 ms) even on expert mode (480 cells).

### 6.3 Chord Logic

When the player chords on a revealed numbered cell:
1. Count adjacent flagged cells.
2. If flagged count equals the cell's number:
   - Reveal all adjacent hidden, unflagged cells.
   - If any revealed cell is a mine, game over.
   - If any revealed cell has number 0, trigger flood-fill from that cell.
3. If flagged count does not equal the cell's number, do nothing.

### 6.4 Mine Counter

The mine counter displays: `total_mines - flag_count`. This can go negative if the player places more flags than there are mines. Display range: -99 to `total_mines`. Three-digit display with leading zeros for positive, leading minus for negative.

### 6.5 Timer Behavior

- Timer starts on the first cell reveal (the same click that triggers mine generation).
- Timer pauses when the game is paused.
- Timer stops on win or loss.
- Maximum display: 999 seconds (rolls over or stops at 999).
- Display format: three-digit integer, no MM:SS conversion (classic minesweeper style).

## 7. Architecture

```
apps/games/minesweeper/
  manifest.json
  package.json
  src/
    main.ts                    # Registers game with runtime
    MinesweeperGame.tsx        # Root component
    components/
      GameBoard.tsx            # Grid layout and cell rendering
      Cell.tsx                 # Single cell rendering (all states)
      GameToolbar.tsx          # Mine counter, smiley, timer
      DifficultySelector.tsx   # Beginner/Intermediate/Expert buttons or menu
      WinOverlay.tsx           # Win screen
      LossOverlay.tsx          # Loss screen (reveals all mines)
      GameOverReveal.tsx       # Mine/wrong-flag rendering on loss
    engine/
      MinesweeperEngine.ts     # Game rules, reveal, chord, win/loss check
      MinesweeperState.ts      # Full game state definition
      MineGenerator.ts         # First-click-safe mine placement
      FloodFill.ts             # BFS flood-fill algorithm
      CellCalculator.ts        # Calculate adjacent mine counts
      ChordEngine.ts           # Chord reveal logic
    services/
      StateSerializer.ts       # Serialize/deserialize minesweeper state
      HighScoreService.ts      # Persist and retrieve high scores (by time)
    hooks/
      useGridNavigation.ts     # Keyboard cursor navigation on grid
      useGameLoop.ts           # Game loop (mostly idle; minesweeper is turn-based)
    types.ts
  tests/
    unit/
      engine.test.ts           # Reveal, chord, win/loss detection
      mine_generator.test.ts   # First-click safety, mine count, placement
      flood_fill.test.ts       # Flood-fill correctness, edge cases
      cell_calculator.test.ts  # Adjacent mine count
      state.test.ts            # State serialization round-trip
      chord.test.ts            # Chord scenarios: correct, incorrect flags
    integration/
      gameplay.test.ts         # Full game play-through: beginner win, loss
      first_click.test.ts      # Verify first-click safety across difficulties
      keyboard.test.ts         # Keyboard-only play-through
      theme.test.ts            # Theme rendering
```

## 8. Data Model

### 8.1 Cell

```typescript
interface Cell {
  row: number;
  col: number;
  mine: boolean;             // Whether this cell contains a mine
  revealed: boolean;         // Whether the cell has been revealed
  flagged: boolean;          // Whether the player placed a flag
  questionMark: boolean;     // Whether the player placed a question mark
  adjacentMines: number;     // 0-8, calculated after mine placement
}
```

### 8.2 Minesweeper State

```typescript
interface MinesweeperState {
  schemaVersion: 1;
  status: "idle" | "playing" | "paused" | "won" | "lost";

  // Grid
  difficulty: "beginner" | "intermediate" | "expert";
  rows: number;                            // 9, 16, 16
  cols: number;                            // 9, 16, 30
  totalMines: number;                      // 10, 40, 99
  grid: Cell[][];                          // rows x cols
  minesPlaced: boolean;                    // false until first click

  // Counters
  flagCount: number;
  revealedCount: number;
  totalSafeCells: number;                  // rows * cols - totalMines

  // Timer
  timerElapsed: number;
  timerStarted: boolean;

  // First click tracking
  firstClickMade: boolean;
  protectedCells: Set<string>;             // "row,col" of protected cells

  // Meta
  startedAt: string;
  lastSaved: string;

  // Undo
  moveHistory: MinesweeperMove[];
  undoPointer: number;
}

type DifficultyConfig = {
  rows: number;
  cols: number;
  mines: number;
};

const DIFFICULTY_CONFIGS: Record<string, DifficultyConfig> = {
  beginner:     { rows: 9,  cols: 9,  mines: 10 },
  intermediate: { rows: 16, cols: 16, mines: 40 },
  expert:       { rows: 16, cols: 30, mines: 99 },
};
```

### 8.3 Move Record

```typescript
interface MinesweeperMove {
  moveIndex: number;
  timestamp: number;
  action: "reveal" | "flag" | "unflag" | "question" | "chord" | "flood_reveal";
  row: number;
  col: number;
  cellsRevealed: Array<{ row: number; col: number; adjacentMines: number }>;
  stateSnapshot: string;                  // JSON of full state before move
}
```

### 8.4 Manifest

```typescript
{
  id: "com.cortexos.games.minesweeper",
  name: "Minesweeper",
  version: "1.0.0",
  description: "Classic mine-clearing puzzle game",
  firstParty: true,
  bundled: true,
  category: "games",
  entry: { frontend: "src/main.ts" },
  window: {
    defaultWidth: 600,
    defaultHeight: 500,
    minWidth: 320,
    minHeight: 400,
    resizable: true,
    singleInstance: true
  },
  permissions: {
    required: ["runtime.state", "runtime.lifecycle"],
    optional: []
  },
  game: {
    supportsPause: true,
    supportsUndo: true,
    supportsSave: true,
    supportsTimer: true,
    maxPlayers: 1,
    difficulties: ["beginner", "intermediate", "expert"],
    defaultDifficulty: "beginner"
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
interface IMinesweeperEngine extends IGameEngine {
  // Grid setup
  createGrid(rows: number, cols: number): Cell[][];
  placeMines(grid: Cell[][], totalMines: number, protectedCells: Set<string>): Cell[][];
  calculateAdjacent(grid: Cell[][], rows: number, cols: number): Cell[][];

  // Reveal
  revealCell(state: MinesweeperState, row: number, col: number): MinesweeperState;
  floodFill(state: MinesweeperState, startRow: number, startCol: number): MinesweeperState;

  // Chord
  chordCell(state: MinesweeperState, row: number, col: number): MinesweeperState;

  // Flags
  toggleFlag(state: MinesweeperState, row: number, col: number): MinesweeperState;
  toggleQuestionMark(state: MinesweeperState, row: number, col: number): MinesweeperState;

  // Win/loss
  checkWin(state: MinesweeperState): boolean;
  checkLoss(state: MinesweeperState, row: number, col: number): boolean;
}
```

### 9.2 Cell Display Interface

```typescript
interface CellDisplayProps {
  cell: Cell;
  gameOver: boolean;
  highlight: boolean;        // Keyboard cursor highlight
  onClick: (row: number, col: number) => void;
  onRightClick: (row: number, col: number) => void;
  onChord: (row: number, col: number) => void;
  row: number;
  col: number;
}
```

## 10. Internal Interfaces

### 10.1 Grid Navigator

```typescript
interface GridNavigator {
  moveUp(): void;
  moveDown(): void;
  moveLeft(): void;
  moveRight(): void;
  getCurrentCell(): { row: number; col: number };
  setCell(row: number, col: number): void;
  clampToBounds(): void;
}

interface GridNavState {
  cursorRow: number;
  cursorCol: number;
  maxRows: number;
  maxCols: number;
}
```

### 10.2 Mine Generator

```typescript
interface MineGenerator {
  generate(
    rows: number,
    cols: number,
    totalMines: number,
    protectedCells: Set<string>
  ): Set<string>;              // Returns set of "row,col" mine positions
}
```

## 11. State Management

### 11.1 State Layers

1. **Ephemeral**: Cursor position for keyboard navigation, hover effects, smiley face state (surprised on mousedown).
2. **Session**: Full `MinesweeperState` object. Auto-saved on every reveal and flag action (debounced to 1 per second maximum). Saved immediately on pause, win, or loss.
3. **Persistent**: High scores (top 10 per difficulty, sorted by time ascending -- fastest win first), preferred difficulty, question-mark mode enabled/disabled.

### 11.2 State Serialization

The full `MinesweeperState` is serialized to JSON for save/load. The grid is serialized as a flat 2D array of cell objects. Mine positions, revealed state, flags, and question marks are all included.

On deserialization:
- Validate `schemaVersion`. If it does not match `1`, start a new game and log a warning.
- Validate grid dimensions match the stored difficulty. If mismatch, start a new game and log a warning.
- Validate that the number of mines matches `totalMines`. If mismatch, start a new game and log an error.
- Restore timer state.

### 11.3 High Score Persistence

High scores are stored per difficulty, sorted by completion time (ascending):

```typescript
interface MinesweeperHighScore extends HighScoreEntry {
  difficulty: "beginner" | "intermediate" | "expert";
  completionTime: number;     // Seconds
}
```

Maximum 10 entries per difficulty. A new high score is offered when the player wins and their time is faster than the 10th entry (or fewer than 10 entries exist).

## 12. Failure Modes and Error Handling

| Failure | Detection | Recovery |
|---------|-----------|----------|
| State load failure (corrupt JSON) | Deserialization returns null or schema version mismatch | Start new game. Show toast: "Could not restore previous game." Log warning. |
| Grid dimension mismatch on load | Validation finds rows/cols do not match difficulty config | Start new game. Log error with expected vs actual dimensions. |
| Mine count mismatch on load | Validation finds mine count inconsistent with grid | Start new game. Log error with mine count. |
| Flood-fill exceeds frame budget | Performance monitor detects > 16 ms frame time | No user-visible impact; log warning with grid size. Flood-fill is guaranteed O(cells) and cannot exceed 480 cells (expert), so this is a theoretical concern only. |
| Window too small for grid | ResizeObserver reports below minWidth/minHeight | Show message: "Please resize the window to play." Do not render grid. |
| Chord on unrevealed cell | Engine detects target cell is not revealed | Ignore action silently. |

## 13. Security and Permissions

- **Required**: `runtime.state`, `runtime.lifecycle` -- for game state persistence and lifecycle management.
- **Optional**: none.
- No filesystem access needed.
- No network access needed.
- Game state JSON is treated as untrusted input during deserialization (validated before use).
- No `eval()` or dynamic code execution.
- Mine positions in serialized state are protected by the pause-overlay obscuring mechanism (no cheating via developer tools is specifically countered, but the game does not expose mine positions through public interfaces).

## 14. Performance Requirements

- Cell reveal (single cell): immediate, under 1 ms.
- Flood-fill on beginner (81 cells): under 2 ms.
- Flood-fill on expert (480 cells): under 8 ms (guaranteed single-frame).
- Full grid re-render (after undo or load): under 16 ms (single frame).
- Memory usage must not exceed 30 MB (grid + state + undo history).
- Startup first meaningful paint: under 400 ms.

## 15. Accessibility Requirements

- **Keyboard navigation**: Full game playable via keyboard. Arrow keys move cursor. Enter/Space reveals. F flags. C chords. Cursor is a visible, high-contrast outline (3px solid `--game-highlight-selected`).
- **Screen reader**: Current cursor position announced (e.g., "Row 3, Column 5, hidden"). Revealed cells announce their number (e.g., "Row 3, Column 5, 2"). Mine counter and timer changes announced via ARIA live region. Win/loss announced.
- **High contrast**: Cell numbers are distinguishable by shape and position, not color alone. In high-contrast mode, numbers use a single high-contrast color and are bold. Flag and mine symbols use distinct shapes (triangle flag, circle mine).
- **Focus indicators**: Keyboard cursor cell has a visible, high-contrast outline.
- **Reduced motion**: No animations to disable in minesweeper (game has no continuous animations). Win/loss overlay is static.
- **Text sizing**: Cell numbers and symbols remain readable at 200% browser zoom. Cell minimum size may increase to accommodate larger text.

## 16. Observability and Logging

### 16.1 Required Log Events

- `minesweeper.launched` (info) -- Game opened with difficulty.
- `minesweeper.new_game` (info) -- New game started. Includes difficulty.
- `minesweeper.first_click` (debug) -- First cell clicked. Mine generation triggered.
- `minesweeper.reveal` (debug) -- Cell revealed. Includes row, col, and number of cells revealed (1 for single, N for flood-fill).
- `minesweeper.flag` (debug) -- Flag placed or removed.
- `minesweeper.chord` (debug) -- Chord performed. Includes row, col, and result (safe or detonation).
- `minesweeper.undo` (info) -- Undo performed. Includes move index.
- `minesweeper.won` (info) -- Game won with time, difficulty, and completion time.
- `minesweeper.lost` (info) -- Game lost with time and cells revealed.
- `minesweeper.gave_up` (info) -- New game started before winning current game.
- `minesweeper.error` (warn) -- Engine error (error type and recovery action).

No PII is logged.

## 17. Testing Requirements

### 17.1 Unit Tests

- **Mine generation**: Correct mine count. Protected cells never contain mines. Mines are placed randomly (probabilistic test with fixed seed for reproducibility).
- **Adjacent mine calculation**: Every cell's count is correct for given mine placements. Edge and corner cells tested.
- **Flood-fill**: Single cell reveal. Full flood-fill from a corner. Flood-fill blocked by flags. Flood-fill on expert-sized grid completes correctly.
- **Reveal mechanics**: Revealing a mine triggers loss. Revealing a number shows correct count. Revealing a flagged cell does nothing.
- **Chord**: Chord with correct flags reveals safely. Chord with incorrect flags can trigger loss. Chord on cell with wrong flag count does nothing.
- **Win detection**: Win when all non-mine cells revealed. No win when any non-mine cell is still hidden.
- **Flag counting**: Mine counter updates correctly. Can go negative.
- **State serialization**: Round-trip test -- serialize, deserialize, verify every cell matches.

### 17.2 Integration Tests

- **Full beginner game win**: Play a predetermined winning game (fixed seed) from start to win.
- **Full game loss**: Reveal a mine and verify game-over state, mine reveal, wrong-flag display.
- **First-click safety**: Start 100 games with fixed seeds, verify first click never hits a mine.
- **Keyboard-only play**: Complete a beginner game using only keyboard.
- **Difficulty switch**: Switch from beginner to expert mid-session, verify grid changes.
- **Undo flow**: Reveal cells, undo, verify grid returns to prior state.
- **Save/load**: Play 10 moves, save, close, reopen, load. Verify exact state.
- **Theme switch**: Verify cell colors, flag colors, and overlays in all three themes.

### 17.3 Accessibility Tests

- AX tree validation: every cell has correct ARIA labels (state, number if revealed).
- Keyboard navigation: arrow keys move cursor through all cells.
- Screen reader: mine counter and timer announcements on change.

## 18. Acceptance Criteria

- [ ] Three difficulty levels render correctly sized grids (9x9, 16x16, 30x16).
- [ ] Mine count per difficulty is correct (10, 40, 99).
- [ ] First click never hits a mine (tested across 100 seeded games).
- [ ] Flood-fill correctly reveals all connected empty cells and their numbered borders.
- [ ] Left-click reveals a single cell or triggers flood-fill.
- [ ] Right-click cycles through flag states correctly.
- [ ] Chord works correctly (auto-reveals when flag count matches number).
- [ ] Chord with wrong flags can cause game loss.
- [ ] Mine counter displays correctly (can go negative).
- [ ] Timer starts on first reveal, stops on win/loss.
- [ ] Win detected when all non-mine cells revealed.
- [ ] Loss detected when a mine is revealed.
- [ ] Game over reveals all mines and marks wrong flags.
- [ ] Smiley face reflects game state (happy, surprised, sunglasses, dead).
- [ ] Keyboard controls allow full gameplay without mouse.
- [ ] Undo reverses reveal and flag actions correctly.
- [ ] Save/load restores exact game state across app restarts.
- [ ] High scores persist across sessions per difficulty (fastest time).
- [ ] Help overlay shows complete rules and controls.
- [ ] All three themes render correctly with no hardcoded colors.
- [ ] Game is fully playable via keyboard alone.
- [ ] Screen reader announces game state changes.
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

Minesweeper is the **second** game to build (after Snake). It validates grid-based state serialization, flood-fill algorithms, and the difficulty system with the game framework.

No Rust crate needed. Pure frontend app.

## 20. Non-Goals and Anti-Patterns

### 20.1 Non-Goals

- Custom grid sizes or mine counts.
- Mine solver or hint system.
- Hexagonal grid variants.
- Win/loss statistics beyond high scores.
- Multiplayer minesweeper.
- Animated cell reveals or cascading effects.

### 20.2 Anti-Patterns

- **Recursive flood-fill**: Use iterative BFS to avoid stack overflow on expert-sized grids.
- **Mutating cell objects in place**: Cells are immutable. State changes produce new cell objects or new grid arrays.
- **Generating mines before first click**: Mine positions must not exist until the first click is processed.
- **Hardcoded number colors**: Use theme tokens for cell number colors where possible.
- **Storing full grid in undo history without pruning**: Cap at 1000 entries. Prune oldest when exceeded.
- **Rendering all cells on every state change**: Use React reconciliation to render only changed cells.

## 21. Implementation Instructions for Claude Code / Codex

### 21.1 Subsystem Ownership

- Minesweeper owns: mine generation, flood-fill, chord logic, cell rendering, grid navigation, win/loss detection.
- Minesweeper does not own: game loop (framework), timer (framework), state persistence (framework), high score storage (framework), theme system, window management.

### 21.2 Recommended Implementation Order

1. Create `manifest.json` and validate against GameManifest schema.
2. Implement `MinesweeperState.ts` -- define state, difficulty configs, and initial grid creation (all cells hidden, no mines). Write unit tests.
3. Implement `MineGenerator.ts` -- first-click-safe mine placement with seed support. Write unit tests extensively.
4. Implement `CellCalculator.ts` -- adjacent mine count for every non-mine cell. Write unit tests.
5. Implement `FloodFill.ts` -- iterative BFS flood-fill. Write unit tests with edge cases (corners, borders, flags blocking fill).
6. Implement `MinesweeperEngine.ts` -- integrate mine generation, reveal, chord, win/loss detection. Write unit tests.
7. Implement `ChordEngine.ts` -- chord reveal with correct and incorrect flag scenarios. Write unit tests.
8. Build `Cell.tsx` component with all visual states (hidden, revealed number, revealed empty, flagged, question-mark, mine, wrong flag).
9. Build `GameBoard.tsx` grid layout.
10. Build `GameToolbar.tsx` with mine counter, smiley face, and timer.
11. Implement `useGridNavigation.ts` hook for keyboard cursor navigation.
12. Integrate with game framework for loop, timer, save/load.
13. Build pause overlay, win overlay, loss overlay, and help overlay.
14. Implement `StateSerializer.ts` and test round-trip.
15. Accessibility audit and fixes.
16. Theme verification (light, dark, high-contrast).

### 21.3 What Can Be Stubbed Initially

- Smiley face can be a text character initially. Replace with SVG icons later.
- Question-mark mode can be disabled by default initially.
- Win/loss overlays can be simple text initially.

### 21.4 What Must Be Real in v1

- Complete mine generation with first-click safety.
- Flood-fill with iterative BFS.
- Chord mechanics (correct and incorrect flags).
- All three difficulty levels.
- Mine counter (including negative display).
- Timer (starts on first click, stops on win/loss).
- Full keyboard navigation.
- Undo for reveal and flag actions.
- Save/load with full state restoration.
- Win and loss detection with correct game-over display.
- Help overlay with complete rules.
- Theme support (all three themes).
- Accessibility (keyboard, screen reader, high contrast).

### 21.5 What Cannot Be Inferred

- Cell size: 28x28 px at default window size, scaled proportionally.
- Number colors for cells 1-8: these use classic minesweeper colors adapted to theme tokens. In high-contrast mode, all numbers use `--game-text-primary`.
- Smiley face button size: 36x36 px.
- Mine counter and timer display: three-digit, seven-segment-style font or monospace.
- Grid padding: 8 px around the grid.
- Cell gap: 1 px between cells (using border or gap).

### 21.6 Stop Conditions

1. All acceptance criteria in section 18 pass.
2. Engine unit test coverage >= 95%.
3. Overall unit test coverage >= 85%.
4. Integration tests for full game win, first-click safety, keyboard play, and save/load pass.
5. Accessibility tests pass.
6. No hardcoded colors (all theme tokens).
7. No linter warnings or type errors.

### 21.7 Testing Gates

- Engine, mine generation, and flood-fill unit tests must pass before UI work begins.
- First-click safety test (100 seeded games) must pass before merge.
- Flood-fill on expert-sized grid must complete within 8 ms verified by performance test.
- Keyboard-only play-through must succeed before merge.
- Save/load round-trip must pass before merge.
