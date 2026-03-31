import React, { useState, useCallback } from 'react';
import './App.css';
import {
  isRed,
  dealGame,
  canPlaceOnFoundation,
  canPlaceOnTableau,
} from './engine';
import type { Card, GameState } from './engine';

interface Selection {
  source: 'tableau' | 'waste' | 'foundation';
  col: number;
  cardIndex: number;
}

const App: React.FC = () => {
  const [game, setGame] = useState<GameState>(dealGame);
  const [selection, setSelection] = useState<Selection | null>(null);
  const [moves, setMoves] = useState(0);

  const flipTopCard = useCallback((tableau: Card[][], col: number) => {
    const column = [...tableau[col]];
    if (column.length > 0 && !column[column.length - 1].faceUp) {
      column[column.length - 1] = { ...column[column.length - 1], faceUp: true };
      tableau[col] = column;
    }
  }, []);

  const newGame = useCallback(() => {
    setGame(dealGame());
    setSelection(null);
    setMoves(0);
  }, []);

  const drawFromStock = useCallback(() => {
    setGame(prev => {
      const next = {
        stock: [...prev.stock],
        waste: [...prev.waste],
        foundations: prev.foundations.map(f => [...f]) as GameState['foundations'],
        tableau: prev.tableau.map(t => [...t]) as GameState['tableau'],
      };
      if (next.stock.length === 0) {
        next.stock = next.waste.reverse().map(c => ({ ...c, faceUp: false }));
        next.waste = [];
      } else {
        const card = next.stock.pop()!;
        card.faceUp = true;
        next.waste.push(card);
      }
      return next;
    });
    setSelection(null);
  }, []);

  const handleCardClick = useCallback((source: 'tableau' | 'waste' | 'foundation', col: number, cardIndex: number) => {
    setGame(prev => {
      const next = {
        stock: [...prev.stock],
        waste: [...prev.waste],
        foundations: prev.foundations.map(f => [...f]) as GameState['foundations'],
        tableau: prev.tableau.map(t => t.map(c => ({ ...c }))) as GameState['tableau'],
      };

      if (selection) {
        const sel = selection;
        let movingCards: Card[] = [];

        if (sel.source === 'waste') {
          movingCards = [next.waste[next.waste.length - 1]];
        } else if (sel.source === 'foundation') {
          movingCards = [next.foundations[sel.col][next.foundations[sel.col].length - 1]];
        } else {
          movingCards = next.tableau[sel.col].slice(sel.cardIndex);
        }

        if (source === 'foundation') {
          if (movingCards.length === 1 && canPlaceOnFoundation(movingCards[0], next.foundations[col])) {
            if (sel.source === 'waste') {
              next.waste.pop();
            } else if (sel.source === 'foundation') {
              next.foundations[sel.col].pop();
            } else {
              next.tableau[sel.col] = next.tableau[sel.col].slice(0, sel.cardIndex);
              flipTopCard(next.tableau as Card[][], sel.col);
            }
            next.foundations[col].push(movingCards[0]);
            setMoves(m => m + 1);
            setSelection(null);
            return next;
          }
        } else if (source === 'tableau') {
          if (canPlaceOnTableau(movingCards[0], next.tableau[col])) {
            if (sel.source === 'waste') {
              next.waste.pop();
            } else if (sel.source === 'foundation') {
              next.foundations[sel.col].pop();
            } else {
              next.tableau[sel.col] = next.tableau[sel.col].slice(0, sel.cardIndex);
              flipTopCard(next.tableau as Card[][], sel.col);
            }
            next.tableau[col] = [...next.tableau[col], ...movingCards];
            setMoves(m => m + 1);
            setSelection(null);
            return next;
          }
        }

        if (source === sel.source && col === sel.col && cardIndex === sel.cardIndex) {
          setSelection(null);
          return prev;
        }
      }

      let clickedCard: Card | null = null;
      if (source === 'tableau') {
        const column = next.tableau[col];
        if (cardIndex < column.length && column[cardIndex].faceUp) {
          clickedCard = column[cardIndex];
        }
      } else if (source === 'waste' && next.waste.length > 0) {
        clickedCard = next.waste[next.waste.length - 1];
      } else if (source === 'foundation' && next.foundations[col].length > 0) {
        clickedCard = next.foundations[col][next.foundations[col].length - 1];
      }

      if (clickedCard) {
        setSelection({ source, col, cardIndex });
      } else {
        setSelection(null);
      }

      return prev;
    });
  }, [selection, flipTopCard]);

  const isSelected = (source: string, col: number, cardIndex: number): boolean => {
    if (!selection) return false;
    if (source === 'tableau' && selection.source === 'tableau' && selection.col === col) {
      return cardIndex >= selection.cardIndex;
    }
    return selection.source === source && selection.col === col && selection.cardIndex === cardIndex;
  };

  const renderCard = (card: Card, source: string, col: number, cardIndex: number, topOffset: number) => {
    const selected = isSelected(source, col, cardIndex);
    if (!card.faceUp) {
      return (
        <div
          key={cardIndex}
          className="card face-down"
          style={{ top: topOffset, zIndex: cardIndex }}
        />
      );
    }
    return (
      <div
        key={cardIndex}
        className={`card face-up ${isRed(card.suit) ? 'red' : 'black'} ${selected ? 'selected' : ''}`}
        style={{ top: topOffset, zIndex: cardIndex }}
        onClick={() => handleCardClick(source as 'tableau' | 'waste' | 'foundation', col, cardIndex)}
      >
        <div className="card-corner card-corner-top">
          <span className="card-value">{card.value}</span>
          <span className="card-suit">{card.suit}</span>
        </div>
        <div className="card-corner card-corner-bottom">
          <span className="card-value">{card.value}</span>
          <span className="card-suit">{card.suit}</span>
        </div>
      </div>
    );
  };

  const foundationSymbols = ['♠', '♥', '♦', '♣'];

  return (
    <div className="app">
      <div className="header">
        <h1>Solitaire</h1>
        <button className="new-game-btn" onClick={newGame}>New Game</button>
      </div>

      <div className="top-row">
        <div className="stock-area">
          {game.stock.length > 0 ? (
            <div className="stock-pile" onClick={drawFromStock}>
              <div className="card face-down" style={{ position: 'relative', top: 0 }} />
              {game.stock.length > 1 && (
                <div className="card face-down" style={{ position: 'absolute', top: -2, left: 1, zIndex: -1 }} />
              )}
            </div>
          ) : (
            <div className="stock-empty" onClick={drawFromStock}>
              ↺
            </div>
          )}
          <div className="waste-pile">
            {game.waste.length > 0 && (() => {
              const card = game.waste[game.waste.length - 1];
              return renderCard(card, 'waste', 0, game.waste.length - 1, 0);
            })()}
          </div>
        </div>

        <div className="foundations">
          {game.foundations.map((foundation, i) => (
            <div
              key={i}
              className="pile-placeholder"
              onClick={() => handleCardClick('foundation', i, foundation.length - 1)}
            >
              {foundation.length > 0 ? (
                <div style={{ position: 'relative', width: '100%', height: '100%' }}>
                  {renderCard(foundation[foundation.length - 1], 'foundation', i, foundation.length - 1, 0)}
                </div>
              ) : (
                foundationSymbols[i]
              )}
            </div>
          ))}
        </div>
      </div>

      <div className="tableau">
        {game.tableau.map((column, col) => (
          <div
            key={col}
            className="tableau-column"
            style={{ height: column.length > 0 ? Math.max(100, column.length * 22 + 80) : 100 }}
            onClick={(e) => {
              if (e.target === e.currentTarget && column.length === 0) {
                handleCardClick('tableau', col, 0);
              }
            }}
          >
            {column.length === 0 && (
              <div className="pile-placeholder" />
            )}
            {column.map((card, ci) =>
              renderCard(card, 'tableau', col, ci, ci * 22)
            )}
          </div>
        ))}
      </div>

      <div className="status-bar">
        Moves: {moves}
      </div>
    </div>
  );
};

export default App;
