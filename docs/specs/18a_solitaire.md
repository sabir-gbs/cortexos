# 18a. Solitaire (Klondike)

## 1. Purpose

The Solitaire game provides a classic Klondike solitaire experience for CortexOS, featuring standard tableau, stock, waste, and foundation piles with drag-and-drop card movement, double-click auto-move, unlimited undo, and both Standard and Vegas scoring modes. It serves as the reference implementation for a mouse-heavy game within the game framework, validating drag-and-drop input handling and complex state management.

## 2. Scope

- Klondike solitaire with standard rules.
- 7 tableau piles (1 to 7 cards, top card face-up).
- 1 stock pile, 1 waste pile, 4 foundation piles (Ace to King by suit).
- Draw-1 mode (one card turned from stock at a time).
- Drag-and-drop card movement between piles.
- Double-click to auto-move a card to its correct foundation pile.
- Right-click or button to auto-move all eligible cards to foundations.
- Unlimited undo (revert any number of moves).
- Two scoring modes: Standard and Vegas.
- Game timer (starts on first card move).
- Win detection and celebration animation.
- App location: `apps/games/solitaire`.

## 3. Out of Scope

- Draw-3 mode (turning three cards from stock at a time) -- deferred to v2.
- Spider solitaire, FreeCell, or other solitaire variants.
- Custom card back designs or themes.
- Statistics tracking beyond high scores (e.g., win percentage, streaks).
- Hints or auto-solve functionality.
- Network features (sharing, leaderboards).
- Sound effects.

## 4. Objectives

1. Implement a fully playable Klondike solitaire with correct rules enforcement.
2. Provide smooth drag-and-drop card interaction as the primary input method, with keyboard alternatives for all actions.
3. Validate the game framework's state serialization with a game that has complex nested state (multiple piles, face-up/face-down cards).
4. Demonstrate unlimited undo with full state snapshots.
5. Implement two distinct scoring systems (Standard and Vegas) to demonstrate the framework's pluggable scoring.

## 5. User-Visible Behavior

### 5.1 Game Layout

The game area is divided into four regions from top to bottom:

1. **Top row**: Stock pile (top-left), Waste pile (next to stock), four Foundation piles (top-right, one per suit).
2. **Tableau area**: Seven tableau columns below the top row. Cards overlap vertically, with face-down cards showing less offset than face-up cards.

### 5.2 Card Representation

- Each card displays its rank (A, 2-10, J, Q, K) and suit symbol (spades, hearts, diamonds, clubs) in the top-left and bottom-right corners.
- Face-down cards show a solid back pattern derived from theme tokens (no images).
- Red suits (hearts, diamonds) use `--game-piece-danger` token. Black suits (spades, clubs) use `--game-piece-primary` token.
- Card size: approximately 70px wide by 100px tall at default window size. Scales with window resize.

### 5.3 Mouse Controls

| Action | Behavior |
|--------|----------|
| Click stock pile | Turn top card from stock to waste. If stock is empty, recycle waste back to stock. |
| Drag card from waste | Pick up top waste card for placement. |
| Drag face-up card from tableau | Pick up that card and all cards below it in the column as a group. |
| Drop on tableau column | Place cards if valid (descending rank, alternating color). Empty columns accept Kings only. |
| Drop on foundation | Place card if valid (same suit, ascending rank from Ace). |
| Double-click a face-up card | Auto-move to foundation if valid. |
| Right-click anywhere | Auto-move all eligible cards from waste and tableau to foundations. |

### 5.4 Keyboard Controls

| Key | Action |
|-----|--------|
| `Tab` | Cycle focus between piles: stock, waste, foundations (left to right), tableau (left to right). |
| `Enter` or `Space` | Pick up focused card / place held cards on focused pile. |
| `Left/Right arrows` | Move focus between adjacent piles in the same row. |
| `Up/Down arrows` | Move focus between top row and tableau. When in tableau, select which card in the column to pick up (topmost face-up card by default; Up to select deeper). |
| `D` | Draw from stock (same as clicking stock). |
| `F` | Auto-move focused card to foundation (same as double-click). |
| `A` | Auto-move all eligible cards to foundations (same as right-click). |
| `Ctrl+Z` | Undo last move. |
| `Ctrl+N` | New game. |
| `Escape` | Pause game. |

### 5.5 Scoring

**Standard Scoring:**

| Action | Points |
|--------|--------|
| Waste to tableau | +5 |
| Waste to foundation | +10 |
| Tableau to foundation | +10 |
| Foundation to tableau | -15 |
| Turn over tableau card | +5 |
| Recycle waste to stock | -20 (draw-1 mode: assessed after first recycle) |

**Vegas Scoring:**

- Start with -52 points (one point per card in the deck).
- Each card moved to a foundation earns +5 points.
- Maximum possible score: 0 (all 52 cards to foundations = 260 - 52 = 208, but Vegas scoring caps at net positive from -52 start).
- No points for moves between tableau or waste. Only foundation placements score.
- If the game is restarted, the score resets to -52.

### 5.6 Win Condition

The game is won when all 52 cards are placed on the four foundation piles (each pile contains 13 cards, Ace through King of one suit). A win triggers:

- Timer stops.
- Win overlay displays: "You Win!", final score, time, and scoring mode.
- Win animation: cards cascade from foundations and bounce across the screen (respects reduced-motion preference; shows static "You Win!" text instead).
- "Play Again" and "Close" buttons.

### 5.7 Unwinnable Detection

Solitaire does not automatically detect unwinnable states in v1. The player may choose to start a new game at any time.

## 6. System Behavior

### 6.1 Game Rules Engine

The rules engine enforces Klondike solitaire rules:

**Tableau rules:**
- Cards are built down in alternating colors: red on black, black on red.
- Only Kings may be placed on empty tableau columns.
- Multiple cards may be moved as a group if they form a valid descending sequence of alternating colors.
- When the top card of a tableau column is face-down, it is automatically flipped face-up.

**Foundation rules:**
- Each foundation pile accepts cards of a single suit, in ascending order from Ace to King.
- Only the top card of each foundation is visible and accessible.

**Stock rules:**
- Clicking the stock turns the top card to the waste pile (draw-1 mode).
- When the stock is empty and the waste has cards, clicking the stock recycles the waste back to the stock in reverse order.
- When both stock and waste are empty, clicking the stock does nothing.

**Card movement validation:**
- Every move is validated before execution. Invalid moves are rejected and the card(s) return to their original position.
- Valid moves are logged at debug level.

### 6.2 Undo System

- Every move creates a state snapshot before execution.
- Undo restores the exact state before the last move.
- Unlimited undo depth (up to the framework's 1000-entry limit; oldest entries pruned).
- Undoable actions: drawing from stock, moving cards, recycling waste, auto-moves.
- Undo is disabled during win animation.

### 6.3 Auto-Complete Detection

When all tableau cards are face-up and no cards remain in the stock or waste, the game offers an "Auto-Complete" button that rapidly moves all remaining cards to the foundations in order. This is optional -- the player can continue manually if preferred.

### 6.4 Timer Behavior

- Timer starts on the first card movement (draw from stock or card placement).
- Timer pauses when the game is paused.
- Timer stops when the game is won.
- Timer is displayed as MM:SS in the toolbar.

## 7. Architecture

```
apps/games/solitaire/
  manifest.json
  package.json
  src/
    main.ts                  # Registers game with runtime
    SolitaireGame.tsx        # Root component
    components/
      GameBoard.tsx          # Layout: stock, waste, foundations, tableau
      Card.tsx               # Single card rendering (face-up and face-down)
      CardPile.tsx           # Generic pile component (stack rendering)
      DragLayer.tsx          # Floating cards during drag
      GameToolbar.tsx        # Score, timer, pause, scoring mode toggle
      WinOverlay.tsx         # Win screen with animation
      AutoCompleteButton.tsx # Auto-complete offer button
    engine/
      SolitaireEngine.ts     # Game rules and validation
      SolitaireState.ts      # Full game state definition
      CardDeck.ts            # Deck creation and shuffling
      MoveValidator.ts       # Move legality checking
      AutoMoveEngine.ts      # Auto-move to foundation logic
      ScoreEngine.ts         # Standard and Vegas scoring
      WinDetector.ts         # Win condition check
      AutoCompleteDetector.ts # Detect when auto-complete is possible
    services/
      StateSerializer.ts     # Serialize/deserialize solitaire state
      HighScoreService.ts    # Persist and retrieve high scores
    hooks/
      useDragDrop.ts         # Mouse drag-and-drop handling
      useKeyboardNav.ts      # Keyboard navigation between piles
      useGameLoop.ts         # Game loop (mostly idle; solitaire is turn-based)
    types.ts
  tests/
    unit/
      engine.test.ts         # Rules, move validation, win detection
      scoring.test.ts        # Standard and Vegas scoring
      deck.test.ts           # Deck creation, shuffle correctness
      state.test.ts          # State serialization round-trip
      autocomplete.test.ts   # Auto-complete detection
    integration/
      gameplay.test.ts       # Full game play-through scenarios
      drag.test.ts           # Drag-and-drop flow
      keyboard.test.ts       # Keyboard-only play-through
      theme.test.ts          # Theme rendering
```

## 8. Data Model

### 8.1 Card

```typescript
interface Card {
  suit: "spades" | "hearts" | "diamonds" | "clubs";
  rank: 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13;
  // rank: 1=Ace, 11=Jack, 12=Queen, 13=King
  id: string;                       // e.g., "hearts-12" for Queen of Hearts
  faceUp: boolean;
}

type Suit = Card["suit"];
type Rank = Card["rank"];

const SUIT_COLOR: Record<Suit, "red" | "black"> = {
  spades: "black",
  hearts: "red",
  diamonds: "red",
  clubs: "black",
};
```

### 8.2 Solitaire State

```typescript
interface SolitaireState {
  schemaVersion: 1;
  status: "idle" | "playing" | "paused" | "won";

  // Piles
  stock: Card[];                    // Top = last element
  waste: Card[];                    // Top = last element
  foundations: [Card[], Card[], Card[], Card[]];  // Index 0-3, top = last element
  tableau: [Card[], Card[], Card[], Card[], Card[], Card[], Card[]]; // 7 columns

  // Scoring
  scoringMode: "standard" | "vegas";
  score: number;

  // Timer
  timerElapsed: number;
  timerStarted: boolean;

  // Undo
  moveHistory: SolitaireMove[];
  undoPointer: number;

  // Meta
  difficulty: "standard";           // Solitaire has one difficulty
  startedAt: string;
  lastSaved: string;
  stockRecycles: number;            // Number of times waste was recycled to stock
}
```

### 8.3 Move Record

```typescript
interface SolitaireMove {
  moveIndex: number;
  timestamp: number;
  action: "draw" | "move" | "recycle" | "auto_move" | "auto_complete";
  source: "stock" | "waste" | "tableau" | "foundation";
  sourceIndex: number;              // Column index for tableau, pile index for foundation
  target: "tableau" | "foundation" | "waste";
  targetIndex: number;
  cards: Card[];                    // Cards moved
  flippedCard: boolean;             // Whether a face-down card was flipped after the move
  scoreDelta: number;               // Points gained or lost from this move
  stateSnapshot: string;            // JSON of full state before move
}
```

### 8.4 Manifest

```typescript
{
  id: "com.cortexos.games.solitaire",
  name: "Solitaire",
  version: "1.0.0",
  description: "Klondike solitaire card game",
  firstParty: true,
  bundled: true,
  category: "games",
  entry: { frontend: "src/main.ts" },
  window: {
    defaultWidth: 800,
    defaultHeight: 600,
    minWidth: 640,
    minHeight: 480,
    resizable: true,
    singleInstance: true
  },
  permissions: {
    required: ["runtime.state", "runtime.lifecycle"],
    optional: ["notifications.send"]
  },
  game: {
    supportsPause: true,
    supportsUndo: true,
    supportsSave: true,
    supportsTimer: true,
    maxPlayers: 1,
    difficulties: ["standard"],
    defaultDifficulty: "standard"
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
interface ISolitaireEngine extends IGameEngine {
  // Deck operations
  createDeck(): Card[];
  shuffleDeck(deck: Card[], seed?: number): Card[];

  // Move validation
  canMoveToTableau(card: Card, targetColumn: number, state: SolitaireState): boolean;
  canMoveToFoundation(card: Card, foundationIndex: number, state: SolitaireState): boolean;
  canPickUp(pile: "waste" | "tableau" | "foundation", index: number, cardIndex: number, state: SolitaireState): Card[];

  // Auto operations
  findAutoFoundationMove(state: SolitaireState): { card: Card; source: string; sourceIndex: number; foundationIndex: number } | null;
  isAutoCompleteAvailable(state: SolitaireState): boolean;
  executeAutoComplete(state: SolitaireState): SolitaireState;

  // Scoring
  calculateMoveScore(move: SolitaireMove, scoringMode: "standard" | "vegas"): number;
}
```

### 9.2 Card Display Interface

```typescript
interface CardDisplayProps {
  card: Card;
  selected: boolean;
  draggable: boolean;
  onClick: () => void;
  onDoubleClick: () => void;
  onDragStart: (e: React.DragEvent) => void;
}
```

## 10. Internal Interfaces

### 10.1 Drag-and-Drop Manager

```typescript
interface DragDropManager {
  beginDrag(source: DragSource, cards: Card[], startEvent: MouseEvent): void;
  updateDrag(moveEvent: MouseEvent): void;
  endDrag(endEvent: MouseEvent): DropTarget | null;
  cancelDrag(): void;
  isDragging(): boolean;
  getDraggedCards(): Card[];
  getDragPosition(): { x: number; y: number };
}

interface DragSource {
  pile: "waste" | "tableau" | "foundation";
  pileIndex: number;
  cardIndex: number;     // Index of the first card being dragged in the pile
}

interface DropTarget {
  pile: "tableau" | "foundation";
  pileIndex: number;
}
```

### 10.2 Keyboard Navigation State

```typescript
interface KeyboardNavState {
  focusedZone: "stock" | "waste" | "foundation" | "tableau";
  focusedPileIndex: number;    // Which pile within the zone
  focusedCardIndex: number;    // Which card within the pile (for tableau depth)
  heldCards: Card[] | null;    // Cards picked up but not yet placed
  heldSource: DragSource | null;
}
```

## 11. State Management

### 11.1 State Layers

1. **Ephemeral**: Drag position, focused pile/card for keyboard navigation, hover effects, animation progress.
2. **Session**: Full `SolitaireState` object. Auto-saved on every move (debounced to 1 per second maximum). Saved immediately on pause.
3. **Persistent**: High scores (top 10), preferred scoring mode (standard or vegas), last used scoring mode.

### 11.2 State Serialization

The full `SolitaireState` is serialized to JSON for save/load. All card identities, face-up/face-down states, pile positions, score, timer, and move history are included.

On deserialization:
- Validate `schemaVersion`. If it does not match `1`, start a new game and log a warning.
- Validate that all 52 cards are present and accounted for across all piles. If card count is wrong, start a new game and log an error.
- Restore timer state and move history for undo.

### 11.3 High Score Persistence

High scores are stored per scoring mode:

```typescript
interface SolitaireHighScore extends HighScoreEntry {
  scoringMode: "standard" | "vegas";
}
```

Maximum 10 entries per scoring mode. High scores are sorted by score descending (standard) or score ascending-closest-to-maximum (Vegas).

## 12. Failure Modes and Error Handling

| Failure | Detection | Recovery |
|---------|-----------|----------|
| State load failure (corrupt JSON) | Deserialization returns null or schema version mismatch | Start new game. Show toast: "Could not restore previous game." Log warning. |
| Card count mismatch on load | Validation finds not exactly 52 cards | Start new game. Log error with card count. |
| Invalid move attempted | MoveValidator returns false | Reject move. Return card(s) to source. No error shown to user (silent rejection is expected behavior). |
| Drop target invalid | DragDropManager returns null on endDrag | Animate cards back to source pile. No error message. |
| Storage quota exceeded | cortex-runtime returns QuotaExceeded | Prune oldest undo history entries. Retry save. If still fails, show toast: "Could not save game. Undo history may be limited." |
| Window too small for board | ResizeObserver reports below minWidth/minHeight | Show message: "Please resize the window to play." Do not render board. |
| Win animation fails | Canvas/DOM error during animation | Skip animation. Show static win overlay. Log error. |

## 13. Security and Permissions

- **Required**: `runtime.state`, `runtime.lifecycle` -- for game state persistence and lifecycle management.
- **Optional**: `notifications.send` -- for notifying the player of a long-running game if the window is in the background (e.g., "Your solitaire game is still in progress").
- No filesystem access needed.
- No network access needed.
- Card state JSON is treated as untrusted input during deserialization (validated before use).
- No `eval()` or dynamic code execution.

## 14. Performance Requirements

- Card drag must update position at 60 fps with no perceptible lag.
- Auto-complete animation must process one card move per 100 ms (10 cards per second), smooth and not frame-rate dependent.
- Full board re-render (after undo or load) must complete within one frame (under 16 ms).
- Memory usage must not exceed 50 MB (52 cards + state + undo history).
- Startup first meaningful paint: under 400 ms.

## 15. Accessibility Requirements

- **Keyboard navigation**: Full game playable via keyboard as described in section 5.4. Tab order follows visual layout. Arrow keys navigate between piles.
- **Screen reader**: Current focus announced (e.g., "Queen of Hearts, waste pile, 3 cards"). Score and timer changes announced via ARIA live region. Win announcement: "Congratulations! You won with a score of X in Y minutes."
- **High contrast**: Cards are distinguishable by rank text and suit symbol, not color alone. Card borders use theme tokens. Face-down cards have a distinct pattern (not just a color difference).
- **Focus indicators**: Focused pile and card have a visible, high-contrast outline (3px solid using `--game-highlight-selected` token).
- **Reduced motion**: Drag animation is instant (card jumps to target). Win animation is replaced with static overlay. Card flip animation is instant.
- **Text sizing**: Card rank and suit text remain readable at 200% browser zoom. Card minimum size may increase to accommodate larger text.

## 16. Observability and Logging

### 16.1 Required Log Events

- `solitaire.launched` (info) -- Game opened with scoring mode.
- `solitaire.new_game` (info) -- New game started.
- `solitaire.draw` (debug) -- Card drawn from stock.
- `solitaire.move` (debug) -- Card moved between piles. Includes source, target, card rank/suit (abbreviated).
- `solitaire.auto_move` (debug) -- Auto-move to foundation.
- `solitaire.undo` (info) -- Undo performed. Includes move index.
- `solitaire.recycle` (debug) -- Waste recycled to stock.
- `solitaire.auto_complete` (info) -- Auto-complete triggered.
- `solitaire.won` (info) -- Game won with score, time, scoring mode.
- `solitaire.gave_up` (info) -- New game started before winning current game.
- `solitaire.score_changed` (debug) -- Score update with delta.
- `solitaire.error` (warn) -- Engine error (error type and recovery action).

No PII is logged. Card sequences are not logged in full.

## 17. Testing Requirements

### 17.1 Unit Tests

- **Deck creation**: 52 cards, 4 suits, 13 ranks each. No duplicates.
- **Shuffle**: Produces a different order each time (probabilistic test with fixed seed for reproducibility).
- **Move validation**: All valid and invalid moves for each pile type. Edge cases: empty tableau columns (King only), foundation sequences, alternating colors.
- **Scoring**: Standard scoring for every move type. Vegas scoring for foundation placements and starting score.
- **Win detection**: Win when all foundations complete. No win when any foundation is incomplete.
- **Auto-complete detection**: Detects when all cards are face-up. Does not trigger when face-down cards remain.
- **Auto-move**: Correctly identifies the best auto-move target for each card.
- **State serialization**: Round-trip test -- serialize, deserialize, verify every card is in the correct position.

### 17.2 Integration Tests

- **Full game win**: Play a predetermined winning game (fixed seed) from start to win.
- **Drag-and-drop**: Drag card from waste to tableau, verify state. Drag group from tableau to tableau, verify state. Drop on invalid target, verify card returns.
- **Keyboard play**: Complete a full game using only keyboard.
- **Undo chain**: Make 10 moves, undo 5, make 3 more, undo 2. Verify state matches expected.
- **Save/load**: Play 20 moves, save, close, reopen, load. Verify exact state.
- **Theme switch**: Verify card colors, board background, and overlays in all three themes.

### 17.3 Accessibility Tests

- AX tree validation: every card, pile, and button has correct ARIA labels.
- Keyboard navigation: Tab and arrow key focus moves through all piles.
- Screen reader: score announcement on change, win announcement on game end.

## 18. Acceptance Criteria

- [ ] Standard Klondike rules are correctly enforced (no invalid moves accepted).
- [ ] 7 tableau piles dealt correctly (1-7 cards, top face-up).
- [ ] Stock draws one card to waste. Recycle works when stock is empty.
- [ ] Drag-and-drop moves cards correctly between all pile types.
- [ ] Double-click auto-moves card to foundation when valid.
- [ ] Right-click auto-moves all eligible cards to foundations.
- [ ] Keyboard controls allow full gameplay without mouse.
- [ ] Undo reverses any number of moves correctly.
- [ ] Standard scoring calculates correctly for all move types.
- [ ] Vegas scoring calculates correctly (starts at -52, +5 per foundation card).
- [ ] Timer starts on first move, pauses on pause, stops on win.
- [ ] Win is detected when all 52 cards are on foundations.
- [ ] Win animation plays (or static overlay with reduced motion).
- [ ] Auto-complete offered when all tableau cards are face-up.
- [ ] Save/load restores exact game state across app restarts.
- [ ] High scores persist across sessions per scoring mode.
- [ ] Help overlay shows complete rules and controls.
- [ ] All three themes render correctly with no hardcoded colors.
- [ ] Game is fully playable via keyboard alone.
- [ ] Screen reader announces game state changes.
- [ ] Unit test coverage >= 85%.
- [ ] Engine test coverage >= 95%.
- [ ] No frame drops during drag operations.

## 19. Build Order and Dependencies
**Layer 12**. Depends on: 09 (app runtime), 16 (theme tokens), 18 (games parent)

### 19.1 Prerequisites

- Spec 18 -- Games Platform Parent (shared game framework).
- `@cortexos/game-framework` (game loop, input, state, chrome).
- `@cortexos/ui-components` (shared UI library).
- `@cortexos/runtime-client` (state persistence).
- `@cortexos/theme` (design token consumer).

### 19.2 Build Position

Solitaire is the **third** game to build (after Snake and Minesweeper). It validates drag-and-drop input handling and complex state serialization with the game framework.

No Rust crate needed. Pure frontend app.

## 20. Non-Goals and Anti-Patterns

### 20.1 Non-Goals

- Draw-3 mode (v2 consideration).
- Other solitaire variants (Spider, FreeCell, Pyramid).
- Hint system or auto-solver.
- Custom card back designs.
- Win/loss statistics tracking beyond high scores.

### 20.2 Anti-Patterns

- **DOM manipulation during drag**: Use React state for drag position, not direct DOM manipulation.
- **Mutating card objects**: Cards are immutable. State changes produce new card arrays.
- **Frame-rate-dependent animation**: Auto-complete timing uses real time, not frame count.
- **Hardcoded card colors**: Use theme tokens for red/black suit colors.
- **Storing full move history without pruning**: Cap at 1000 entries.
- **Blocking main thread during shuffle or deal**: These operations are synchronous but trivial (< 1 ms) and acceptable.

## 21. Implementation Instructions for Claude Code / Codex

### 21.1 Subsystem Ownership

- Solitaire owns: Klondike rules engine, card rendering, drag-and-drop interaction, keyboard navigation, scoring logic, auto-complete logic.
- Solitaire does not own: game loop (framework), timer (framework), state persistence (framework), high score storage (framework), theme system, window management.

### 21.2 Recommended Implementation Order

1. Create `manifest.json` and validate against GameManifest schema.
2. Implement `CardDeck.ts` -- create 52 cards, shuffle with seed support. Write unit tests.
3. Implement `SolitaireState.ts` -- define state and deal function (7 tableau piles). Write unit tests.
4. Implement `MoveValidator.ts` -- all move validation rules. Write unit tests extensively.
5. Implement `SolitaireEngine.ts` -- integrate validator, handle moves, detect win. Write unit tests.
6. Implement `ScoreEngine.ts` -- standard and Vegas scoring. Write unit tests.
7. Implement `AutoMoveEngine.ts` and `AutoCompleteDetector.ts`. Write unit tests.
8. Build `Card.tsx` component with face-up and face-down rendering.
9. Build `CardPile.tsx` and `GameBoard.tsx` layout.
10. Implement `useDragDrop.ts` hook for mouse drag-and-drop.
11. Implement `useKeyboardNav.ts` hook for keyboard navigation.
12. Integrate with game framework for loop, timer, scoring, save/load.
13. Build toolbar, pause overlay, win overlay, and help overlay.
14. Implement `StateSerializer.ts` and test round-trip.
15. Accessibility audit and fixes.
16. Theme verification (light, dark, high-contrast).

### 21.3 What Can Be Stubbed Initially

- Win animation can be a simple overlay initially.
- Auto-complete animation can place cards instantly without smooth transitions.
- Keyboard navigation can be basic (stock draw and card placement) before full arrow-key navigation.

### 21.4 What Must Be Real in v1

- Complete Klondike rules engine with all validation.
- Drag-and-drop for all valid move types.
- Double-click and right-click auto-move.
- Full keyboard navigation (all actions accessible).
- Both scoring modes (Standard and Vegas).
- Unlimited undo.
- Save/load with full state restoration.
- Win detection and overlay.
- Auto-complete offer and execution.
- Help overlay with complete rules.
- Theme support (all three themes).
- Accessibility (keyboard, screen reader, high contrast).

### 21.5 What Cannot Be Inferred

- Card dimensions (70x100px at default window size, scaled proportionally).
- Tableau card vertical offset: 25px for face-down cards, 35px for face-up cards.
- Stock pile visual: top card only visible, others represented by a subtle stack shadow.
- Waste pile visual: top 1-3 cards fanned slightly (only top card interactive in draw-1 mode).

### 21.6 Stop Conditions

1. All acceptance criteria in section 18 pass.
2. Engine unit test coverage >= 95%.
3. Overall unit test coverage >= 85%.
4. Integration tests for full game win, drag-and-drop, keyboard play, undo, and save/load pass.
5. Accessibility tests pass.
6. No hardcoded colors (all theme tokens).
7. No linter warnings or type errors.

### 21.7 Testing Gates

- Engine and scoring unit tests must pass before UI work begins.
- Move validation tests must cover every valid and invalid move scenario.
- Keyboard-only play-through must succeed before merge.
- Save/load round-trip must pass before merge.
- Performance: drag at 60 fps with no dropped frames verified.
