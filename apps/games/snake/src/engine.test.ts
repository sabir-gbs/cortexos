import { describe, it, expect } from 'vitest';
import {
  createInitialState,
  tick,
  changeDirection,
  randomFood,
  isCollision,
  GRID_SIZE,
  SnakeState,
  Direction,
  Point,
} from './engine';

describe('Snake Engine', () => {
  describe('createInitialState', () => {
    it('creates a snake of length 3', () => {
      const state = createInitialState();
      expect(state.snake).toHaveLength(3);
    });

    it('places the snake head at (10, 10)', () => {
      const state = createInitialState();
      expect(state.snake[0]).toEqual({ x: 10, y: 10 });
    });

    it('sets direction to UP', () => {
      const state = createInitialState();
      expect(state.direction).toBe('UP');
    });

    it('sets score to 0', () => {
      const state = createInitialState();
      expect(state.score).toBe(0);
    });

    it('sets gameOver to false', () => {
      const state = createInitialState();
      expect(state.gameOver).toBe(false);
    });

    it('sets paused to false', () => {
      const state = createInitialState();
      expect(state.paused).toBe(false);
    });

    it('places food not on the snake', () => {
      const state = createInitialState();
      const onSnake = state.snake.some(
        s => s.x === state.food.x && s.y === state.food.y
      );
      expect(onSnake).toBe(false);
    });

    it('places food within grid bounds', () => {
      const state = createInitialState();
      expect(state.food.x).toBeGreaterThanOrEqual(0);
      expect(state.food.x).toBeLessThan(GRID_SIZE);
      expect(state.food.y).toBeGreaterThanOrEqual(0);
      expect(state.food.y).toBeLessThan(GRID_SIZE);
    });
  });

  describe('tick', () => {
    it('moves the snake head in the current direction (UP)', () => {
      const state = createInitialState();
      const next = tick(state);
      expect(next.snake[0]).toEqual({ x: 10, y: 9 });
    });

    it('moves the snake head DOWN', () => {
      // Use a snake going UP first, then switch to DOWN from a safe position
      const snake: Point[] = [{ x: 5, y: 5 }, { x: 5, y: 4 }, { x: 5, y: 3 }];
      const state: SnakeState = {
        snake,
        food: { x: 0, y: 0 },
        direction: 'DOWN',
        score: 0,
        gameOver: false,
        paused: false,
      };
      const next = tick(state);
      expect(next.snake[0]).toEqual({ x: 5, y: 6 });
    });

    it('moves the snake head LEFT', () => {
      const state: SnakeState = {
        ...createInitialState(),
        direction: 'LEFT',
      };
      const next = tick(state);
      expect(next.snake[0]).toEqual({ x: 9, y: 10 });
    });

    it('moves the snake head RIGHT', () => {
      const state: SnakeState = {
        ...createInitialState(),
        direction: 'RIGHT',
      };
      const next = tick(state);
      expect(next.snake[0]).toEqual({ x: 11, y: 10 });
    });

    it('keeps the snake the same length when not eating', () => {
      const state = createInitialState();
      const next = tick(state);
      expect(next.snake).toHaveLength(state.snake.length);
    });

    it('grows the snake when eating food', () => {
      const state = createInitialState();
      // Place food directly in front of the snake (UP from head at 10,10)
      const stateWithFood: SnakeState = {
        ...state,
        food: { x: 10, y: 9 },
      };
      const next = tick(stateWithFood);
      expect(next.snake).toHaveLength(state.snake.length + 1);
    });

    it('increases score by 10 when eating food', () => {
      const state = createInitialState();
      const stateWithFood: SnakeState = {
        ...state,
        food: { x: 10, y: 9 },
      };
      const next = tick(stateWithFood);
      expect(next.score).toBe(10);
    });

    it('does not increase score when not eating food', () => {
      const state = createInitialState();
      const next = tick(state);
      expect(next.score).toBe(0);
    });

    it('generates new food when eating', () => {
      const state = createInitialState();
      const food = { x: 10, y: 9 };
      const stateWithFood: SnakeState = { ...state, food };
      const next = tick(stateWithFood);
      // The food should change (it was eaten)
      expect(next.food).not.toEqual(food);
    });

    it('keeps the same food when not eating', () => {
      const state = createInitialState();
      const next = tick(state);
      expect(next.food).toEqual(state.food);
    });

    describe('wall collision', () => {
      it('causes game over when hitting the top wall', () => {
        const snake: Point[] = [{ x: 5, y: 0 }, { x: 5, y: 1 }, { x: 5, y: 2 }];
        const state: SnakeState = {
          snake,
          food: { x: 0, y: 0 },
          direction: 'UP',
          score: 0,
          gameOver: false,
          paused: false,
        };
        const next = tick(state);
        expect(next.gameOver).toBe(true);
      });

      it('causes game over when hitting the bottom wall', () => {
        const snake: Point[] = [
          { x: 5, y: GRID_SIZE - 1 },
          { x: 5, y: GRID_SIZE - 2 },
          { x: 5, y: GRID_SIZE - 3 },
        ];
        const state: SnakeState = {
          snake,
          food: { x: 0, y: 0 },
          direction: 'DOWN',
          score: 0,
          gameOver: false,
          paused: false,
        };
        const next = tick(state);
        expect(next.gameOver).toBe(true);
      });

      it('causes game over when hitting the left wall', () => {
        const snake: Point[] = [{ x: 0, y: 5 }, { x: 1, y: 5 }, { x: 2, y: 5 }];
        const state: SnakeState = {
          snake,
          food: { x: 0, y: 0 },
          direction: 'LEFT',
          score: 0,
          gameOver: false,
          paused: false,
        };
        const next = tick(state);
        expect(next.gameOver).toBe(true);
      });

      it('causes game over when hitting the right wall', () => {
        const snake: Point[] = [
          { x: GRID_SIZE - 1, y: 5 },
          { x: GRID_SIZE - 2, y: 5 },
          { x: GRID_SIZE - 3, y: 5 },
        ];
        const state: SnakeState = {
          snake,
          food: { x: 0, y: 0 },
          direction: 'RIGHT',
          score: 0,
          gameOver: false,
          paused: false,
        };
        const next = tick(state);
        expect(next.gameOver).toBe(true);
      });
    });

    describe('self collision', () => {
      it('causes game over when the head hits the body', () => {
        // Snake going RIGHT, body is to the right of the head
        // We'll make a snake that will collide with itself on the next tick
        const snake: Point[] = [
          { x: 5, y: 5 },
          { x: 6, y: 5 },
          { x: 6, y: 4 },
          { x: 5, y: 4 },
        ];
        const state: SnakeState = {
          snake,
          food: { x: 0, y: 0 },
          direction: 'RIGHT',
          score: 0,
          gameOver: false,
          paused: false,
        };
        // Head at (5,5) moving RIGHT -> new head (6,5) which is snake[1]
        const next = tick(state);
        expect(next.gameOver).toBe(true);
      });
    });

    describe('paused state', () => {
      it('does not move the snake when paused', () => {
        const state: SnakeState = {
          ...createInitialState(),
          paused: true,
        };
        const next = tick(state);
        // State should be identical
        expect(next.snake).toEqual(state.snake);
        expect(next.score).toBe(state.score);
        expect(next.gameOver).toBe(state.gameOver);
      });
    });

    describe('game over state', () => {
      it('does not move the snake when game is over', () => {
        const state: SnakeState = {
          ...createInitialState(),
          gameOver: true,
        };
        const next = tick(state);
        expect(next.snake).toEqual(state.snake);
        expect(next.score).toBe(state.score);
      });
    });
  });

  describe('changeDirection', () => {
    it('allows turning left from UP', () => {
      expect(changeDirection('UP', 'LEFT')).toBe('LEFT');
    });

    it('allows turning right from UP', () => {
      expect(changeDirection('UP', 'RIGHT')).toBe('RIGHT');
    });

    it('allows turning left from DOWN', () => {
      expect(changeDirection('DOWN', 'LEFT')).toBe('LEFT');
    });

    it('allows turning right from DOWN', () => {
      expect(changeDirection('DOWN', 'RIGHT')).toBe('RIGHT');
    });

    it('allows turning up from LEFT', () => {
      expect(changeDirection('LEFT', 'UP')).toBe('UP');
    });

    it('allows turning down from LEFT', () => {
      expect(changeDirection('LEFT', 'DOWN')).toBe('DOWN');
    });

    it('allows turning up from RIGHT', () => {
      expect(changeDirection('RIGHT', 'UP')).toBe('UP');
    });

    it('allows turning down from RIGHT', () => {
      expect(changeDirection('RIGHT', 'DOWN')).toBe('DOWN');
    });

    it('rejects reversing from UP to DOWN', () => {
      expect(changeDirection('UP', 'DOWN')).toBe('UP');
    });

    it('rejects reversing from DOWN to UP', () => {
      expect(changeDirection('DOWN', 'UP')).toBe('DOWN');
    });

    it('rejects reversing from LEFT to RIGHT', () => {
      expect(changeDirection('LEFT', 'RIGHT')).toBe('LEFT');
    });

    it('rejects reversing from RIGHT to LEFT', () => {
      expect(changeDirection('RIGHT', 'LEFT')).toBe('RIGHT');
    });

    it('allows continuing in the same direction', () => {
      expect(changeDirection('UP', 'UP')).toBe('UP');
      expect(changeDirection('DOWN', 'DOWN')).toBe('DOWN');
      expect(changeDirection('LEFT', 'LEFT')).toBe('LEFT');
      expect(changeDirection('RIGHT', 'RIGHT')).toBe('RIGHT');
    });
  });

  describe('randomFood', () => {
    it('never places food on the snake', () => {
      // Use a snake that fills most of the board to stress-test
      const snake: Point[] = [];
      for (let y = 0; y < GRID_SIZE; y++) {
        for (let x = 0; x < GRID_SIZE - 1; x++) {
          snake.push({ x, y });
        }
      }
      // Only column GRID_SIZE-1 is free, so food must land there
      for (let i = 0; i < 50; i++) {
        const food = randomFood(snake);
        const onSnake = snake.some(s => s.x === food.x && s.y === food.y);
        expect(onSnake).toBe(false);
      }
    });

    it('places food within grid bounds', () => {
      const snake: Point[] = [{ x: 0, y: 0 }];
      for (let i = 0; i < 50; i++) {
        const food = randomFood(snake);
        expect(food.x).toBeGreaterThanOrEqual(0);
        expect(food.x).toBeLessThan(GRID_SIZE);
        expect(food.y).toBeGreaterThanOrEqual(0);
        expect(food.y).toBeLessThan(GRID_SIZE);
      }
    });
  });

  describe('isCollision', () => {
    it('detects wall collision (out of bounds)', () => {
      const state: SnakeState = {
        snake: [{ x: -1, y: 5 }],
        food: { x: 0, y: 0 },
        direction: 'UP',
        score: 0,
        gameOver: false,
        paused: false,
      };
      expect(isCollision(state)).toBe(true);
    });

    it('detects self collision', () => {
      const state: SnakeState = {
        snake: [{ x: 5, y: 5 }, { x: 5, y: 5 }],
        food: { x: 0, y: 0 },
        direction: 'UP',
        score: 0,
        gameOver: false,
        paused: false,
      };
      expect(isCollision(state)).toBe(true);
    });

    it('returns false when no collision', () => {
      const state = createInitialState();
      expect(isCollision(state)).toBe(false);
    });
  });
});
