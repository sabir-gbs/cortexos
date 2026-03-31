import React, { useState, useCallback } from 'react';
import './App.css';
import {
  ROWS,
  COLS,
  MINE_COUNT,
  createEmptyBoard,
  placeMines,
  revealEmpty,
  checkWin,
  type Board,
  type Cell,
} from './engine';

type GameStatus = 'playing' | 'won' | 'lost';

const App: React.FC = () => {
  const [board, setBoard] = useState<Board>(createEmptyBoard);
  const [status, setStatus] = useState<GameStatus>('playing');
  const [minesInitialized, setMinesInitialized] = useState(false);
  const [flagsPlaced, setFlagsPlaced] = useState(0);

  const resetGame = useCallback(() => {
    setBoard(createEmptyBoard());
    setStatus('playing');
    setMinesInitialized(false);
    setFlagsPlaced(0);
  }, []);

  const handleLeftClick = useCallback((row: number, col: number) => {
    if (status !== 'playing') return;
    setBoard(prev => {
      const cell = prev[row][col];
      if (cell.revealed || cell.flagged) return prev;

      let next = prev.map(r => r.map(c => ({ ...c })));

      if (!minesInitialized) {
        next = placeMines(next, row, col);
        setMinesInitialized(true);
      }

      if (next[row][col].mine) {
        next[row][col].revealed = true;
        for (let r = 0; r < ROWS; r++) {
          for (let c = 0; c < COLS; c++) {
            if (next[r][c].mine) {
              next[r][c].revealed = true;
            }
          }
        }
        setStatus('lost');
        return next;
      }

      next = revealEmpty(next, row, col);

      if (checkWin(next)) {
        setStatus('won');
      }
      return next;
    });
  }, [status, minesInitialized]);

  const handleRightClick = useCallback((e: React.MouseEvent, row: number, col: number) => {
    e.preventDefault();
    if (status !== 'playing') return;
    setBoard(prev => {
      const next = prev.map(r => r.map(c => ({ ...c })));
      const cell = next[row][col];
      if (cell.revealed) return prev;

      if (cell.flagged) {
        cell.flagged = false;
        setFlagsPlaced(f => f - 1);
      } else {
        cell.flagged = true;
        setFlagsPlaced(f => f + 1);
      }
      return next;
    });
  }, [status]);

  const getCellContent = (cell: Cell): string => {
    if (cell.flagged && !cell.revealed) return '\u{1F6A9}';
    if (!cell.revealed) return '';
    if (cell.mine) return '\u{1F4A3}';
    if (cell.adjacentMines === 0) return '';
    return String(cell.adjacentMines);
  };

  const getCellClass = (cell: Cell, row: number, col: number): string => {
    let cls = 'cell';
    if (cell.revealed) {
      cls += cell.mine ? (status === 'lost' && cell.mine ? ' mine-shown' : ' mine-hit') : ' revealed';
      if (!cell.mine && cell.adjacentMines > 0) {
        cls += ` cell-${cell.adjacentMines}`;
      }
    } else if (cell.flagged) {
      cls += ' flagged';
    } else {
      cls += ' hidden';
    }
    return cls;
  };

  return (
    <div className="app">
      <div className="header">
        <h1>Minesweeper</h1>
        <span className="mine-count">
          {'\u{1F4A3}'} {MINE_COUNT - flagsPlaced}
        </span>
        <span className={`status-text ${status === 'won' ? 'won' : status === 'lost' ? 'lost' : ''}`}>
          {status === 'won' ? 'You Win!' : status === 'lost' ? 'Game Over' : ''}
        </span>
        <button className="reset-btn" onClick={resetGame}>Reset</button>
      </div>

      <div className="board">
        {board.map((row, r) =>
          row.map((cell, c) => (
            <div
              key={`${r}-${c}`}
              className={getCellClass(cell, r, c)}
              onClick={() => handleLeftClick(r, c)}
              onContextMenu={(e) => handleRightClick(e, r, c)}
            >
              {getCellContent(cell)}
            </div>
          ))
        )}
      </div>

      <div className="footer">Left-click to reveal, right-click to flag</div>
    </div>
  );
};

export default App;
