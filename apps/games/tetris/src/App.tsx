import React, { useState, useEffect, useCallback, useRef } from 'react';
import './App.css';
import {
  BOARD_WIDTH,
  BOARD_HEIGHT,
  PieceType,
  Position,
  Piece,
  Board,
  PIECE_SHAPES,
  PIECE_TYPES,
  randomPiece,
  getShape,
  createEmptyBoard,
  isValidPosition,
  placePiece,
  clearLines,
  getLevel,
  getSpeed,
} from './engine';

const App: React.FC = () => {
  const [board, setBoard] = useState<Board>(createEmptyBoard);
  const [currentPiece, setCurrentPiece] = useState<Piece>(randomPiece);
  const [nextPiece, setNextPiece] = useState<Piece>(randomPiece);
  const [score, setScore] = useState(0);
  const [level, setLevel] = useState(1);
  const [gameOver, setGameOver] = useState(false);

  const boardRef = useRef(board);
  const currentPieceRef = useRef(currentPiece);
  const nextPieceRef = useRef(nextPiece);
  const gameOverRef = useRef(gameOver);
  const scoreRef = useRef(score);

  boardRef.current = board;
  currentPieceRef.current = currentPiece;
  nextPieceRef.current = nextPiece;
  gameOverRef.current = gameOver;
  scoreRef.current = score;

  const spawnNext = useCallback(() => {
    const next = nextPieceRef.current;
    const newNext = randomPiece();
    const spawn = { ...next, position: { x: Math.floor(BOARD_WIDTH / 2) - 1, y: 0 }, rotation: 0 };

    if (!isValidPosition(boardRef.current, spawn)) {
      setGameOver(true);
      return;
    }

    setCurrentPiece(spawn);
    setNextPiece(newNext);
  }, []);

  const lockPiece = useCallback(() => {
    const piece = currentPieceRef.current;
    const currentBoard = boardRef.current;

    let newBoard = placePiece(currentBoard, piece);
    const { board: clearedBoard, linesCleared } = clearLines(newBoard);
    newBoard = clearedBoard;

    const linePoints = [0, 100, 300, 500, 800];
    const points = linePoints[linesCleared] || 0;
    const newScore = scoreRef.current + points;

    setBoard(newBoard);
    setScore(newScore);
    setLevel(getLevel(newScore));

    setCurrentPiece(piece => {
      setCurrentPiece(piece);
      return piece;
    });

    setTimeout(() => spawnNext(), 0);
  }, [spawnNext]);

  const moveDown = useCallback(() => {
    if (gameOverRef.current) return;
    const piece = currentPieceRef.current;
    const moved = { ...piece, position: { ...piece.position, y: piece.position.y + 1 } };

    if (isValidPosition(boardRef.current, moved)) {
      setCurrentPiece(moved);
    } else {
      lockPiece();
    }
  }, [lockPiece]);

  const moveLeft = useCallback(() => {
    if (gameOverRef.current) return;
    const piece = currentPieceRef.current;
    const moved = { ...piece, position: { ...piece.position, x: piece.position.x - 1 } };
    if (isValidPosition(boardRef.current, moved)) {
      setCurrentPiece(moved);
    }
  }, []);

  const moveRight = useCallback(() => {
    if (gameOverRef.current) return;
    const piece = currentPieceRef.current;
    const moved = { ...piece, position: { ...piece.position, x: piece.position.x + 1 } };
    if (isValidPosition(boardRef.current, moved)) {
      setCurrentPiece(moved);
    }
  }, []);

  const rotate = useCallback(() => {
    if (gameOverRef.current) return;
    const piece = currentPieceRef.current;
    const rotated = { ...piece, rotation: (piece.rotation + 1) % 4 };
    if (isValidPosition(boardRef.current, rotated)) {
      setCurrentPiece(rotated);
    }
  }, []);

  const resetGame = useCallback(() => {
    setBoard(createEmptyBoard());
    const p1 = randomPiece();
    const p2 = randomPiece();
    setCurrentPiece(p1);
    setNextPiece(p2);
    setScore(0);
    setLevel(1);
    setGameOver(false);
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(e.key)) {
        e.preventDefault();
      }
      if (gameOverRef.current) return;

      switch (e.key) {
        case 'ArrowLeft': moveLeft(); break;
        case 'ArrowRight': moveRight(); break;
        case 'ArrowUp': rotate(); break;
        case 'ArrowDown': moveDown(); break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [moveLeft, moveRight, rotate, moveDown]);

  useEffect(() => {
    const interval = setInterval(() => {
      if (gameOverRef.current) return;
      moveDown();
    }, getSpeed(level));

    return () => clearInterval(interval);
  }, [level, moveDown]);

  const getBoardWithPiece = (): Board => {
    const display = board.map(row => [...row]);
    const piece = currentPiece;
    const shape = getShape(piece);
    for (let r = 0; r < shape.length; r++) {
      for (let c = 0; c < shape[r].length; c++) {
        if (shape[r][c]) {
          const x = piece.position.x + c;
          const y = piece.position.y + r;
          if (y >= 0 && y < BOARD_HEIGHT && x >= 0 && x < BOARD_WIDTH) {
            display[y][x] = piece.type;
          }
        }
      }
    }
    return display;
  };

  const getNextPieceGrid = (): (PieceType | null)[][] => {
    const grid: (PieceType | null)[][] = Array.from({ length: 4 }, () =>
      Array.from({ length: 4 }, () => null)
    );
    const shape = PIECE_SHAPES[nextPiece.type][0];
    for (let r = 0; r < shape.length; r++) {
      for (let c = 0; c < shape[r].length; c++) {
        if (shape[r][c]) {
          grid[r][c] = nextPiece.type;
        }
      }
    }
    return grid;
  };

  const displayBoard = getBoardWithPiece();

  return (
    <div className="app">
      <div className="header">
        <h1>Tetris</h1>
        <div className="stats">
          <div className="stat">
            <div className="stat-label">Score</div>
            <div className="stat-value">{score}</div>
          </div>
          <div className="stat">
            <div className="stat-label">Level</div>
            <div className="stat-value">{level}</div>
          </div>
        </div>
        {gameOver && <span className="status-text">Game Over</span>}
        <button className="restart-btn" onClick={resetGame}>Restart</button>
      </div>

      <div className="game-area">
        <div className="board">
          {displayBoard.map((row, y) =>
            row.map((cell, x) => (
              <div
                key={`${y}-${x}`}
                className={`cell ${cell ? `filled color-${cell}` : ''}`}
              />
            ))
          )}
        </div>

        <div className="sidebar">
          <div className="next-piece-label">Next</div>
          <div className="next-piece-box">
            {getNextPieceGrid().map((row, y) =>
              row.map((cell, x) => (
                <div
                  key={`next-${y}-${x}`}
                  className={`preview-cell ${cell ? `filled color-${cell}` : ''}`}
                />
              ))
            )}
          </div>

          <div className="controls">
            <strong>Controls:</strong><br />
            Left/Right: Move<br />
            Up: Rotate<br />
            Down: Soft drop
          </div>
        </div>
      </div>
    </div>
  );
};

export default App;
