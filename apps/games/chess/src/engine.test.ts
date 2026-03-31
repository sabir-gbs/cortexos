import { describe, it, expect } from 'vitest';
import {
  createInitialBoard,
  getValidMoves,
  inBounds,
  type Board,
  type Position,
  type Piece,
} from './engine';

function emptyBoard(): Board {
  return Array.from({ length: 8 }, () => Array(8).fill(null));
}

function place(board: Board, row: number, col: number, piece: Piece): void {
  board[row][col] = piece;
}

function hasMove(moves: Position[], row: number, col: number): boolean {
  return moves.some(m => m.row === row && m.col === col);
}

describe('createInitialBoard', () => {
  it('has correct starting position for white pieces', () => {
    const board = createInitialBoard();
    // White back row (row 7)
    expect(board[7][0]).toEqual({ type: 'R', color: 'white' });
    expect(board[7][1]).toEqual({ type: 'N', color: 'white' });
    expect(board[7][2]).toEqual({ type: 'B', color: 'white' });
    expect(board[7][3]).toEqual({ type: 'Q', color: 'white' });
    expect(board[7][4]).toEqual({ type: 'K', color: 'white' });
    expect(board[7][5]).toEqual({ type: 'B', color: 'white' });
    expect(board[7][6]).toEqual({ type: 'N', color: 'white' });
    expect(board[7][7]).toEqual({ type: 'R', color: 'white' });
    // White pawns (row 6)
    for (let c = 0; c < 8; c++) {
      expect(board[6][c]).toEqual({ type: 'P', color: 'white' });
    }
  });

  it('has correct starting position for black pieces', () => {
    const board = createInitialBoard();
    // Black back row (row 0)
    expect(board[0][0]).toEqual({ type: 'R', color: 'black' });
    expect(board[0][1]).toEqual({ type: 'N', color: 'black' });
    expect(board[0][2]).toEqual({ type: 'B', color: 'black' });
    expect(board[0][3]).toEqual({ type: 'Q', color: 'black' });
    expect(board[0][4]).toEqual({ type: 'K', color: 'black' });
    expect(board[0][5]).toEqual({ type: 'B', color: 'black' });
    expect(board[0][6]).toEqual({ type: 'N', color: 'black' });
    expect(board[0][7]).toEqual({ type: 'R', color: 'black' });
    // Black pawns (row 1)
    for (let c = 0; c < 8; c++) {
      expect(board[1][c]).toEqual({ type: 'P', color: 'black' });
    }
  });

  it('has empty middle rows', () => {
    const board = createInitialBoard();
    for (let r = 2; r <= 5; r++) {
      for (let c = 0; c < 8; c++) {
        expect(board[r][c]).toBeNull();
      }
    }
  });
});

describe('inBounds', () => {
  it('returns true for valid positions', () => {
    expect(inBounds(0, 0)).toBe(true);
    expect(inBounds(7, 7)).toBe(true);
    expect(inBounds(3, 4)).toBe(true);
  });

  it('returns false for out-of-bounds positions', () => {
    expect(inBounds(-1, 0)).toBe(false);
    expect(inBounds(0, -1)).toBe(false);
    expect(inBounds(8, 0)).toBe(false);
    expect(inBounds(0, 8)).toBe(false);
    expect(inBounds(-1, -1)).toBe(false);
    expect(inBounds(8, 8)).toBe(false);
  });
});

describe('getValidMoves - Pawn', () => {
  it('white pawn can move forward 1 from non-start row', () => {
    const board = emptyBoard();
    place(board, 4, 3, { type: 'P', color: 'white' });
    const moves = getValidMoves(board, { row: 4, col: 3 });
    // White moves upward (row decreases)
    expect(hasMove(moves, 3, 3)).toBe(true);
    // Should NOT get 2-square move since not on start row
    expect(hasMove(moves, 2, 3)).toBe(false);
  });

  it('white pawn can move forward 1 or 2 from start row', () => {
    const board = emptyBoard();
    place(board, 6, 3, { type: 'P', color: 'white' });
    const moves = getValidMoves(board, { row: 6, col: 3 });
    expect(hasMove(moves, 5, 3)).toBe(true);
    expect(hasMove(moves, 4, 3)).toBe(true);
  });

  it('black pawn can move forward 1 or 2 from start row', () => {
    const board = emptyBoard();
    place(board, 1, 3, { type: 'P', color: 'black' });
    const moves = getValidMoves(board, { row: 1, col: 3 });
    expect(hasMove(moves, 2, 3)).toBe(true);
    expect(hasMove(moves, 3, 3)).toBe(true);
  });

  it('pawn cannot move through a piece', () => {
    const board = emptyBoard();
    place(board, 6, 3, { type: 'P', color: 'white' });
    place(board, 5, 3, { type: 'P', color: 'black' }); // blocks path
    const moves = getValidMoves(board, { row: 6, col: 3 });
    // Cannot move forward at all (blocked)
    expect(hasMove(moves, 5, 3)).toBe(false);
    expect(hasMove(moves, 4, 3)).toBe(false);
  });

  it('pawn cannot double-jump if first square is blocked', () => {
    const board = emptyBoard();
    place(board, 6, 3, { type: 'P', color: 'white' });
    place(board, 5, 3, { type: 'P', color: 'black' });
    const moves = getValidMoves(board, { row: 6, col: 3 });
    expect(hasMove(moves, 4, 3)).toBe(false);
  });

  it('pawn can capture diagonally', () => {
    const board = emptyBoard();
    place(board, 4, 3, { type: 'P', color: 'white' });
    place(board, 3, 2, { type: 'P', color: 'black' });
    place(board, 3, 4, { type: 'P', color: 'black' });
    const moves = getValidMoves(board, { row: 4, col: 3 });
    expect(hasMove(moves, 3, 2)).toBe(true);
    expect(hasMove(moves, 3, 4)).toBe(true);
    // Forward move should also be available
    expect(hasMove(moves, 3, 3)).toBe(true);
  });

  it('pawn cannot capture forward', () => {
    const board = emptyBoard();
    place(board, 4, 3, { type: 'P', color: 'white' });
    place(board, 3, 3, { type: 'P', color: 'black' });
    const moves = getValidMoves(board, { row: 4, col: 3 });
    expect(hasMove(moves, 3, 3)).toBe(false);
  });

  it('pawn cannot capture own color diagonally', () => {
    const board = emptyBoard();
    place(board, 4, 3, { type: 'P', color: 'white' });
    place(board, 3, 2, { type: 'P', color: 'white' });
    place(board, 3, 4, { type: 'P', color: 'white' });
    const moves = getValidMoves(board, { row: 4, col: 3 });
    expect(hasMove(moves, 3, 2)).toBe(false);
    expect(hasMove(moves, 3, 4)).toBe(false);
  });
});

describe('getValidMoves - Rook', () => {
  it('moves horizontally and vertically on empty board', () => {
    const board = emptyBoard();
    place(board, 4, 4, { type: 'R', color: 'white' });
    const moves = getValidMoves(board, { row: 4, col: 4 });
    // Up
    expect(hasMove(moves, 3, 4)).toBe(true);
    expect(hasMove(moves, 2, 4)).toBe(true);
    expect(hasMove(moves, 1, 4)).toBe(true);
    expect(hasMove(moves, 0, 4)).toBe(true);
    // Down
    expect(hasMove(moves, 5, 4)).toBe(true);
    expect(hasMove(moves, 6, 4)).toBe(true);
    expect(hasMove(moves, 7, 4)).toBe(true);
    // Left
    expect(hasMove(moves, 4, 3)).toBe(true);
    expect(hasMove(moves, 4, 2)).toBe(true);
    expect(hasMove(moves, 4, 1)).toBe(true);
    expect(hasMove(moves, 4, 0)).toBe(true);
    // Right
    expect(hasMove(moves, 4, 5)).toBe(true);
    expect(hasMove(moves, 4, 6)).toBe(true);
    expect(hasMove(moves, 4, 7)).toBe(true);
    // Total: 14 squares (7 in each of 4 directions minus center)
    expect(moves).toHaveLength(14);
  });

  it('is blocked by friendly piece and cannot move past it', () => {
    const board = emptyBoard();
    place(board, 4, 4, { type: 'R', color: 'white' });
    place(board, 2, 4, { type: 'P', color: 'white' }); // block upward
    const moves = getValidMoves(board, { row: 4, col: 4 });
    // Should be able to move to row 3 but not row 2 or beyond
    expect(hasMove(moves, 3, 4)).toBe(true);
    expect(hasMove(moves, 2, 4)).toBe(false);
    expect(hasMove(moves, 1, 4)).toBe(false);
    expect(hasMove(moves, 0, 4)).toBe(false);
  });

  it('can capture enemy piece but stops after it', () => {
    const board = emptyBoard();
    place(board, 4, 4, { type: 'R', color: 'white' });
    place(board, 2, 4, { type: 'P', color: 'black' });
    const moves = getValidMoves(board, { row: 4, col: 4 });
    expect(hasMove(moves, 2, 4)).toBe(true);
    expect(hasMove(moves, 1, 4)).toBe(false);
    expect(hasMove(moves, 0, 4)).toBe(false);
  });

  it('does not include diagonal moves', () => {
    const board = emptyBoard();
    place(board, 4, 4, { type: 'R', color: 'white' });
    const moves = getValidMoves(board, { row: 4, col: 4 });
    expect(hasMove(moves, 3, 3)).toBe(false);
    expect(hasMove(moves, 5, 5)).toBe(false);
  });
});

describe('getValidMoves - Bishop', () => {
  it('moves diagonally on empty board', () => {
    const board = emptyBoard();
    place(board, 4, 4, { type: 'B', color: 'white' });
    const moves = getValidMoves(board, { row: 4, col: 4 });
    // Up-left diagonal
    expect(hasMove(moves, 3, 3)).toBe(true);
    expect(hasMove(moves, 2, 2)).toBe(true);
    expect(hasMove(moves, 1, 1)).toBe(true);
    expect(hasMove(moves, 0, 0)).toBe(true);
    // Up-right diagonal
    expect(hasMove(moves, 3, 5)).toBe(true);
    expect(hasMove(moves, 2, 6)).toBe(true);
    expect(hasMove(moves, 1, 7)).toBe(true);
    // Down-left diagonal
    expect(hasMove(moves, 5, 3)).toBe(true);
    expect(hasMove(moves, 6, 2)).toBe(true);
    expect(hasMove(moves, 7, 1)).toBe(true);
    // Down-right diagonal
    expect(hasMove(moves, 5, 5)).toBe(true);
    expect(hasMove(moves, 6, 6)).toBe(true);
    expect(hasMove(moves, 7, 7)).toBe(true);
    // Total: 13 squares on diagonals from center
    expect(moves).toHaveLength(13);
  });

  it('is blocked by friendly piece', () => {
    const board = emptyBoard();
    place(board, 4, 4, { type: 'B', color: 'white' });
    place(board, 2, 2, { type: 'P', color: 'white' });
    const moves = getValidMoves(board, { row: 4, col: 4 });
    expect(hasMove(moves, 3, 3)).toBe(true);
    expect(hasMove(moves, 2, 2)).toBe(false);
    expect(hasMove(moves, 1, 1)).toBe(false);
  });

  it('can capture enemy piece and stops', () => {
    const board = emptyBoard();
    place(board, 4, 4, { type: 'B', color: 'white' });
    place(board, 2, 2, { type: 'P', color: 'black' });
    const moves = getValidMoves(board, { row: 4, col: 4 });
    expect(hasMove(moves, 2, 2)).toBe(true);
    expect(hasMove(moves, 1, 1)).toBe(false);
  });
});

describe('getValidMoves - Knight', () => {
  it('has L-shape moves from center', () => {
    const board = emptyBoard();
    place(board, 4, 4, { type: 'N', color: 'white' });
    const moves = getValidMoves(board, { row: 4, col: 4 });
    const expected: [number, number][] = [
      [2, 3], [2, 5], [3, 2], [3, 6],
      [5, 2], [5, 6], [6, 3], [6, 5],
    ];
    expect(moves).toHaveLength(8);
    for (const [r, c] of expected) {
      expect(hasMove(moves, r, c)).toBe(true);
    }
  });

  it('can jump over pieces', () => {
    const board = emptyBoard();
    // Surround the knight with pieces
    place(board, 4, 4, { type: 'N', color: 'white' });
    place(board, 3, 4, { type: 'P', color: 'white' });
    place(board, 5, 4, { type: 'P', color: 'white' });
    place(board, 4, 3, { type: 'P', color: 'white' });
    place(board, 4, 5, { type: 'P', color: 'white' });
    const moves = getValidMoves(board, { row: 4, col: 4 });
    // Knight should still be able to jump to all 8 L-shaped squares
    expect(moves).toHaveLength(8);
  });

  it('cannot move to square occupied by friendly piece', () => {
    const board = emptyBoard();
    place(board, 4, 4, { type: 'N', color: 'white' });
    place(board, 2, 3, { type: 'P', color: 'white' });
    const moves = getValidMoves(board, { row: 4, col: 4 });
    expect(hasMove(moves, 2, 3)).toBe(false);
    expect(moves).toHaveLength(7);
  });

  it('can capture enemy piece', () => {
    const board = emptyBoard();
    place(board, 4, 4, { type: 'N', color: 'white' });
    place(board, 2, 3, { type: 'P', color: 'black' });
    const moves = getValidMoves(board, { row: 4, col: 4 });
    expect(hasMove(moves, 2, 3)).toBe(true);
  });

  it('handles edge of board correctly (fewer moves)', () => {
    const board = emptyBoard();
    place(board, 0, 0, { type: 'N', color: 'white' });
    const moves = getValidMoves(board, { row: 0, col: 0 });
    // From (0,0) only (1,2) and (2,1) are valid
    expect(moves).toHaveLength(2);
    expect(hasMove(moves, 1, 2)).toBe(true);
    expect(hasMove(moves, 2, 1)).toBe(true);
  });
});

describe('getValidMoves - Queen', () => {
  it('combines rook and bishop moves on empty board', () => {
    const board = emptyBoard();
    place(board, 4, 4, { type: 'Q', color: 'white' });
    const moves = getValidMoves(board, { row: 4, col: 4 });
    // Queen should have rook moves + bishop moves = 14 + 13 = 27
    expect(moves).toHaveLength(27);
  });

  it('is blocked by pieces in all directions', () => {
    const board = emptyBoard();
    place(board, 4, 4, { type: 'Q', color: 'white' });
    // Block in each direction
    place(board, 3, 4, { type: 'P', color: 'white' }); // up
    place(board, 5, 4, { type: 'P', color: 'white' }); // down
    place(board, 4, 3, { type: 'P', color: 'white' }); // left
    place(board, 4, 5, { type: 'P', color: 'white' }); // right
    place(board, 3, 3, { type: 'P', color: 'white' }); // up-left
    place(board, 3, 5, { type: 'P', color: 'white' }); // up-right
    place(board, 5, 3, { type: 'P', color: 'white' }); // down-left
    place(board, 5, 5, { type: 'P', color: 'white' }); // down-right
    const moves = getValidMoves(board, { row: 4, col: 4 });
    expect(moves).toHaveLength(0);
  });
});

describe('getValidMoves - King', () => {
  it('moves one square in any direction from center', () => {
    const board = emptyBoard();
    place(board, 4, 4, { type: 'K', color: 'white' });
    const moves = getValidMoves(board, { row: 4, col: 4 });
    const expected: [number, number][] = [
      [3, 3], [3, 4], [3, 5],
      [4, 3],         [4, 5],
      [5, 3], [5, 4], [5, 5],
    ];
    expect(moves).toHaveLength(8);
    for (const [r, c] of expected) {
      expect(hasMove(moves, r, c)).toBe(true);
    }
  });

  it('cannot move to square occupied by friendly piece', () => {
    const board = emptyBoard();
    place(board, 4, 4, { type: 'K', color: 'white' });
    place(board, 3, 4, { type: 'P', color: 'white' });
    const moves = getValidMoves(board, { row: 4, col: 4 });
    expect(hasMove(moves, 3, 4)).toBe(false);
    expect(moves).toHaveLength(7);
  });

  it('can capture enemy piece', () => {
    const board = emptyBoard();
    place(board, 4, 4, { type: 'K', color: 'white' });
    place(board, 3, 4, { type: 'P', color: 'black' });
    const moves = getValidMoves(board, { row: 4, col: 4 });
    expect(hasMove(moves, 3, 4)).toBe(true);
    expect(moves).toHaveLength(8);
  });

  it('handles corner correctly (only 3 moves)', () => {
    const board = emptyBoard();
    place(board, 0, 0, { type: 'K', color: 'white' });
    const moves = getValidMoves(board, { row: 0, col: 0 });
    expect(moves).toHaveLength(3);
    expect(hasMove(moves, 0, 1)).toBe(true);
    expect(hasMove(moves, 1, 0)).toBe(true);
    expect(hasMove(moves, 1, 1)).toBe(true);
  });
});

describe('getValidMoves - general', () => {
  it('returns empty array for empty square', () => {
    const board = emptyBoard();
    const moves = getValidMoves(board, { row: 3, col: 3 });
    expect(moves).toEqual([]);
  });

  it('cannot capture own pieces', () => {
    const board = emptyBoard();
    // Place a white rook surrounded by white pawns
    place(board, 4, 4, { type: 'R', color: 'white' });
    place(board, 3, 4, { type: 'P', color: 'white' });
    place(board, 5, 4, { type: 'P', color: 'white' });
    place(board, 4, 3, { type: 'P', color: 'white' });
    place(board, 4, 5, { type: 'P', color: 'white' });
    const moves = getValidMoves(board, { row: 4, col: 4 });
    // No moves should be available in the four cardinal directions
    expect(hasMove(moves, 3, 4)).toBe(false);
    expect(hasMove(moves, 5, 4)).toBe(false);
    expect(hasMove(moves, 4, 3)).toBe(false);
    expect(hasMove(moves, 4, 5)).toBe(false);
  });
});
