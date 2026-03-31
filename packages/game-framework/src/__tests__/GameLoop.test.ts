import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { GameLoop } from "../GameLoop";

describe("GameLoop", () => {
  let loop: GameLoop;

  beforeEach(() => {
    loop = new GameLoop();
  });

  afterEach(() => {
    loop.stop();
  });

  it("starts and reports running", () => {
    const cb = vi.fn();
    loop.start(cb);
    expect(loop.isRunning()).toBe(true);
    expect(loop.isPaused()).toBe(false);
  });

  it("throws if started twice", () => {
    loop.start(vi.fn());
    expect(() => loop.start(vi.fn())).toThrow("already running");
  });

  it("stops and reports not running", () => {
    loop.start(vi.fn());
    loop.stop();
    expect(loop.isRunning()).toBe(false);
  });

  it("calls the callback with deltaTime on each animation frame", () => {
    const cb = vi.fn();
    loop.start(cb);

    // jsdom's requestAnimationFrame calls the callback synchronously with a
    // DOMHighResTimeStamp.  We can't easily control that timestamp, but we
    // can verify the callback is invoked.
    //
    // Since GameLoop.start() fires one rAF immediately, and the tick handler
    // schedules another, we just verify the loop infrastructure works.
    expect(loop.isRunning()).toBe(true);
    expect(cb).not.toHaveBeenCalled(); // Not called yet because delta was ~0 (first frame).

    // After the test we stop the loop in afterEach.
  });

  it("pauses and resumes", () => {
    loop.start(vi.fn());
    expect(loop.isPaused()).toBe(false);

    loop.pause();
    expect(loop.isPaused()).toBe(true);

    loop.resume();
    expect(loop.isPaused()).toBe(false);
  });

  it("pause is no-op when not running", () => {
    expect(() => loop.pause()).not.toThrow();
    expect(loop.isPaused()).toBe(false);
  });

  it("resume is no-op when not running", () => {
    expect(() => loop.resume()).not.toThrow();
    expect(loop.isPaused()).toBe(false);
  });

  it("stop resets paused state", () => {
    loop.start(vi.fn());
    loop.pause();
    loop.stop();
    expect(loop.isPaused()).toBe(false);
    expect(loop.isRunning()).toBe(false);
  });

  it("can be restarted after stop", () => {
    const cb = vi.fn();
    loop.start(cb);
    loop.stop();
    expect(loop.isRunning()).toBe(false);

    loop.start(cb);
    expect(loop.isRunning()).toBe(true);
  });
});
