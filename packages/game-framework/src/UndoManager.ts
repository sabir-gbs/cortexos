/**
 * Generic undo manager with a bounded history.
 *
 * The manager keeps a linear history and a pointer into it.  `push`
 * truncates any future entries (like a browser history push).  `undo`
 * walks the pointer backwards.
 */
export class UndoManager<T> {
  private history: T[] = [];
  private pointer = -1;
  private maxSize: number;

  constructor(maxSize = 1000) {
    this.maxSize = maxSize;
  }

  /**
   * Push a new state onto the history.
   *
   * Any states ahead of the current pointer are discarded (redo is not
   * supported by this implementation).  If the history exceeds `maxSize`,
   * the oldest entry is evicted.
   */
  push(state: T): void {
    // Discard anything ahead of the pointer.
    if (this.pointer < this.history.length - 1) {
      this.history = this.history.slice(0, this.pointer + 1);
    }

    this.history.push(state);
    this.pointer = this.history.length - 1;

    // Evict oldest if over capacity.
    if (this.history.length > this.maxSize) {
      this.history.shift();
      this.pointer--;
    }
  }

  /**
   * Undo: return the previous state and move the pointer back.
   * Returns `null` if there is nothing to undo.
   */
  undo(): T | null {
    if (!this.canUndo()) return null;
    this.pointer--;
    return this.history[this.pointer];
  }

  /** Whether the pointer can move backwards. */
  canUndo(): boolean {
    return this.pointer > 0;
  }

  /** Clear all history and reset the pointer. */
  clear(): void {
    this.history = [];
    this.pointer = -1;
  }

  /** Number of entries currently stored. */
  getHistorySize(): number {
    return this.history.length;
  }

  /** Current pointer position (0-based). -1 if empty. */
  getPointer(): number {
    return this.pointer;
  }

  /** Return the state at the current pointer, or `null` if empty. */
  getCurrent(): T | null {
    return this.pointer >= 0 ? this.history[this.pointer] : null;
  }

  /** Return a shallow copy of the history array. */
  getHistory(): T[] {
    return [...this.history];
  }
}
