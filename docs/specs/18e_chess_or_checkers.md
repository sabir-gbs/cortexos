# 18e — Chess

## 1. Purpose
Define the chess game bundled with CortexOS as a first-party game. Two-player local (human vs human, no AI in v1).

## 2. Scope
- Standard chess rules (all piece movements, captures)
- Castling (kingside and queenside)
- En passant
- Pawn promotion
- Check, checkmate, stalemate detection
- Legal move highlighting
- Move history (algebraic notation)
- Captured pieces display
- Undo
- Board themes matching OS theme

## 3. Out of Scope
- AI opponent (v1)
- Online multiplayer
- Chess clock/timer (v1)
- Opening book or endgame tablebases
- PGN import/export (v1)

## 4. Objectives
1. Complete and correct chess rule implementation.
2. All special moves (castling, en passant, promotion) work correctly.
3. Legal move validation prevents all illegal moves.

## 5. User-Visible Behavior

| Action | Outcome |
|---|---|
| User clicks a piece | Legal move squares highlighted |
| User clicks highlighted square | Piece moves, turn switches |
| User tries illegal move | Nothing happens (piece stays) |
| King in check | Check indicator shown, must escape |
| Checkmate | Game over message, winner announced |
| Stalemate | Draw message |
| User clicks Undo | Last move reversed |
| Pawn reaches last rank | Promotion dialog appears (Queen/Rook/Bishop/Knight) |

## 6. System Behavior

### 6.1 Piece Movement Rules

| Piece | Movement |
|---|---|
| King | 1 square any direction (8 possible moves) |
| Queen | Any distance, any direction (straight + diagonal) |
| Rook | Any distance, straight lines (horizontal + vertical) |
| Bishop | Any distance, diagonals only |
| Knight | L-shape: 2+1 or 1+2 squares (8 possible moves, can jump) |
| Pawn | Forward 1 (or 2 from start), capture diagonally forward 1 |

### 6.2 Special Moves

**Castling** (king moves 2 squares toward rook):
- Conditions: king and rook haven't moved, no pieces between, king not in check, king doesn't pass through check, king doesn't land in check
- Kingside: King e1→g1, Rook h1→f1 (white)
- Queenside: King e1→c1, Rook a1→d1 (white)

**En passant**:
- When opponent's pawn advances 2 squares from start, landing beside your pawn
- Your pawn can capture it as if it had only moved 1 square
- Must be done immediately (next move only)

**Pawn promotion**:
- When pawn reaches opponent's back rank (row 8 for white, row 1 for black)
- Player chooses: Queen, Rook, Bishop, or Knight
- Promotion dialog shown with piece selection

### 6.3 Check/Checkmate/Stalemate

**Check**: King is attacked by opponent piece. Player must escape check on their turn.
- Escape by: moving king, blocking, or capturing attacker

**Checkmate**: King is in check AND no legal moves exist. Game over. Attacker wins.

**Stalemate**: King is NOT in check AND no legal moves exist. Draw.

### 6.3.1 Additional Draw Conditions

The game must also declare a draw for these standard rule states:

- **Threefold repetition**: the same position occurs for the third time with the same side to move, identical castling rights, and identical en passant availability.
- **Fifty-move rule**: 50 full moves by each side occur without any pawn move or capture.
- **Insufficient material**: neither side has enough material to force checkmate, at minimum covering:
  - king versus king
  - king and bishop versus king
  - king and knight versus king
  - king and bishop versus king and bishop when both bishops remain on the same color complex
- **Dead position**: any position from which checkmate cannot occur by any legal sequence, even if not covered by the simpler insufficient-material shortcuts above.

### 6.4 Legal Move Validation
A move is legal if:
1. Piece can make that movement per its rules
2. Path is not blocked (except knight)
3. Destination is empty or contains opponent piece (capture)
4. Move does not leave own king in check
5. Special move conditions met (castling, en passant)

## 7. Architecture
```
┌────────────────────────────────┐
│      Chess Game (TS)           │
│  ┌──────────────────────────┐  │
│  │  Board Model             │  │
│  │  (8×8 array, pieces)     │  │
│  └──────────┬───────────────┘  │
│  ┌──────────┴───────────────┐  │
│  │  Move Validator          │  │
│  │  (legal moves, check,    │  │
│  │   checkmate, stalemate)  │  │
│  └──────────┬───────────────┘  │
│  ┌──────────┴───────────────┐  │
│  │  Game Controller         │  │
│  │  (turns, undo, state)    │  │
│  └──────────┬───────────────┘  │
│  ┌──────────┴───────────────┐  │
│  │  Renderer (Canvas/DOM)   │  │
│  └──────────────────────────┘  │
└────────────────────────────────┘
```

## 8. Data Model
```typescript
type PieceType = 'king' | 'queen' | 'rook' | 'bishop' | 'knight' | 'pawn';
type Color = 'white' | 'black';

interface Piece {
  type: PieceType;
  color: Color;
  has_moved: boolean;          // For castling and en passant tracking
}

type Board = (Piece | null)[][];  // 8×8, board[0][0] = a8, board[7][7] = h1

interface Position {
  row: number;  // 0-7
  col: number;  // 0-7
}

interface Move {
  from: Position;
  to: Position;
  piece: Piece;
  captured: Piece | null;
  is_castle: boolean;
  is_en_passant: boolean;
  promotion: PieceType | null;
}

interface GameState {
  board: Board;
  current_turn: Color;
  move_history: Move[];
  captured_pieces: { white: PieceType[]; black: PieceType[] };
  is_check: boolean;
  is_checkmate: boolean;
  is_stalemate: boolean;
  is_draw_by_repetition: boolean;
  is_draw_by_fifty_move_rule: boolean;
  is_draw_by_insufficient_material: boolean;
  is_dead_position: boolean;
  halfmove_clock: number;
  position_hash_history: string[];
  en_passant_target: Position | null;  // Square that can be captured en passant
  castling_rights: {
    white_kingside: boolean;
    white_queenside: boolean;
    black_kingside: boolean;
    black_queenside: boolean;
  };
  selected_square: Position | null;
  legal_moves: Position[];
  started_at: string;
}
```

### Initial Board Setup
```
Row 0: r n b q k b n r  (black pieces)
Row 1: p p p p p p p p  (black pawns)
Row 2-5: . . . . . . . .  (empty)
Row 6: P P P P P P P P  (white pawns)
Row 7: R N B Q K B N R  (white pieces)
```

## 9. Public Interfaces
- Standard app manifest (spec 09)
- Save/load via cortex-files
- Single instance app

## 10. Internal Interfaces
- Click/touch input from window manager
- Theme tokens for board colors
- File save/load for game state

## 11. State Management
- Game state in browser memory
- Auto-save on every move
- Save file: `/home/{user}/Documents/games/chess_save.json`
- Move history persisted with game state

## 12. Failure Modes and Error Handling
| Failure | Handling |
|---|---|
| Corrupted save | Start new game, log warning |
| Invalid move attempt | Silently ignored (piece stays) |

## 13. Security and Permissions
- No special permissions
- Reads/writes own save files

## 14. Performance Requirements
| Metric | Target |
|---|---|
| Legal move calculation | < 10ms (for all pieces) |
| Board render | < 16ms (60fps) |
| Move animation | 200ms |

## 15. Accessibility Requirements
- Pieces distinguishable by shape, not color alone (white/black outlines)
- Legal move highlights use pattern + color
- Move history in readable text
- Keyboard: Arrow keys to navigate board, Enter to select/move

## 16. Observability and Logging
- Game start/end logged at DEBUG
- No runtime telemetry

## 17. Testing Requirements
- Unit: each piece's legal moves on empty board
- Unit: castling (all conditions: moved pieces, check, blocked)
- Unit: en passant (trigger and expiry)
- Unit: pawn promotion (all 4 choices)
- Unit: check detection
- Unit: checkmate detection (classic positions: back rank, scholar's mate)
- Unit: stalemate detection
- Unit: threefold repetition detection with castling/en-passant state included in the position identity
- Unit: fifty-move rule counter reset on pawn moves and captures
- Unit: insufficient material detection
- Unit: dead-position detection fallback
- Unit: move leaves king in check (must be illegal)
- Integration: full game from start to checkmate
- Integration: undo restores complete state including castling rights

## 18. Acceptance Criteria
- [ ] All 6 piece types move correctly per chess rules
- [ ] Castling works (kingside + queenside, all conditions enforced)
- [ ] En passant works (only on immediately following move)
- [ ] Pawn promotion shows dialog, all 4 pieces selectable
- [ ] Check detected and indicated visually
- [ ] Checkmate detected, game ends, winner shown
- [ ] Stalemate detected, game ends, draw shown
- [ ] Threefold repetition, fifty-move rule, insufficient material, and dead position all resolve to draw correctly
- [ ] Legal moves highlighted when piece selected
- [ ] Illegal moves blocked (including moves leaving king in check)
- [ ] Move history displayed in algebraic notation
- [ ] Captured pieces displayed
- [ ] Undo works (reverses all state including special move rights)
- [ ] Board theme matches OS theme
- [ ] Game state saved and restored

## 19. Build Order and Dependencies
**Layer 12**. Depends on: 09 (app runtime), 16 (theme tokens), 18 (games parent)

## 20. Non-Goals and Anti-Patterns
- No AI opponent (v1)
- No online play (v1)
- No chess clock (v1)
- No PGN/FEN import/export (v1)
- No draw offers or resignation (v1 — just close game)
- NEVER allow illegal moves to execute
- NEVER skip check validation

## 21. Implementation Instructions for Claude Code / Codex
1. Define Board, Piece, Position, Move, GameState types.
2. Implement initial board setup.
3. Implement move generation for each piece type.
4. Implement legal move filter (remove moves that leave king in check).
5. Implement check detection (is king attacked by any opponent piece).
6. Implement checkmate detection (in check + no legal moves).
7. Implement stalemate detection (not in check + no legal moves).
8. Implement draw detection for repetition, fifty-move rule, insufficient material, and dead positions.
9. Implement castling: track piece movement, validate all conditions.
10. Implement en passant: track double-pawn-advance target, expire after one turn.
11. Implement pawn promotion: dialog UI, replace piece on board.
12. Implement undo: stack of moves with full state snapshots.
13. Implement algebraic notation conversion for move history.
14. Implement board renderer with theme-aware colors.
15. Write tests: every piece type, every special move, check/checkmate/stalemate/draw.
