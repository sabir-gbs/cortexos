import { describe, it, expect } from "vitest";
import { UndoManager } from "../UndoManager";

describe("UndoManager", () => {
  it("starts empty", () => {
    const mgr = new UndoManager<string>();
    expect(mgr.getHistorySize()).toBe(0);
    expect(mgr.canUndo()).toBe(false);
    expect(mgr.getCurrent()).toBeNull();
    expect(mgr.undo()).toBeNull();
  });

  it("push adds to history and moves pointer", () => {
    const mgr = new UndoManager<string>();
    mgr.push("a");
    mgr.push("b");
    mgr.push("c");
    expect(mgr.getHistorySize()).toBe(3);
    expect(mgr.getCurrent()).toBe("c");
    expect(mgr.getPointer()).toBe(2);
  });

  it("undo moves pointer back and returns previous state", () => {
    const mgr = new UndoManager<string>();
    mgr.push("a");
    mgr.push("b");
    mgr.push("c");

    const result = mgr.undo();
    expect(result).toBe("b");
    expect(mgr.getCurrent()).toBe("b");
    expect(mgr.getPointer()).toBe(1);
  });

  it("undo returns null when at the beginning", () => {
    const mgr = new UndoManager<string>();
    mgr.push("only");
    expect(mgr.undo()).toBeNull();
    expect(mgr.getCurrent()).toBe("only");
  });

  it("canUndo reports correctly", () => {
    const mgr = new UndoManager<string>();
    expect(mgr.canUndo()).toBe(false);

    mgr.push("first");
    expect(mgr.canUndo()).toBe(false); // Only one entry; nothing to go back to.

    mgr.push("second");
    expect(mgr.canUndo()).toBe(true);
  });

  it("push after undo discards future entries", () => {
    const mgr = new UndoManager<string>();
    mgr.push("a");
    mgr.push("b");
    mgr.push("c");

    mgr.undo(); // back to "b"
    mgr.push("x");

    expect(mgr.getHistorySize()).toBe(3); // a, b, x
    expect(mgr.getCurrent()).toBe("x");
    // "c" is gone.
    expect(mgr.getHistory()).toEqual(["a", "b", "x"]);
  });

  it("respects maxSize and evicts oldest entries", () => {
    const mgr = new UndoManager<number>(5);
    for (let i = 0; i < 7; i++) {
      mgr.push(i);
    }
    // maxSize is 5, so the first two entries (0, 1) should have been evicted.
    expect(mgr.getHistorySize()).toBe(5);
    expect(mgr.getCurrent()).toBe(6);
    expect(mgr.getPointer()).toBe(4);
  });

  it("clear resets everything", () => {
    const mgr = new UndoManager<string>();
    mgr.push("a");
    mgr.push("b");
    mgr.clear();
    expect(mgr.getHistorySize()).toBe(0);
    expect(mgr.getPointer()).toBe(-1);
    expect(mgr.canUndo()).toBe(false);
    expect(mgr.getCurrent()).toBeNull();
  });

  it("works with object types", () => {
    interface State {
      value: number;
      label: string;
    }
    const mgr = new UndoManager<State>();
    mgr.push({ value: 1, label: "one" });
    mgr.push({ value: 2, label: "two" });

    const prev = mgr.undo();
    expect(prev!.value).toBe(1);
    expect(prev!.label).toBe("one");
  });

  it("handles rapid push/undo cycles", () => {
    const mgr = new UndoManager<number>();
    for (let i = 0; i < 100; i++) {
      mgr.push(i);
    }
    for (let i = 0; i < 50; i++) {
      mgr.undo();
    }
    expect(mgr.getCurrent()).toBe(49);
    expect(mgr.getPointer()).toBe(49);
  });
});
