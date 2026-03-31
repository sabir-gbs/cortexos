export type PieceType = 'K' | 'Q' | 'R' | 'B' | 'N' | 'P';
export type Color = 'white' | 'black';

export interface Piece {
  type: PieceType;
  color: Color;
}

export type Board = (Piece | null)[][];

export interface Position {
  row: number;
  col: number;
}

export function createInitialBoard(): Board {
  const board: Board = Array.from({ length: 8 }, () => Array(8).fill(null));

  const backRow: PieceType[] = ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'];
  for (let c = 0; c < 8; c++) {
    board[0][c] = { type: backRow[c], color: 'black' };
    board[1][c] = { type: 'P', color: 'black' };
    board[6][c] = { type: 'P', color: 'white' };
    board[7][c] = { type: backRow[c], color: 'white' };
  }

  return board;
}

export function inBounds(r: number, c: number): boolean {
  return r >= 0 && r < 8 && c >= 0 && c < 8;
}

export function getValidMoves(board: Board, pos: Position): Position[] {
  const piece = board[pos.row][pos.col];
  if (!piece) return [];

  const moves: Position[] = [];
  const { row, col } = pos;
  const color = piece.color;
  const enemy = color === 'white' ? 'black' : 'white';

  const addIfValid = (r: number, c: number) => {
    if (!inBounds(r, c)) return false;
    const target = board[r][c];
    if (target && target.color === color) return false;
    moves.push({ row: r, col: c });
    return !target;
  };

  const addSlidingMoves = (directions: [number, number][]) => {
    for (const [dr, dc] of directions) {
      for (let i = 1; i < 8; i++) {
        const nr = row + dr * i;
        const nc = col + dc * i;
        if (!inBounds(nr, nc)) break;
        const target = board[nr][nc];
        if (target) {
          if (target.color === enemy) {
            moves.push({ row: nr, col: nc });
          }
          break;
        }
        moves.push({ row: nr, col: nc });
      }
    }
  };

  switch (piece.type) {
    case 'P': {
      const dir = color === 'white' ? -1 : 1;
      const startRow = color === 'white' ? 6 : 1;

      if (inBounds(row + dir, col) && !board[row + dir][col]) {
        moves.push({ row: row + dir, col });
        if (row === startRow && !board[row + dir * 2][col]) {
          moves.push({ row: row + dir * 2, col });
        }
      }

      for (const dc of [-1, 1]) {
        const nr = row + dir;
        const nc = col + dc;
        if (inBounds(nr, nc) && board[nr][nc] && board[nr][nc]!.color === enemy) {
          moves.push({ row: nr, col: nc });
        }
      }
      break;
    }
    case 'R':
      addSlidingMoves([[0, 1], [0, -1], [1, 0], [-1, 0]]);
      break;
    case 'B':
      addSlidingMoves([[1, 1], [1, -1], [-1, 1], [-1, -1]]);
      break;
    case 'Q':
      addSlidingMoves([[0, 1], [0, -1], [1, 0], [-1, 0], [1, 1], [1, -1], [-1, 1], [-1, -1]]);
      break;
    case 'N': {
      const knightMoves: [number, number][] = [
        [-2, -1], [-2, 1], [-1, -2], [-1, 2],
        [1, -2], [1, 2], [2, -1], [2, 1],
      ];
      for (const [dr, dc] of knightMoves) {
        addIfValid(row + dr, col + dc);
      }
      break;
    }
    case 'K': {
      for (let dr = -1; dr <= 1; dr++) {
        for (let dc = -1; dc <= 1; dc++) {
          if (dr === 0 && dc === 0) continue;
          addIfValid(row + dr, col + dc);
        }
      }
      break;
    }
  }

  return moves;
}
