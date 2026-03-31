import { describe, it, expect } from 'vitest';
import {
  BOARD_WIDTH,
  BOARD_HEIGHT,
  PIECE_SHAPES,
  PIECE_TYPES,
  PieceType,
  Piece,
  Board,
  createEmptyBoard,
  isValidPosition,
  placePiece,
  clearLines,
  getLevel,
  getSpeed,
  getShape,
  randomPiece,
} from './engine';

describe('Tetris Engine', () => {
  describe('PIECE_SHAPES', () => {
    it('defines all 7 piece types', () => {
      const expectedTypes: PieceType[] = ['I', 'O', 'T', 'S', 'Z', 'J', 'L'];
      for (const type of expectedTypes) {
        expect(PIECE_SHAPES[type]).toBeDefined();
      }
    });

    it('has exactly 4 rotations for each piece type', () => {
      for (const type of PIECE_TYPES) {
        expect(PIECE_SHAPES[type]).toHaveLength(4);
      }
    });

    it('has square matrices (rows equal columns) for each rotation', () => {
      for (const type of PIECE_TYPES) {
        for (let r = 0; r < 4; r++) {
          const shape = PIECE_SHAPES[type][r];
          const cols = shape[0].length;
          for (const row of shape) {
            expect(row).toHaveLength(cols);
          }
        }
      }
    });

    it('contains only 0s and 1s in every shape', () => {
      for (const type of PIECE_TYPES) {
        for (let r = 0; r < 4; r++) {
          for (const row of PIECE_SHAPES[type][r]) {
            for (const cell of row) {
              expect(cell === 0 || cell === 1).toBe(true);
            }
          }
        }
      }
    });

    it('has at least one filled cell in every rotation', () => {
      for (const type of PIECE_TYPES) {
        for (let r = 0; r < 4; r++) {
          const filledCount = PIECE_SHAPES[type][r]
            .flat()
            .filter(c => c === 1).length;
          expect(filledCount).toBeGreaterThan(0);
        }
      }
    });

    it('O piece has the same shape in all 4 rotations', () => {
      const base = JSON.stringify(PIECE_SHAPES['O'][0]);
      for (let r = 1; r < 4; r++) {
        expect(JSON.stringify(PIECE_SHAPES['O'][r])).toBe(base);
      }
    });

    it('I piece has different shapes in different rotations', () => {
      const shapes = PIECE_SHAPES['I'].map(s => JSON.stringify(s));
      const uniqueShapes = new Set(shapes);
      expect(uniqueShapes.size).toBeGreaterThan(1);
    });

    it('every piece type has 4 filled cells except O which has 2x2=4', () => {
      // Standard tetrominoes have exactly 4 filled cells
      for (const type of PIECE_TYPES) {
        const shape = PIECE_SHAPES[type][0];
        const filledCount = shape.flat().filter(c => c === 1).length;
        expect(filledCount).toBe(4);
      }
    });
  });

  describe('createEmptyBoard', () => {
    it('creates a board with correct dimensions', () => {
      const board = createEmptyBoard();
      expect(board).toHaveLength(BOARD_HEIGHT);
      for (const row of board) {
        expect(row).toHaveLength(BOARD_WIDTH);
      }
    });

    it('creates a board with all null cells', () => {
      const board = createEmptyBoard();
      for (const row of board) {
        for (const cell of row) {
          expect(cell).toBeNull();
        }
      }
    });
  });

  describe('isValidPosition', () => {
    it('returns true for a piece at the default spawn position on empty board', () => {
      const board = createEmptyBoard();
      const piece: Piece = {
        type: 'T',
        position: { x: Math.floor(BOARD_WIDTH / 2) - 1, y: 0 },
        rotation: 0,
      };
      expect(isValidPosition(board, piece)).toBe(true);
    });

    it('detects left wall collision', () => {
      const board = createEmptyBoard();
      const piece: Piece = {
        type: 'T',
        position: { x: -1, y: 5 },
        rotation: 0,
      };
      // T shape rotation 0: [[0,1,0],[1,1,1],[0,0,0]]
      // The middle-left cell (row 1, col 0) would be at x=-1 -> collision
      expect(isValidPosition(board, piece)).toBe(false);
    });

    it('detects right wall collision', () => {
      const board = createEmptyBoard();
      const piece: Piece = {
        type: 'T',
        position: { x: BOARD_WIDTH - 1, y: 5 },
        rotation: 0,
      };
      // T shape rotation 0: [[0,1,0],[1,1,1],[0,0,0]]
      // position.x = 9, shape width = 3, so rightmost at x=11 > BOARD_WIDTH=10
      expect(isValidPosition(board, piece)).toBe(false);
    });

    it('detects floor collision', () => {
      const board = createEmptyBoard();
      const piece: Piece = {
        type: 'T',
        position: { x: 3, y: BOARD_HEIGHT - 1 },
        rotation: 0,
      };
      // T shape rotation 0 has 3 rows; at y=19, rows go to y=21 which is past the board
      expect(isValidPosition(board, piece)).toBe(false);
    });

    it('allows pieces above the board (negative y) as valid', () => {
      const board = createEmptyBoard();
      const piece: Piece = {
        type: 'T',
        position: { x: 3, y: -1 },
        rotation: 0,
      };
      // Pieces can be partially above the board; the shape extends to y=1 which is valid
      expect(isValidPosition(board, piece)).toBe(true);
    });

    it('detects collision with placed pieces', () => {
      const board = createEmptyBoard();
      // Place a block at (3, 0)
      board[0][3] = 'T';
      const piece: Piece = {
        type: 'I',
        position: { x: 0, y: 0 },
        rotation: 0,
      };
      // I piece rotation 0: [[0,0,0,0],[1,1,1,1],[0,0,0,0],[0,0,0,0]]
      // Row 1 spans x=0..3, so x=3 at y=1 is not colliding with board[0][3]
      // Let's use a more precise placement
      expect(isValidPosition(board, piece)).toBe(true);

      // Now test with a piece that actually overlaps
      const piece2: Piece = {
        type: 'T',
        position: { x: 2, y: -1 },
        rotation: 0,
      };
      // T rotation 0: row 0 = [0,1,0] -> y=-1+0=-1 (skip), row 1 = [1,1,1] -> y=0, x=2,3,4
      // board[0][3] = 'T', so cell at y=0, x=3 should collide
      expect(isValidPosition(board, piece2)).toBe(false);
    });

    it('returns true for a piece fully within bounds on empty board', () => {
      const board = createEmptyBoard();
      const piece: Piece = {
        type: 'O',
        position: { x: 4, y: 10 },
        rotation: 0,
      };
      expect(isValidPosition(board, piece)).toBe(true);
    });
  });

  describe('placePiece', () => {
    it('places a piece correctly on an empty board', () => {
      const board = createEmptyBoard();
      const piece: Piece = {
        type: 'T',
        position: { x: 3, y: 0 },
        rotation: 0,
      };
      // T rotation 0: [[0,1,0],[1,1,1],[0,0,0]]
      const result = placePiece(board, piece);
      // Row 0: cell at x=4 should be T
      expect(result[0][4]).toBe('T');
      // Row 1: cells at x=3,4,5 should be T
      expect(result[1][3]).toBe('T');
      expect(result[1][4]).toBe('T');
      expect(result[1][5]).toBe('T');
      // Row 0 x=3 should be null (it's a 0 in the shape)
      expect(result[0][3]).toBeNull();
    });

    it('does not mutate the original board', () => {
      const board = createEmptyBoard();
      const piece: Piece = {
        type: 'T',
        position: { x: 3, y: 0 },
        rotation: 0,
      };
      const originalSnapshot = JSON.stringify(board);
      placePiece(board, piece);
      expect(JSON.stringify(board)).toBe(originalSnapshot);
    });

    it('places an I piece correctly', () => {
      const board = createEmptyBoard();
      const piece: Piece = {
        type: 'I',
        position: { x: 3, y: 0 },
        rotation: 0,
      };
      // I rotation 0: [[0,0,0,0],[1,1,1,1],[0,0,0,0],[0,0,0,0]]
      const result = placePiece(board, piece);
      // Row 1: x=3,4,5,6 should be I
      expect(result[1][3]).toBe('I');
      expect(result[1][4]).toBe('I');
      expect(result[1][5]).toBe('I');
      expect(result[1][6]).toBe('I');
      // Other rows should be empty
      expect(result[0][3]).toBeNull();
      expect(result[2][3]).toBeNull();
    });
  });

  describe('clearLines', () => {
    it('returns the same board when no lines are full', () => {
      const board = createEmptyBoard();
      board[19][0] = 'I';
      board[19][5] = 'T';
      const result = clearLines(board);
      expect(result.linesCleared).toBe(0);
      expect(result.board).toEqual(board);
    });

    it('clears a single full row and shifts rows above down', () => {
      const board = createEmptyBoard();
      // Fill the bottom row completely
      for (let x = 0; x < BOARD_WIDTH; x++) {
        board[BOARD_HEIGHT - 1][x] = 'I';
      }
      // Place a piece one row above
      board[BOARD_HEIGHT - 2][0] = 'T';

      const result = clearLines(board);
      expect(result.linesCleared).toBe(1);
      // The T piece should have shifted down by 1
      expect(result.board[BOARD_HEIGHT - 1][0]).toBe('T');
      // Top row should now be empty
      expect(result.board[0].every(cell => cell === null)).toBe(true);
    });

    it('clears multiple full rows', () => {
      const board = createEmptyBoard();
      // Fill bottom 2 rows completely
      for (let x = 0; x < BOARD_WIDTH; x++) {
        board[BOARD_HEIGHT - 1][x] = 'I';
        board[BOARD_HEIGHT - 2][x] = 'T';
      }
      // Place a piece 2 rows above bottom
      board[BOARD_HEIGHT - 3][0] = 'S';

      const result = clearLines(board);
      expect(result.linesCleared).toBe(2);
      // The S piece should have shifted down by 2
      expect(result.board[BOARD_HEIGHT - 1][0]).toBe('S');
      // Top 2 rows should be empty
      expect(result.board[0].every(cell => cell === null)).toBe(true);
      expect(result.board[1].every(cell => cell === null)).toBe(true);
    });

    it('clears all rows when the entire board is full', () => {
      const board = createEmptyBoard();
      for (let y = 0; y < BOARD_HEIGHT; y++) {
        for (let x = 0; x < BOARD_WIDTH; x++) {
          board[y][x] = 'I';
        }
      }
      const result = clearLines(board);
      expect(result.linesCleared).toBe(BOARD_HEIGHT);
      // All rows should be empty now
      for (const row of result.board) {
        for (const cell of row) {
          expect(cell).toBeNull();
        }
      }
    });

    it('preserves the order of remaining rows', () => {
      const board = createEmptyBoard();
      // Row 18: has a T at column 0
      board[18][0] = 'T';
      // Row 19: full
      for (let x = 0; x < BOARD_WIDTH; x++) {
        board[19][x] = 'I';
      }
      const result = clearLines(board);
      expect(result.linesCleared).toBe(1);
      // T should be in the bottom row now
      expect(result.board[BOARD_HEIGHT - 1][0]).toBe('T');
    });
  });

  describe('getLevel', () => {
    it('returns level 1 for score 0', () => {
      expect(getLevel(0)).toBe(1);
    });

    it('returns level 1 for score 499', () => {
      expect(getLevel(499)).toBe(1);
    });

    it('returns level 2 for score 500', () => {
      expect(getLevel(500)).toBe(2);
    });

    it('returns level 3 for score 1000', () => {
      expect(getLevel(1000)).toBe(3);
    });

    it('returns level 5 for score 2000', () => {
      expect(getLevel(2000)).toBe(5);
    });

    it('increases every 500 points', () => {
      for (let i = 0; i < 10; i++) {
        expect(getLevel(i * 500)).toBe(i + 1);
      }
    });
  });

  describe('getSpeed', () => {
    it('returns 800 for level 1', () => {
      expect(getSpeed(1)).toBe(800);
    });

    it('returns 730 for level 2', () => {
      expect(getSpeed(2)).toBe(730);
    });

    it('returns 660 for level 3', () => {
      expect(getSpeed(3)).toBe(660);
    });

    it('decreases speed with each level', () => {
      for (let level = 2; level < 10; level++) {
        expect(getSpeed(level)).toBeLessThan(getSpeed(level - 1));
      }
    });

    it('never goes below 100ms', () => {
      // Even at very high levels
      expect(getSpeed(100)).toBe(100);
      expect(getSpeed(1000)).toBe(100);
    });

    it('reaches minimum of 100 at level 11', () => {
      // 800 - (11-1)*70 = 800 - 700 = 100
      expect(getSpeed(11)).toBe(100);
    });

    it('clamps to 100 for levels beyond 11', () => {
      expect(getSpeed(12)).toBe(100);
      expect(getSpeed(50)).toBe(100);
    });
  });

  describe('getShape', () => {
    it('returns the correct rotation of a piece', () => {
      const piece: Piece = { type: 'T', position: { x: 0, y: 0 }, rotation: 0 };
      expect(getShape(piece)).toEqual(PIECE_SHAPES['T'][0]);
    });

    it('returns rotation 2 correctly', () => {
      const piece: Piece = { type: 'T', position: { x: 0, y: 0 }, rotation: 2 };
      expect(getShape(piece)).toEqual(PIECE_SHAPES['T'][2]);
    });

    it('handles rotation 3 for I piece', () => {
      const piece: Piece = { type: 'I', position: { x: 0, y: 0 }, rotation: 3 };
      const shape = getShape(piece);
      // I rotation 3: [[0,1,0,0],[0,1,0,0],[0,1,0,0],[0,1,0,0]]
      expect(shape).toEqual(PIECE_SHAPES['I'][3]);
    });
  });

  describe('randomPiece', () => {
    it('returns a valid piece type', () => {
      for (let i = 0; i < 50; i++) {
        const piece = randomPiece();
        expect(PIECE_TYPES).toContain(piece.type);
      }
    });

    it('returns a piece with rotation 0', () => {
      for (let i = 0; i < 20; i++) {
        const piece = randomPiece();
        expect(piece.rotation).toBe(0);
      }
    });

    it('spawns the piece near the center top', () => {
      for (let i = 0; i < 20; i++) {
        const piece = randomPiece();
        expect(piece.position.x).toBe(Math.floor(BOARD_WIDTH / 2) - 1);
        expect(piece.position.y).toBe(0);
      }
    });
  });
});
