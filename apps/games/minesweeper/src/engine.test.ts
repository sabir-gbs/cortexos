import { describe, it, expect } from 'vitest';
import {
  createEmptyBoard,
  placeMines,
  revealEmpty,
  checkWin,
  ROWS,
  COLS,
  MINE_COUNT,
  type Board,
} from './engine';

describe('createEmptyBoard', () => {
  it('creates a board with correct dimensions', () => {
    const board = createEmptyBoard();
    expect(board).toHaveLength(ROWS);
    for (const row of board) {
      expect(row).toHaveLength(COLS);
    }
  });

  it('creates all cells as unrevealed, unflagged, with no mines', () => {
    const board = createEmptyBoard();
    for (let r = 0; r < ROWS; r++) {
      for (let c = 0; c < COLS; c++) {
        const cell = board[r][c];
        expect(cell.mine).toBe(false);
        expect(cell.revealed).toBe(false);
        expect(cell.flagged).toBe(false);
        expect(cell.adjacentMines).toBe(0);
      }
    }
  });
});

describe('placeMines', () => {
  it('places the correct number of mines', () => {
    const board = createEmptyBoard();
    const result = placeMines(board, 0, 0);
    let mineCount = 0;
    for (let r = 0; r < ROWS; r++) {
      for (let c = 0; c < COLS; c++) {
        if (result[r][c].mine) mineCount++;
      }
    }
    expect(mineCount).toBe(MINE_COUNT);
  });

  it('excludes the clicked cell from receiving a mine', () => {
    const board = createEmptyBoard();
    const excludeRow = 4;
    const excludeCol = 4;
    // Run multiple times to reduce chance of random false positive
    for (let i = 0; i < 20; i++) {
      const result = placeMines(createEmptyBoard(), excludeRow, excludeCol);
      expect(result[excludeRow][excludeCol].mine).toBe(false);
    }
  });

  it('calculates correct adjacent mine counts', () => {
    const board = createEmptyBoard();
    const result = placeMines(board, 0, 0);
    // Recalculate adjacent mine counts independently and verify
    for (let r = 0; r < ROWS; r++) {
      for (let c = 0; c < COLS; c++) {
        if (result[r][c].mine) continue;
        let count = 0;
        for (let dr = -1; dr <= 1; dr++) {
          for (let dc = -1; dc <= 1; dc++) {
            const nr = r + dr;
            const nc = c + dc;
            if (
              nr >= 0 && nr < ROWS &&
              nc >= 0 && nc < COLS &&
              result[nr][nc].mine
            ) {
              count++;
            }
          }
        }
        expect(result[r][c].adjacentMines).toBe(count);
      }
    }
  });

  it('does not mutate the original board', () => {
    const board = createEmptyBoard();
    const original = board.map(r => r.map(c => ({ ...c })));
    placeMines(board, 0, 0);
    expect(board).toEqual(original);
  });
});

describe('revealEmpty', () => {
  it('reveals the clicked cell', () => {
    const board = createEmptyBoard();
    const result = revealEmpty(board, 4, 4);
    expect(result[4][4].revealed).toBe(true);
  });

  it('flood fills through empty cells (adjacentMines = 0)', () => {
    // Create a board with no mines so all cells are empty (adjacentMines = 0)
    const board = createEmptyBoard();
    const result = revealEmpty(board, 0, 0);
    // Every cell should be revealed since there are no mines and all adjacentMines = 0
    for (let r = 0; r < ROWS; r++) {
      for (let c = 0; c < COLS; c++) {
        expect(result[r][c].revealed).toBe(true);
      }
    }
  });

  it('stops at numbered cells (does not reveal past them)', () => {
    // Build a board where a single mine is surrounded by empty cells
    const board = createEmptyBoard();
    // Place a mine at (2,2)
    board[2][2].mine = true;
    // Calculate adjacent counts manually
    for (let r = 1; r <= 3; r++) {
      for (let c = 1; c <= 3; c++) {
        if (r === 2 && c === 2) continue;
        board[r][c].adjacentMines = 1;
      }
    }
    // All other cells remain 0

    // Reveal starting from (0,0) which has no adjacent mines
    const result = revealEmpty(board, 0, 0);

    // Cells far from the mine should be revealed (they have 0 adjacent mines)
    expect(result[0][0].revealed).toBe(true);
    expect(result[0][1].revealed).toBe(true);

    // Numbered cells adjacent to the mine should be revealed (flood fills INTO them but stops there)
    expect(result[1][1].revealed).toBe(true);

    // Cells beyond the numbered boundary should NOT be revealed
    expect(result[2][2].revealed).toBe(false);
    // The cells at the boundary of the mine have adjacentMines=1, so flood
    // stops after revealing them. But the mine cell itself should not be revealed.
  });

  it('does not reveal flagged cells', () => {
    const board = createEmptyBoard();
    // Flag a cell that would otherwise be revealed by flood fill
    board[0][1].flagged = true;
    const result = revealEmpty(board, 0, 0);
    // The flagged cell should remain unrevealed
    expect(result[0][1].revealed).toBe(false);
    expect(result[0][1].flagged).toBe(true);
    // But the clicked cell should be revealed
    expect(result[0][0].revealed).toBe(true);
  });

  it('reveals a mine cell directly (no flood fill)', () => {
    const board = createEmptyBoard();
    board[4][4].mine = true;
    board[4][4].adjacentMines = 0; // mine cells skip count calc
    const result = revealEmpty(board, 4, 4);
    expect(result[4][4].revealed).toBe(true);
    // Flood fill should NOT continue from a mine cell even with adjacentMines = 0
    // because the flood check is `adjacentMines === 0 && !mine`
    let otherRevealed = false;
    for (let r = 0; r < ROWS; r++) {
      for (let c = 0; c < COLS; c++) {
        if (r === 4 && c === 4) continue;
        if (result[r][c].revealed) otherRevealed = true;
      }
    }
    expect(otherRevealed).toBe(false);
  });

  it('does not mutate the original board', () => {
    const board = createEmptyBoard();
    const original = board.map(r => r.map(c => ({ ...c })));
    revealEmpty(board, 4, 4);
    expect(board).toEqual(original);
  });
});

describe('checkWin', () => {
  it('returns true when all non-mine cells are revealed', () => {
    const board = createEmptyBoard();
    // Reveal every cell (no mines on the board)
    for (let r = 0; r < ROWS; r++) {
      for (let c = 0; c < COLS; c++) {
        board[r][c].revealed = true;
      }
    }
    expect(checkWin(board)).toBe(true);
  });

  it('returns true when only mine cells remain unrevealed', () => {
    const board = createEmptyBoard();
    // Place some mines
    board[0][0].mine = true;
    board[1][1].mine = true;
    // Reveal all non-mine cells
    for (let r = 0; r < ROWS; r++) {
      for (let c = 0; c < COLS; c++) {
        if (!board[r][c].mine) {
          board[r][c].revealed = true;
        }
      }
    }
    expect(checkWin(board)).toBe(true);
  });

  it('returns false when non-mine cells remain hidden', () => {
    const board = createEmptyBoard();
    // Don't reveal anything
    expect(checkWin(board)).toBe(false);
  });

  it('returns false when some non-mine cells are still unrevealed', () => {
    const board = createEmptyBoard();
    board[0][0].mine = true;
    // Reveal most but not all non-mine cells
    for (let r = 0; r < ROWS; r++) {
      for (let c = 0; c < COLS; c++) {
        if (r === 8 && c === 8) continue; // leave one unrevealed
        if (!board[r][c].mine) {
          board[r][c].revealed = true;
        }
      }
    }
    expect(checkWin(board)).toBe(false);
  });
});
