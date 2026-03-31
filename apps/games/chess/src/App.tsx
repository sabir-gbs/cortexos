import React, { useState, useCallback } from 'react';
import './App.css';
import {
  createInitialBoard,
  getValidMoves,
  type Board,
  type Color,
  type Piece,
  type Position,
  type PieceType,
} from './engine';

const PIECE_UNICODE: Record<Color, Record<PieceType, string>> = {
  white: { K: '\u2654', Q: '\u2655', R: '\u2656', B: '\u2657', N: '\u2658', P: '\u2659' },
  black: { K: '\u265A', Q: '\u265B', R: '\u265C', B: '\u265D', N: '\u265E', P: '\u265F' },
};

const App: React.FC = () => {
  const [board, setBoard] = useState<Board>(createInitialBoard);
  const [selected, setSelected] = useState<Position | null>(null);
  const [validMoves, setValidMoves] = useState<Position[]>([]);
  const [currentTurn, setCurrentTurn] = useState<Color>('white');
  const [lastMove, setLastMove] = useState<[Position, Position] | null>(null);

  const newGame = useCallback(() => {
    setBoard(createInitialBoard());
    setSelected(null);
    setValidMoves([]);
    setCurrentTurn('white');
    setLastMove(null);
  }, []);

  const handleCellClick = useCallback((row: number, col: number) => {
    const clickedPiece = board[row][col];

    if (selected) {
      const isValid = validMoves.some(m => m.row === row && m.col === col);

      if (isValid) {
        setBoard(prev => {
          const next = prev.map(r => [...r]);
          next[row][col] = next[selected.row][selected.col];
          next[selected.row][selected.col] = null;
          return next;
        });
        setLastMove([selected, { row, col }]);
        setCurrentTurn(t => t === 'white' ? 'black' : 'white');
        setSelected(null);
        setValidMoves([]);
        return;
      }

      if (clickedPiece && clickedPiece.color === currentTurn) {
        const pos = { row, col };
        setSelected(pos);
        setValidMoves(getValidMoves(board, pos));
        return;
      }

      setSelected(null);
      setValidMoves([]);
      return;
    }

    if (clickedPiece && clickedPiece.color === currentTurn) {
      const pos = { row, col };
      setSelected(pos);
      setValidMoves(getValidMoves(board, pos));
    }
  }, [selected, validMoves, board, currentTurn]);

  const isLightSquare = (row: number, col: number): boolean =>
    (row + col) % 2 === 0;

  const isSelected = (row: number, col: number): boolean =>
    selected !== null && selected.row === row && selected.col === col;

  const isValidMove = (row: number, col: number): boolean =>
    validMoves.some(m => m.row === row && m.col === col);

  const isLastMove = (row: number, col: number): boolean => {
    if (!lastMove) return false;
    return (lastMove[0].row === row && lastMove[0].col === col) ||
           (lastMove[1].row === row && lastMove[1].col === col);
  };

  const isCapture = (row: number, col: number): boolean =>
    isValidMove(row, col) && board[row][col] !== null;

  const getCellClass = (row: number, col: number): string => {
    let cls = 'cell';
    cls += isLightSquare(row, col) ? ' light' : ' dark';
    if (isSelected(row, col)) cls += ' selected';
    if (isValidMove(row, col)) cls += isCapture(row, col) ? ' valid-capture' : ' valid-move';
    if (isLastMove(row, col) && !isSelected(row, col) && !isValidMove(row, col)) cls += ' last-move';
    return cls;
  };

  const getPieceChar = (piece: Piece): string =>
    PIECE_UNICODE[piece.color][piece.type];

  return (
    <div className="app">
      <div className="header">
        <h1>Chess</h1>
        <span className={`turn-indicator ${currentTurn}-turn`}>
          {currentTurn === 'white' ? 'White' : 'Black'} to move
        </span>
        <button className="new-game-btn" onClick={newGame}>New Game</button>
      </div>

      <div className="board">
        {board.map((row, r) =>
          row.map((piece, c) => (
            <div
              key={`${r}-${c}`}
              className={getCellClass(r, c)}
              onClick={() => handleCellClick(r, c)}
            >
              {piece && (
                <span className={`piece-${piece.color}`}>
                  {getPieceChar(piece)}
                </span>
              )}
            </div>
          ))
        )}
      </div>

      <div className="footer">Click a piece to select, then click a destination to move</div>
    </div>
  );
};

export default App;
