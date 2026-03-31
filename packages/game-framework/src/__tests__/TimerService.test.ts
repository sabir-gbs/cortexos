import { describe, it, expect, beforeEach } from "vitest";
import { TimerService } from "../TimerService";

/**
 * Create a controllable mock clock.  Call `advance(ms)` to move time
 * forward.  The clock returns values in milliseconds (like performance.now).
 */
function createMockClock() {
  let now = 0;
  return {
    clock: () => now,
    advance: (ms: number) => {
      now += ms;
    },
    set: (ms: number) => {
      now = ms;
    },
  };
}

describe("TimerService", () => {
  let timer: TimerService;
  let mockClock: ReturnType<typeof createMockClock>;

  beforeEach(() => {
    mockClock = createMockClock();
    timer = new TimerService(mockClock.clock);
  });

  it("returns 0 before start", () => {
    expect(timer.getElapsed()).toBe(0);
  });

  it("tracks elapsed time after start", () => {
    timer.start();
    mockClock.advance(5000);
    expect(timer.getElapsed()).toBeCloseTo(5, 5);
  });

  it("tick returns delta and accumulates", () => {
    timer.start();
    mockClock.advance(1000);
    const delta = timer.tick();
    expect(delta).toBeCloseTo(1, 5);
    expect(timer.getElapsed()).toBeCloseTo(1, 5);
  });

  it("tick returns 0 when paused", () => {
    timer.start();
    timer.pause();
    mockClock.advance(1000);
    expect(timer.tick()).toBe(0);
  });

  it("tick returns 0 when not started", () => {
    expect(timer.tick()).toBe(0);
  });

  it("pause stops accumulation", () => {
    timer.start();
    mockClock.advance(3000);
    timer.pause();
    const elapsedAtPause = timer.getElapsed();

    mockClock.advance(5000);
    expect(timer.getElapsed()).toBeCloseTo(elapsedAtPause, 5);
  });

  it("resume continues accumulation", () => {
    timer.start();
    mockClock.advance(2000);
    timer.pause();

    mockClock.advance(5000); // time passes while paused -- ignored
    timer.resume();

    mockClock.advance(3000);
    // 2s before pause + 3s after resume = 5s total
    expect(timer.getElapsed()).toBeCloseTo(5, 5);
  });

  it("reset clears elapsed but keeps running", () => {
    timer.start();
    mockClock.advance(5000);
    timer.reset();
    expect(timer.getElapsed()).toBeCloseTo(0, 5);

    mockClock.advance(2000);
    expect(timer.getElapsed()).toBeCloseTo(2, 5);
  });

  it("formatTime returns MM:SS", () => {
    timer.start();
    mockClock.advance(125_000); // 2 min 5 sec
    expect(timer.formatTime()).toBe("02:05");
  });

  it("formatTime shows 00:00 initially", () => {
    timer.start();
    expect(timer.formatTime()).toBe("00:00");
  });

  it("formatTime handles large values", () => {
    timer.start();
    mockClock.advance(3661_000); // 1 hour 1 min 1 sec
    expect(timer.formatTime()).toBe("61:01");
  });

  it("pause is idempotent", () => {
    timer.start();
    timer.pause();
    timer.pause(); // second pause should be no-op
    mockClock.advance(1000);
    expect(timer.getElapsed()).toBeCloseTo(0, 5);
  });

  it("resume is idempotent when not paused", () => {
    timer.start();
    timer.resume(); // no-op
    mockClock.advance(2000);
    expect(timer.getElapsed()).toBeCloseTo(2, 5);
  });

  it("works with default clock (performance.now) for production use", () => {
    const prodTimer = new TimerService(); // no clock injected
    prodTimer.start();
    // Just verify it doesn't throw and returns something reasonable.
    const elapsed = prodTimer.getElapsed();
    expect(elapsed).toBeGreaterThanOrEqual(0);
    expect(typeof prodTimer.formatTime()).toBe("string");
  });
});
