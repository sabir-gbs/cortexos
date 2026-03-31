import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { InputManager } from "../InputManager";

/** Helper: fire a keyboard event on the window (jsdom). */
function fireKeyEvent(type: "keydown" | "keyup", key: string): void {
  const event = new KeyboardEvent(type, { key, bubbles: true });
  window.dispatchEvent(event);
}

/** Helper: fire a mouse event on an element. */
function fireMouseEvent(
  el: HTMLElement,
  type: string,
  opts: Partial<MouseEventInit> = {},
): void {
  const event = new MouseEvent(type, { bubbles: true, cancelable: true, ...opts });
  el.dispatchEvent(event);
}

describe("InputManager", () => {
  let manager: InputManager;
  let container: HTMLDivElement;

  beforeEach(() => {
    manager = new InputManager();
    container = document.createElement("div");
    // jsdom needs the element in the document for getBoundingClientRect to
    // return sensible values.
    document.body.appendChild(container);
  });

  afterEach(() => {
    manager.detach();
    container.remove();
  });

  it("attaches and detaches without error", () => {
    expect(() => manager.attach(container)).not.toThrow();
    expect(() => manager.detach()).not.toThrow();
  });

  it("captures key down events", () => {
    manager.attach(container);
    fireKeyEvent("keydown", "ArrowUp");

    const input = manager.getInput();
    expect(input.keyboard.pressedKeys.has("ArrowUp")).toBe(true);
    expect(input.keyboard.keyPressedThisFrame).toContain("ArrowUp");
  });

  it("captures key up events", () => {
    manager.attach(container);
    fireKeyEvent("keydown", "a");
    fireKeyEvent("keyup", "a");

    const input = manager.getInput();
    expect(input.keyboard.pressedKeys.has("a")).toBe(false);
    expect(input.keyboard.keyReleasedThisFrame).toContain("a");
  });

  it("does not double-add pressed keys on repeated keydown", () => {
    manager.attach(container);
    fireKeyEvent("keydown", "a");
    fireKeyEvent("keydown", "a"); // repeated keydown while still held

    const input = manager.getInput();
    expect(input.keyboard.keyPressedThisFrame.filter((k) => k === "a").length).toBe(1);
  });

  it("clearFrameState resets per-frame data", () => {
    manager.attach(container);
    fireKeyEvent("keydown", "x");
    fireKeyEvent("keyup", "y");

    manager.clearFrameState();
    const input = manager.getInput();
    expect(input.keyboard.keyPressedThisFrame).toEqual([]);
    expect(input.keyboard.keyReleasedThisFrame).toEqual([]);
    expect(input.mouse.clicked).toBe(false);
    expect(input.mouse.rightClicked).toBe(false);
    expect(input.mouse.doubleClicked).toBe(false);
  });

  it("preserves pressedKeys across clearFrameState", () => {
    manager.attach(container);
    fireKeyEvent("keydown", "Shift");
    manager.clearFrameState();

    const input = manager.getInput();
    expect(input.keyboard.pressedKeys.has("Shift")).toBe(true);
  });

  it("returns a copy of pressedKeys (not the internal set)", () => {
    manager.attach(container);
    fireKeyEvent("keydown", "q");

    const input1 = manager.getInput();
    const input2 = manager.getInput();
    expect(input1.keyboard.pressedKeys).not.toBe(input2.keyboard.pressedKeys);
  });

  it("captures mouse click events", () => {
    manager.attach(container);
    fireMouseEvent(container, "click");

    const input = manager.getInput();
    expect(input.mouse.clicked).toBe(true);
  });

  it("captures mouse double-click events", () => {
    manager.attach(container);
    fireMouseEvent(container, "dblclick");

    const input = manager.getInput();
    expect(input.mouse.doubleClicked).toBe(true);
  });

  it("mouse defaults are sensible", () => {
    manager.attach(container);
    const input = manager.getInput();
    expect(input.mouse.position).toEqual({ x: 0, y: 0 });
    expect(input.mouse.dragStart).toBeNull();
    expect(input.mouse.dragEnd).toBeNull();
    expect(input.mouse.isDragging).toBe(false);
  });

  it("detach is safe when not attached", () => {
    expect(() => manager.detach()).not.toThrow();
  });
});
