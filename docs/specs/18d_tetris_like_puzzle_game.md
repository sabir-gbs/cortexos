# 18d — Tetris (Falling Block Puzzle Game)

## 1. Purpose
Define the Tetris-like falling block puzzle game bundled with CortexOS as a first-party game.

## 2. Scope
- Game board (10x20 grid)
- 7 tetromino shapes with rotation
- Controls (move, rotate, soft/hard drop, hold, pause)
- Line clearing, scoring, leveling
- Next piece preview, hold piece, ghost piece
- Game state persistence

## 3. Out of Scope
- Multiplayer (v1 — single player only)
- AI opponent
- Custom piece sets
- Sound effects (v1)

## 4. Objectives
1. Faithful recreation of classic falling block puzzle mechanics.
2. Smooth 60fps rendering.
3. Game state saveable and resumable.

## 5. User-Visible Behavior

| Action | Outcome |
|---|---|
| User starts game | Pieces begin falling |
| User presses Left/Right | Piece moves one column |
| User presses Up | Piece rotates clockwise |
| User presses Z | Piece rotates counter-clockwise |
| User presses Down | Piece soft drops (faster fall) |
| User presses Space | Piece hard drops (instant) |
| User presses Shift | Current piece swapped with hold |
| User presses P | Game pauses |
| Line completed | Line clears with animation |
| Every 10 lines | Level increases, speed increases |
| Piece can't spawn | Game over |

## 6. System Behavior

### 6.1 Board
- Grid: 10 columns × 20 rows
- Each cell: empty or filled with a color
- Board border: solid walls (left, right, bottom), open top

### 6.2 Tetrominoes
| Shape | Color | Cells (relative to center) |
|---|---|---|
| I | Cyan (#06B6D4) | (0,0), (1,0), (2,0), (3,0) |
| O | Yellow (#EAB308) | (0,0), (1,0), (0,1), (1,1) |
| T | Purple (#8B5CF6) | (0,0), (1,0), (2,0), (1,1) |
| S | Green (#22C55E) | (1,0), (2,0), (0,1), (1,1) |
| Z | Red (#EF4444) | (0,0), (1,0), (1,1), (2,1) |
| L | Orange (#F97316) | (0,0), (1,0), (2,0), (2,1) |
| J | Blue (#3B82F6) | (0,0), (0,1), (1,1), (2,1) |

### 6.3 Rotation System
- Super Rotation System (SRS): standard wall kicks
- 4 rotation states per piece (0°, 90°, 180°, 270°)
- Wall kick: if rotation collides, try offset positions (up to 5 tests)
- O piece does not rotate

### 6.3.1 Spawn Rules
- Pieces spawn using a 4×4 bounding box aligned to the top-center of the visible board.
- Spawn column is centered per modern guideline behavior: the bounding box's left edge starts at column 3 on the 10-column board.
- Spawn orientation is rotation state `0` for every tetromino.
- The I piece spawns horizontally; J, L, S, T, and Z spawn in their state-0 orientation; O spawns unrotated.
- New pieces may occupy rows above the visible playfield during spawn and initial rotation checks.
- A spawn fails, and the game ends immediately, if any occupied cell of the new active piece overlaps an already-filled board cell after applying the canonical spawn position.

### 6.4 Drop Speed
```
Level 1:  1000ms per row
Level 2:  900ms
Level 3:  800ms
...
Level 10: 100ms (minimum)
Formula: max(100, 1100 - (level * 100)) ms
```

### 6.5 Scoring
| Lines Cleared | Points (× current level) |
|---|---|
| 1 (Single) | 100 |
| 2 (Double) | 300 |
| 3 (Triple) | 500 |
| 4 (Tetris) | 800 |

Soft drop: 1 point per row dropped. Hard drop: 2 points per row dropped.

### 6.6 Hold Piece
- Press Shift to swap current piece with hold slot
- Only one swap per piece drop (can't swap again until next piece)
- Hold slot starts empty
- If hold is empty, current piece goes to hold, next piece spawns

### 6.6.1 Lock Delay and Top-Out
- When a falling piece first contacts the stack or floor, a lock delay of 500ms begins.
- Any successful left/right movement or rotation that keeps the piece grounded resets the lock delay, capped at 15 resets for the current piece.
- Hard drop bypasses lock delay and locks immediately.
- Soft drop does not bypass lock delay; it only increases descent speed.
- Top-out occurs only when the next piece cannot spawn at its canonical spawn position. A piece partially above the visible board is allowed if its spawn cells are not obstructed.

### 6.6.2 Input Repeat
- Discrete taps move exactly one column or perform one rotation.
- Held horizontal movement uses delayed auto shift (DAS) of 150ms, then auto-repeat rate (ARR) of 50ms per column.
- Soft drop repeats every frame while held.
- Hard drop, hold, pause, and rotate actions are edge-triggered and must not auto-repeat while the key remains held.

### 6.7 Ghost Piece
- Semi-transparent projection showing where piece would land
- Updates in real-time as piece moves/rotates
- Same shape and color as active piece, 30% opacity

### 6.8 Bag Randomizer
- Pieces dealt in bags of all 7 tetrominoes (shuffled)
- Ensures all 7 pieces appear before any repeats
- Prevents drought of specific pieces

## 7. Architecture
```
┌────────────────────────────┐
│     Tetris Game (TS)       │
│  ┌──────────────────────┐  │
│  │  Game Loop (60fps)   │  │
│  │  (update, render)    │  │
│  └──────────┬───────────┘  │
│  ┌──────────┴───────────┐  │
│  │  Board State         │  │
│  │  (grid, pieces)      │  │
│  └──────────┬───────────┘  │
│  ┌──────────┴───────────┐  │
│  │  Input Handler       │  │
│  │  (keyboard events)   │  │
│  └──────────┬───────────┘  │
│  ┌──────────┴───────────┐  │
│  │  Renderer (Canvas)   │  │
│  └──────────────────────┘  │
└────────────────────────────┘
```

## 8. Data Model
```typescript
type Cell = null | TetrominoColor;
type Board = Cell[][];  // 20 rows × 10 cols

type TetrominoType = 'I' | 'O' | 'T' | 'S' | 'Z' | 'L' | 'J';

interface ActivePiece {
  type: TetrominoType;
  position: { x: number; y: number };  // Top-left of bounding box
  rotation: 0 | 1 | 2 | 3;
}

interface GameState {
  board: Board;
  active_piece: ActivePiece | null;
  next_pieces: TetrominoType[];  // Queue (min 3 visible)
  hold_piece: TetrominoType | null;
  can_hold: boolean;              // False after swap until next piece
  score: number;
  level: number;
  lines_cleared: number;
  is_paused: boolean;
  is_game_over: boolean;
  started_at: string;
  elapsed_ms: number;
}
```

## 9. Public Interfaces
- Game follows standard app manifest (spec 09)
- Save/load via cortex-files (game state serialized as JSON)
- Single instance app

## 10. Internal Interfaces
- Keyboard input from window manager
- File save/load for game state persistence
- Theme tokens for colors

## 11. State Management
- Game state in browser memory during play
- Auto-save on pause or every 30 seconds
- Save file: `/home/{user}/Documents/games/tetris_save.json`
- High score persisted separately: `/home/{user}/Documents/games/tetris_highscore.json`

## 12. Failure Modes and Error Handling
| Failure | Handling |
|---|---|
| Corrupted save file | Start new game, log warning |
| Canvas rendering fails | Show error message, offer restart |

## 13. Security and Permissions
- No special permissions needed
- Reads/writes to own save files via cortex-files

## 14. Performance Requirements
| Metric | Target |
|---|---|
| Render frame rate | 60fps |
| Input latency | < 16ms (one frame) |
| Piece drop animation | Smooth, no jitter |

## 15. Accessibility Requirements
- Keyboard-only controls (already default)
- Piece shapes distinguishable by both color and shape
- Score/level displayed in readable text (not color-only)
- Pause accessible via P key

## 16. Observability and Logging
- Game start/end logged at DEBUG
- No runtime telemetry for games

## 17. Testing Requirements
- Unit: piece rotation (all 4 states for all 7 pieces)
- Unit: wall kick logic
- Unit: line clearing (single, double, triple, tetris)
- Unit: scoring calculation
- Unit: bag randomizer (all 7 unique before repeat)
- Unit: spawn position/orientation for all 7 tetrominoes
- Unit: lock delay reset cap and immediate hard-drop lock
- Unit: top-out detection when canonical spawn is obstructed
- Integration: full game cycle (start → play → game over)
- Visual: ghost piece renders correctly

## 18. Acceptance Criteria
- [ ] All 7 tetrominoes render and rotate correctly
- [ ] Wall kicks work (SRS)
- [ ] Line clearing with animation
- [ ] Scoring: 100/300/500/800 per 1/2/3/4 lines
- [ ] Level progression every 10 lines with speed increase
- [ ] Next piece preview (3 pieces)
- [ ] Hold piece works (one swap per drop)
- [ ] Ghost piece shows landing position
- [ ] Game over detection (piece can't spawn)
- [ ] Spawn position and rotation state are deterministic for every tetromino
- [ ] Lock delay behaves consistently, including reset cap and hard-drop immediate lock
- [ ] Horizontal key-hold behavior matches DAS/ARR requirements
- [ ] High score persisted
- [ ] Pause/resume works
- [ ] 60fps rendering

## 19. Build Order and Dependencies
**Layer 12**. Depends on: 09 (app runtime), 16 (theme tokens), 18 (games parent)

## 20. Non-Goals and Anti-Patterns
- No multiplayer (v1)
- No T-spin detection (v1)
- No combos or back-to-back bonuses (v1)
- No custom themes for game board (v1)
- NEVER allow game to access files outside its save directory

## 21. Implementation Instructions for Claude Code / Codex
1. Define tetromino shapes as rotation matrices (4 rotations × 7 pieces).
2. Implement board as 20×10 2D array with collision detection.
3. Implement SRS wall kick data table.
4. Implement game loop: update (gravity, input) → render (canvas).
5. Implement bag randomizer for piece generation.
6. Implement ghost piece: project piece downward until collision.
7. Implement scoring and level progression.
8. Implement hold piece with one-swap-per-drop rule.
9. Implement save/load via cortex-files.
10. Write tests: rotation, collision, scoring, bag randomizer.
