/**
 * Game timer with start / pause / resume / reset.
 *
 * All time values are in **seconds**.
 *
 * An optional `clock` function can be injected for testing; in production
 * it defaults to `performance.now` (millisecond timestamps).
 */
export class TimerService {
  private elapsed = 0;
  private started = false;
  private paused = false;
  private lastTick = 0;
  private clock: () => number;

  constructor(clock?: () => number) {
    this.clock = clock ?? (() => performance.now());
  }

  /** Start (or restart) the timer. */
  start(): void {
    this.elapsed = 0;
    this.started = true;
    this.paused = false;
    this.lastTick = this.clock();
  }

  /** Pause the timer. No-op if not running. */
  pause(): void {
    if (!this.started || this.paused) return;
    // Account for time since last tick.
    this.elapsed += (this.clock() - this.lastTick) / 1000;
    this.paused = true;
  }

  /** Resume after a pause. */
  resume(): void {
    if (!this.started || !this.paused) return;
    this.paused = false;
    this.lastTick = this.clock();
  }

  /** Reset elapsed to zero. Keeps the timer in its current started/paused state. */
  reset(): void {
    this.elapsed = 0;
    this.lastTick = this.clock();
  }

  /** Total elapsed seconds (including time accumulated since the last tick). */
  getElapsed(): number {
    if (!this.started) return 0;
    if (this.paused) return this.elapsed;
    return this.elapsed + (this.clock() - this.lastTick) / 1000;
  }

  /**
   * Record a tick. Returns the delta (seconds) since the last tick and
   * accumulates it into elapsed.
   */
  tick(): number {
    if (!this.started || this.paused) return 0;
    const now = this.clock();
    const delta = (now - this.lastTick) / 1000;
    this.lastTick = now;
    this.elapsed += delta;
    return delta;
  }

  /** Format the elapsed time as MM:SS. */
  formatTime(): string {
    const total = Math.floor(this.getElapsed());
    const minutes = Math.floor(total / 60);
    const seconds = total % 60;
    return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  }
}
