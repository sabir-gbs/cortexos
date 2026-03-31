export const GRID_SIZE = 20;

export interface Point {
  x: number;
  y: number;
}

export type Direction = 'UP' | 'DOWN' | 'LEFT' | 'RIGHT';

export interface SnakeState {
  snake: Point[];
  food: Point;
  direction: Direction;
  score: number;
  gameOver: boolean;
  paused: boolean;
}

export function randomFood(snake: Point[]): Point {
  let pos: Point;
  do {
    pos = {
      x: Math.floor(Math.random() * GRID_SIZE),
      y: Math.floor(Math.random() * GRID_SIZE),
    };
  } while (snake.some(s => s.x === pos.x && s.y === pos.y));
  return pos;
}

export function createInitialState(): SnakeState {
  const snake: Point[] = [{ x: 10, y: 10 }, { x: 10, y: 11 }, { x: 10, y: 12 }];
  return {
    snake,
    food: randomFood(snake),
    direction: 'UP',
    score: 0,
    gameOver: false,
    paused: false,
  };
}

export function tick(state: SnakeState): SnakeState {
  if (state.gameOver || state.paused) return state;

  const { snake, direction, food, score } = state;
  const head = snake[0];

  let newHead: Point;
  switch (direction) {
    case 'UP': newHead = { x: head.x, y: head.y - 1 }; break;
    case 'DOWN': newHead = { x: head.x, y: head.y + 1 }; break;
    case 'LEFT': newHead = { x: head.x - 1, y: head.y }; break;
    case 'RIGHT': newHead = { x: head.x + 1, y: head.y }; break;
  }

  // Check collision with walls
  if (newHead.x < 0 || newHead.x >= GRID_SIZE || newHead.y < 0 || newHead.y >= GRID_SIZE) {
    return { ...state, gameOver: true };
  }

  // Check collision with self
  if (snake.some(s => s.x === newHead.x && s.y === newHead.y)) {
    return { ...state, gameOver: true };
  }

  const ate = newHead.x === food.x && newHead.y === food.y;
  const newSnake = [newHead, ...snake];
  if (!ate) {
    newSnake.pop();
  }

  return {
    ...state,
    snake: newSnake,
    food: ate ? randomFood(newSnake) : food,
    score: ate ? score + 10 : score,
  };
}

export function changeDirection(current: Direction, requested: Direction): Direction {
  const opposites: Record<Direction, Direction> = { UP: 'DOWN', DOWN: 'UP', LEFT: 'RIGHT', RIGHT: 'LEFT' };
  if (opposites[requested] === current) return current;
  return requested;
}

export function isCollision(state: SnakeState): boolean {
  const head = state.snake[0];
  if (head.x < 0 || head.x >= GRID_SIZE || head.y < 0 || head.y >= GRID_SIZE) return true;
  return state.snake.some((s, i) => i > 0 && s.x === head.x && s.y === head.y);
}
