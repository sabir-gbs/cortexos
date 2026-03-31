// Shared game types for CortexOS games (Spec 18)

/** Current status of a game session. */
export type GameStatus = "idle" | "playing" | "paused" | "game_over" | "won";

/** Difficulty levels supported by games. */
export type Difficulty = "easy" | "medium" | "hard" | "expert";

/** Base state that every game must extend. */
export interface GameStateBase {
  schemaVersion: number;
  status: GameStatus;
  score: number;
  timerElapsed: number;
  timerStarted: boolean;
  difficulty: Difficulty;
  moveHistory: MoveRecord[];
  undoPointer: number;
  startedAt: number;
  lastSaved: number;
}

/** A single move recorded in the game's move history. */
export interface MoveRecord {
  /** Monotonically increasing move number. */
  index: number;
  /** Machine-readable move identifier, e.g. "flip:3,4" or "move:e2-e4". */
  action: string;
  /** Human-readable description for replay / debugging. */
  description?: string;
  /** Unix-ms timestamp when the move was made. */
  timestamp: number;
}

/** One entry in the high-score table. */
export interface HighScoreEntry {
  playerName: string;
  score: number;
  difficulty: Difficulty;
  /** Unix-ms timestamp when the score was achieved. */
  achievedAt: number;
}

/** Persistence shape stored in localStorage (or similar). */
export interface HighScoreStore {
  /** Map key is the difficulty string; value is a sorted array of entries. */
  entries: Record<string, HighScoreEntry[]>;
}

/** Keyboard state for the current frame. */
export interface KeyboardInput {
  /** All keys currently held down. */
  pressedKeys: Set<string>;
  /** Keys that were pressed since the last frame. */
  keyPressedThisFrame: string[];
  /** Keys that were released since the last frame. */
  keyReleasedThisFrame: string[];
}

/** Mouse state for the current frame. */
export interface MouseInput {
  /** Current cursor position relative to the attached element. */
  position: { x: number; y: number };
  /** True when the left button was clicked this frame. */
  clicked: boolean;
  /** True when the right button was clicked this frame. */
  rightClicked: boolean;
  /** True when a double-click occurred this frame. */
  doubleClicked: boolean;
  /** Starting position of the current drag, if any. */
  dragStart: { x: number; y: number } | null;
  /** Ending position of the most recent drag. */
  dragEnd: { x: number; y: number } | null;
  /** Whether a drag is in progress. */
  isDragging: boolean;
}

/** Combined input snapshot for one frame. */
export interface GameInput {
  keyboard: KeyboardInput;
  mouse: MouseInput;
}

/** Interface that every game engine must implement. */
export interface IGameEngine<TState extends GameStateBase = GameStateBase> {
  /** Unique identifier for the game, e.g. "minesweeper". */
  id: string;
  /** Human-readable name, e.g. "Minesweeper". */
  name: string;

  /** Create the initial game state for the given difficulty. */
  createInitialState(difficulty: Difficulty): TState;

  /**
   * Advance the game state by one tick.
   * @param state  Current game state (should not be mutated).
   * @param input  Input snapshot for this frame.
   * @param dt     Delta time in seconds since last update.
   * @returns      New game state.
   */
  update(state: TState, input: GameInput, dt: number): TState;

  /** Render the current state. Returns React node (or void for canvas games). */
  render(state: TState): React.ReactNode;

  /** Return true when the player has met the win condition. */
  checkWinCondition(state: TState): boolean;

  /** Return true when the player has met the loss condition. */
  checkLossCondition(state: TState): boolean;

  /** Calculate / recalculate the score from state. */
  calculateScore(state: TState): number;

  /** Return a list of valid action strings the player can take right now. */
  getValidActions(state: TState): string[];

  /** Serialize state to a JSON-safe plain object. */
  serializeState(state: TState): unknown;

  /** Deserialize a plain object back into a game state. */
  deserializeState(data: unknown): TState;
}

/** Metadata describing a game for the game launcher / manifest. */
export interface GameManifest {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: string;
  supportedDifficulties: Difficulty[];
  minScreenSize: { width: number; height: number };
  hasTimer: boolean;
  hasScore: boolean;
}

/** Theme tokens that games can read via CSS custom properties. */
export interface GameThemeTokens {
  "--game-board-bg": string;
  "--game-board-border": string;
  "--game-cell-bg": string;
  "--game-cell-border": string;
  "--game-cell-hover": string;
  "--game-cell-revealed": string;
  "--game-text-primary": string;
  "--game-text-secondary": string;
  "--game-text-accent": string;
  "--game-toolbar-bg": string;
  "--game-toolbar-border": string;
  "--game-overlay-bg": string;
  "--game-button-bg": string;
  "--game-button-text": string;
  "--game-button-hover": string;
  "--game-button-active": string;
  "--game-success": string;
  "--game-warning": string;
  "--game-danger": string;
  "--game-info": string;
  "--game-font-mono": string;
  "--game-font-sans": string;
  "--game-radius-sm": string;
  "--game-radius-md": string;
  "--game-radius-lg": string;
  "--game-transition-fast": string;
  "--game-transition-normal": string;
}
