// Barrel export for @cortexos/game-framework

// Types
export type {
  GameStatus,
  Difficulty,
  GameStateBase,
  MoveRecord,
  HighScoreEntry,
  HighScoreStore,
  KeyboardInput,
  MouseInput,
  GameInput,
  IGameEngine,
  GameManifest,
  GameThemeTokens,
} from "./types";

// Core services
export { GameLoop } from "./GameLoop";
export { InputManager } from "./InputManager";
export { StateSerializer } from "./StateSerializer";
export { TimerService } from "./TimerService";
export { ScoreService } from "./ScoreService";
export { UndoManager } from "./UndoManager";

// React components
export {
  Toolbar,
  ScoreDisplay,
  TimerDisplay,
  GameOverlay,
  HelpOverlay,
} from "./GameChrome";
export type {
  ToolbarProps,
  ScoreDisplayProps,
  TimerDisplayProps,
  GameOverlayProps,
  HelpOverlayProps,
} from "./GameChrome";
