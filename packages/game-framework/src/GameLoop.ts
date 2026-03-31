/**
 * requestAnimationFrame-based game loop.
 *
 * Usage:
 *   const loop = new GameLoop();
 *   loop.start((dt) => { /* update game *\/ });
 *   // later …
 *   loop.stop();
 */
export class GameLoop {
  private running = false;
  private paused = false;
  private rafId: number | null = null;
  private lastTime = 0;
  private onUpdate: ((deltaTime: number) => void) | null = null;

  // ------------------------------------------------------------------
  // Public API
  // ------------------------------------------------------------------

  /** Start the loop. @throws if already running. */
  start(callback: (deltaTime: number) => void): void {
    if (this.running) {
      throw new Error("GameLoop is already running");
    }
    this.onUpdate = callback;
    this.running = true;
    this.paused = false;
    this.lastTime = performance.now();
    this.rafId = requestAnimationFrame(this.tick);
  }

  /** Stop the loop entirely. Can be restarted with `start()`. */
  stop(): void {
    if (this.rafId !== null) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
    this.running = false;
    this.paused = false;
    this.onUpdate = null;
  }

  /** Pause updates. The loop keeps running but skips the callback. */
  pause(): void {
    if (!this.running) return;
    this.paused = true;
  }

  /** Resume after a pause. */
  resume(): void {
    if (!this.running) return;
    this.paused = false;
    // Reset lastTime so the first frame after resume doesn't include the
    // paused duration as a giant delta.
    this.lastTime = performance.now();
  }

  isRunning(): boolean {
    return this.running;
  }

  isPaused(): boolean {
    return this.paused;
  }

  // ------------------------------------------------------------------
  // Internals
  // ------------------------------------------------------------------

  private tick = (now: number): void => {
    if (!this.running) return;

    this.rafId = requestAnimationFrame(this.tick);

    if (this.paused) {
      this.lastTime = now;
      return;
    }

    const deltaTime = (now - this.lastTime) / 1000; // seconds
    this.lastTime = now;

    // Guard against tab-switch causing a huge delta on the first frame back.
    const clampedDelta = Math.min(deltaTime, 0.25);

    this.onUpdate?.(clampedDelta);
  };
}
