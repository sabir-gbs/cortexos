import React, { useState, useEffect, useCallback, useRef } from 'react';
import './App.css';
import {
  GRID_SIZE,
  Direction,
  Point,
  SnakeState,
  randomFood,
  createInitialState,
  tick,
  changeDirection,
  isCollision,
} from './engine';

const INITIAL_SPEED = 150;

const App: React.FC = () => {
  const initialState = createInitialState();

  const [snake, setSnake] = useState<Point[]>(initialState.snake);
  const [food, setFood] = useState<Point>(initialState.food);
  const [direction, setDirection] = useState<Direction>(initialState.direction);
  const [gameOver, setGameOver] = useState(initialState.gameOver);
  const [score, setScore] = useState(initialState.score);
  const [paused, setPaused] = useState(initialState.paused);

  const directionRef = useRef(direction);
  const snakeRef = useRef(snake);
  const foodRef = useRef(food);
  const gameOverRef = useRef(gameOver);
  const pausedRef = useRef(paused);

  directionRef.current = direction;
  snakeRef.current = snake;
  foodRef.current = food;
  gameOverRef.current = gameOver;
  pausedRef.current = paused;

  const resetGame = useCallback(() => {
    const state = createInitialState();
    setSnake(state.snake);
    setFood(state.food);
    setDirection(state.direction);
    setGameOver(state.gameOver);
    setScore(state.score);
    setPaused(state.paused);
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', ' '].includes(e.key)) {
        e.preventDefault();
      }

      if (e.key === ' ') {
        if (gameOverRef.current) {
          resetGame();
          return;
        }
        setPaused(p => !p);
        return;
      }

      const dir = directionRef.current;
      switch (e.key) {
        case 'ArrowUp':
          setDirection(changeDirection(dir, 'UP'));
          break;
        case 'ArrowDown':
          setDirection(changeDirection(dir, 'DOWN'));
          break;
        case 'ArrowLeft':
          setDirection(changeDirection(dir, 'LEFT'));
          break;
        case 'ArrowRight':
          setDirection(changeDirection(dir, 'RIGHT'));
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [resetGame]);

  useEffect(() => {
    const interval = setInterval(() => {
      if (gameOverRef.current || pausedRef.current) return;

      const state: SnakeState = {
        snake: snakeRef.current,
        food: foodRef.current,
        direction: directionRef.current,
        score: 0,
        gameOver: false,
        paused: false,
      };

      const next = tick(state);

      if (next.gameOver) {
        setGameOver(true);
        return;
      }

      if (next.snake.length > snakeRef.current.length) {
        setScore(s => s + 10);
        setFood(next.food);
      }

      setSnake(next.snake);
    }, INITIAL_SPEED);

    return () => clearInterval(interval);
  }, []);

  const isSnakeHead = (x: number, y: number): boolean =>
    snake.length > 0 && snake[0].x === x && snake[0].y === y;

  const isSnakeBody = (x: number, y: number): boolean =>
    snake.some((s, i) => i > 0 && s.x === x && s.y === y);

  const isFood = (x: number, y: number): boolean =>
    food.x === x && food.y === y;

  return (
    <div className="app">
      <div className="header">
        <h1>Snake</h1>
        <span className="score">Score: {score}</span>
        {gameOver && <span className="status-text lost">Game Over</span>}
        {paused && !gameOver && <span className="status-text">Paused</span>}
        <button className="restart-btn" onClick={resetGame}>Restart</button>
      </div>

      <div className="board">
        {Array.from({ length: GRID_SIZE }, (_, y) =>
          Array.from({ length: GRID_SIZE }, (_, x) => {
            let cls = 'cell';
            if (isSnakeHead(x, y)) cls = 'cell snake-head';
            else if (isSnakeBody(x, y)) cls = 'cell snake';
            else if (isFood(x, y)) cls = 'cell food';
            return <div key={`${x}-${y}`} className={cls} />;
          })
        )}
      </div>

      <div className="footer">Arrow keys to move, Space to pause</div>
    </div>
  );
};

export default App;
