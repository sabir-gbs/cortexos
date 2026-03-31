import { describe, it, expect } from "vitest";
import { StateSerializer } from "../StateSerializer";
import type { GameStateBase } from "../types";

function makeState(overrides: Partial<GameStateBase> = {}): GameStateBase {
  return {
    schemaVersion: 1,
    status: "playing",
    score: 100,
    timerElapsed: 42.5,
    timerStarted: true,
    difficulty: "medium",
    moveHistory: [
      { index: 1, action: "flip:3,4", description: "Flipped cell 3,4", timestamp: Date.now() },
    ],
    undoPointer: 0,
    startedAt: Date.now() - 40000,
    lastSaved: Date.now(),
    ...overrides,
  };
}

describe("StateSerializer", () => {
  const serializer = new StateSerializer();

  it("serializes and deserializes a basic state", () => {
    const state = makeState();
    const json = serializer.serialize(state);
    const restored = serializer.deserialize(json, 1);

    expect(restored).not.toBeNull();
    expect(restored!.score).toBe(100);
    expect(restored!.status).toBe("playing");
    expect(restored!.difficulty).toBe("medium");
  });

  it("preserves move history", () => {
    const state = makeState();
    const json = serializer.serialize(state);
    const restored = serializer.deserialize(json, 1);

    expect(restored!.moveHistory).toHaveLength(1);
    expect(restored!.moveHistory[0].action).toBe("flip:3,4");
  });

  it("returns null on malformed JSON", () => {
    expect(serializer.deserialize("not json", 1)).toBeNull();
  });

  it("returns null on missing envelope fields", () => {
    expect(serializer.deserialize("{}", 1)).toBeNull();
  });

  it("returns null on version mismatch", () => {
    const state = makeState({ schemaVersion: 2 });
    const json = serializer.serialize(state);
    // Try to deserialize expecting version 1.
    expect(serializer.deserialize(json, 1)).toBeNull();
  });

  it("returns null when envelope.v is not a number", () => {
    const payload = JSON.stringify({ v: "one", s: {} });
    expect(serializer.deserialize(payload, 1)).toBeNull();
  });

  it("handles states containing Set fields", () => {
    // Create a state-like object that has a Set.
    const state = {
      ...makeState(),
      revealedCells: new Set(["3,4", "5,6"]),
    };
    const json = serializer.serialize(state as unknown as GameStateBase);
    const restored = serializer.deserialize(json, 1);

    expect(restored).not.toBeNull();
    // The Set should be converted to a portable representation.
    const revealed = (restored as unknown as Record<string, unknown>).revealedCells as {
      __type: string;
      values: string[];
    };
    expect(revealed.__type).toBe("Set");
    expect(revealed.values).toEqual(["3,4", "5,6"]);
  });

  it("handles states containing Map fields", () => {
    const state = {
      ...makeState(),
      cellValues: new Map([
        ["3,4", 1],
        ["5,6", 2],
      ]),
    };
    const json = serializer.serialize(state as unknown as GameStateBase);
    const restored = serializer.deserialize(json, 1);

    expect(restored).not.toBeNull();
    const cells = (restored as unknown as Record<string, unknown>).cellValues as {
      __type: string;
      entries: [string, number][];
    };
    expect(cells.__type).toBe("Map");
    expect(cells.entries).toEqual([
      ["3,4", 1],
      ["5,6", 2],
    ]);
  });

  it("round-trips numeric values correctly", () => {
    const state = makeState({ timerElapsed: 123.456, score: 9999 });
    const json = serializer.serialize(state);
    const restored = serializer.deserialize(json, 1);
    expect(restored!.timerElapsed).toBeCloseTo(123.456);
    expect(restored!.score).toBe(9999);
  });
});
