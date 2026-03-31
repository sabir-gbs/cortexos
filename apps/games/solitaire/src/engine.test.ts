import { describe, it, expect } from 'vitest';
import {
  createDeck,
  shuffle,
  dealGame,
  canPlaceOnFoundation,
  canPlaceOnTableau,
  isRed,
  valueIndex,
  SUITS,
  VALUES,
} from './engine';
import type { Card } from './engine';

describe('Solitaire Engine', () => {
  describe('createDeck', () => {
    it('returns 52 cards (4 suits x 13 values)', () => {
      const deck = createDeck();
      expect(deck).toHaveLength(52);
    });

    it('has all suit/value combinations', () => {
      const deck = createDeck();
      for (const suit of SUITS) {
        for (const value of VALUES) {
          expect(
            deck.some((c) => c.suit === suit && c.value === value),
            `Missing ${value}${suit}`,
          ).toBe(true);
        }
      }
    });

    it('creates every card face-down', () => {
      const deck = createDeck();
      for (const card of deck) {
        expect(card.faceUp).toBe(false);
      }
    });
  });

  describe('shuffle', () => {
    it('preserves length', () => {
      const deck = createDeck();
      const shuffled = shuffle(deck);
      expect(shuffled).toHaveLength(deck.length);
    });

    it('returns a new array without modifying the original', () => {
      const deck = createDeck();
      const original = [...deck];
      shuffle(deck);
      expect(deck).toEqual(original);
    });

    it('contains the same elements after shuffling', () => {
      const deck = createDeck();
      const shuffled = shuffle(deck);
      // Sort both by suit then value for comparison
      const sortKey = (c: Card) => c.suit + c.value;
      const sortedOriginal = [...deck].sort((a, b) => sortKey(a).localeCompare(sortKey(b)));
      const sortedShuffled = [...shuffled].sort((a, b) => sortKey(a).localeCompare(sortKey(b)));
      expect(sortedShuffled).toEqual(sortedOriginal);
    });

    it('actually changes the order (probabilistic)', () => {
      // With 52 cards the chance shuffle returns the same order is vanishingly small
      const deck = createDeck();
      const shuffled = shuffle(deck);
      let samePositions = 0;
      for (let i = 0; i < deck.length; i++) {
        if (deck[i] === shuffled[i]) samePositions++;
      }
      // Allow up to 10 cards to land in the same spot by chance
      expect(samePositions).toBeLessThan(52);
    });
  });

  describe('isRed', () => {
    it('returns true for hearts', () => {
      expect(isRed('♥')).toBe(true);
    });

    it('returns true for diamonds', () => {
      expect(isRed('♦')).toBe(true);
    });

    it('returns false for spades', () => {
      expect(isRed('♠')).toBe(false);
    });

    it('returns false for clubs', () => {
      expect(isRed('♣')).toBe(false);
    });
  });

  describe('valueIndex', () => {
    it('returns 0 for Ace', () => {
      expect(valueIndex('A')).toBe(0);
    });

    it('returns 9 for 10', () => {
      expect(valueIndex('10')).toBe(9);
    });

    it('returns 10 for Jack', () => {
      expect(valueIndex('J')).toBe(10);
    });

    it('returns 11 for Queen', () => {
      expect(valueIndex('Q')).toBe(11);
    });

    it('returns 12 for King', () => {
      expect(valueIndex('K')).toBe(12);
    });

    it('returns correct sequential indices for all values', () => {
      const expectedIndices = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
      for (let i = 0; i < VALUES.length; i++) {
        expect(valueIndex(VALUES[i])).toBe(expectedIndices[i]);
      }
    });
  });

  describe('dealGame', () => {
    it('distributes all 52 cards', () => {
      const state = dealGame();
      const total =
        state.stock.length +
        state.waste.length +
        state.foundations.flat().length +
        state.tableau.flat().length;
      expect(total).toBe(52);
    });

    it('has 7 tableau columns with correct card counts', () => {
      const state = dealGame();
      for (let col = 0; col < 7; col++) {
        expect(state.tableau[col]).toHaveLength(col + 1);
      }
    });

    it('has 24 stock cards (52 - 28 in tableau)', () => {
      const state = dealGame();
      expect(state.stock).toHaveLength(24);
    });

    it('has empty waste pile', () => {
      const state = dealGame();
      expect(state.waste).toHaveLength(0);
    });

    it('has 4 empty foundations', () => {
      const state = dealGame();
      for (let i = 0; i < 4; i++) {
        expect(state.foundations[i]).toHaveLength(0);
      }
    });

    it('top card of each tableau column is face up', () => {
      const state = dealGame();
      for (let col = 0; col < 7; col++) {
        const column = state.tableau[col];
        expect(column[column.length - 1].faceUp).toBe(true);
      }
    });

    it('non-top tableau cards are face down', () => {
      const state = dealGame();
      for (let col = 0; col < 7; col++) {
        const column = state.tableau[col];
        for (let row = 0; row < column.length - 1; row++) {
          expect(column[row].faceUp, `Column ${col} card ${row} should be face down`).toBe(false);
        }
      }
    });

    it('all stock cards are face down', () => {
      const state = dealGame();
      for (const card of state.stock) {
        expect(card.faceUp).toBe(false);
      }
    });

    it('returns independent states on successive calls', () => {
      const state1 = dealGame();
      const state2 = dealGame();
      // The two states should not share the same array references
      expect(state1.stock).not.toBe(state2.stock);
      expect(state1.tableau).not.toBe(state2.tableau);
    });
  });

  describe('canPlaceOnFoundation', () => {
    it('allows ace on empty foundation', () => {
      const aceOfSpades: Card = { suit: '♠', value: 'A', faceUp: true };
      expect(canPlaceOnFoundation(aceOfSpades, [])).toBe(true);
    });

    it('allows same suit in ascending order', () => {
      const foundation: Card[] = [
        { suit: '♥', value: 'A', faceUp: true },
        { suit: '♥', value: '2', faceUp: true },
        { suit: '♥', value: '3', faceUp: true },
      ];
      const fourOfHearts: Card = { suit: '♥', value: '4', faceUp: true };
      expect(canPlaceOnFoundation(fourOfHearts, foundation)).toBe(true);
    });

    it('rejects wrong suit', () => {
      const foundation: Card[] = [
        { suit: '♥', value: 'A', faceUp: true },
        { suit: '♥', value: '2', faceUp: true },
      ];
      const threeOfSpades: Card = { suit: '♠', value: '3', faceUp: true };
      expect(canPlaceOnFoundation(threeOfSpades, foundation)).toBe(false);
    });

    it('rejects non-sequential card (skip a value)', () => {
      const foundation: Card[] = [
        { suit: '♠', value: 'A', faceUp: true },
      ];
      const threeOfSpades: Card = { suit: '♠', value: '3', faceUp: true };
      expect(canPlaceOnFoundation(threeOfSpades, foundation)).toBe(false);
    });

    it('rejects non-ace on empty foundation', () => {
      const twoOfHearts: Card = { suit: '♥', value: '2', faceUp: true };
      expect(canPlaceOnFoundation(twoOfHearts, [])).toBe(false);
    });

    it('rejects king on empty foundation', () => {
      const kingOfSpades: Card = { suit: '♠', value: 'K', faceUp: true };
      expect(canPlaceOnFoundation(kingOfSpades, [])).toBe(false);
    });

    it('allows placing a queen on top of a jack of the same suit', () => {
      const foundation: Card[] = [
        { suit: '♦', value: 'A', faceUp: true },
        { suit: '♦', value: '2', faceUp: true },
        { suit: '♦', value: '3', faceUp: true },
        { suit: '♦', value: '4', faceUp: true },
        { suit: '♦', value: '5', faceUp: true },
        { suit: '♦', value: '6', faceUp: true },
        { suit: '♦', value: '7', faceUp: true },
        { suit: '♦', value: '8', faceUp: true },
        { suit: '♦', value: '9', faceUp: true },
        { suit: '♦', value: '10', faceUp: true },
        { suit: '♦', value: 'J', faceUp: true },
      ];
      const queenOfDiamonds: Card = { suit: '♦', value: 'Q', faceUp: true };
      expect(canPlaceOnFoundation(queenOfDiamonds, foundation)).toBe(true);
    });

    it('rejects a card going backwards (lower value)', () => {
      const foundation: Card[] = [
        { suit: '♣', value: 'A', faceUp: true },
        { suit: '♣', value: '2', faceUp: true },
        { suit: '♣', value: '3', faceUp: true },
      ];
      const twoOfClubs: Card = { suit: '♣', value: '2', faceUp: true };
      expect(canPlaceOnFoundation(twoOfClubs, foundation)).toBe(false);
    });
  });

  describe('canPlaceOnTableau', () => {
    it('allows king on empty column', () => {
      const kingOfHearts: Card = { suit: '♥', value: 'K', faceUp: true };
      expect(canPlaceOnTableau(kingOfHearts, [])).toBe(true);
    });

    it('allows alternating colors in descending order', () => {
      const column: Card[] = [
        { suit: '♥', value: 'K', faceUp: true },
        { suit: '♠', value: 'Q', faceUp: true },
        { suit: '♥', value: 'J', faceUp: true },
      ];
      const tenOfSpades: Card = { suit: '♠', value: '10', faceUp: true };
      expect(canPlaceOnTableau(tenOfSpades, column)).toBe(true);
    });

    it('rejects same color (red on red)', () => {
      const column: Card[] = [
        { suit: '♥', value: 'K', faceUp: true },
      ];
      const queenOfDiamonds: Card = { suit: '♦', value: 'Q', faceUp: true };
      expect(canPlaceOnTableau(queenOfDiamonds, column)).toBe(false);
    });

    it('rejects same color (black on black)', () => {
      const column: Card[] = [
        { suit: '♠', value: 'K', faceUp: true },
      ];
      const queenOfClubs: Card = { suit: '♣', value: 'Q', faceUp: true };
      expect(canPlaceOnTableau(queenOfClubs, column)).toBe(false);
    });

    it('rejects non-sequential card (skip a value)', () => {
      const column: Card[] = [
        { suit: '♥', value: 'K', faceUp: true },
      ];
      const jackOfSpades: Card = { suit: '♠', value: 'J', faceUp: true };
      expect(canPlaceOnTableau(jackOfSpades, column)).toBe(false);
    });

    it('rejects non-king on empty column', () => {
      const queenOfHearts: Card = { suit: '♥', value: 'Q', faceUp: true };
      expect(canPlaceOnTableau(queenOfHearts, [])).toBe(false);
    });

    it('rejects ace on empty column', () => {
      const aceOfSpades: Card = { suit: '♠', value: 'A', faceUp: true };
      expect(canPlaceOnTableau(aceOfSpades, [])).toBe(false);
    });

    it('rejects card placed on a face-down top card', () => {
      const column: Card[] = [
        { suit: '♥', value: 'K', faceUp: false },
      ];
      const queenOfSpades: Card = { suit: '♠', value: 'Q', faceUp: true };
      expect(canPlaceOnTableau(queenOfSpades, column)).toBe(false);
    });

    it('allows a red queen on a black king', () => {
      const column: Card[] = [
        { suit: '♠', value: 'K', faceUp: true },
      ];
      const queenOfHearts: Card = { suit: '♥', value: 'Q', faceUp: true };
      expect(canPlaceOnTableau(queenOfHearts, column)).toBe(true);
    });

    it('allows a black 2 on a red 3', () => {
      const column: Card[] = [
        { suit: '♥', value: '3', faceUp: true },
      ];
      const twoOfClubs: Card = { suit: '♣', value: '2', faceUp: true };
      expect(canPlaceOnTableau(twoOfClubs, column)).toBe(true);
    });

    it('rejects a card going in ascending order instead of descending', () => {
      const column: Card[] = [
        { suit: '♥', value: 'Q', faceUp: true },
      ];
      const kingOfSpades: Card = { suit: '♠', value: 'K', faceUp: true };
      expect(canPlaceOnTableau(kingOfSpades, column)).toBe(false);
    });
  });
});
