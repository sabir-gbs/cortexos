import type { GameInput } from "./types";

type EventWithCoords = MouseEvent | PointerEvent;

/**
 * Captures keyboard and mouse events from a DOM element and exposes a
 * per-frame snapshot through `getInput()`.
 *
 * Call `clearFrameState()` at the end of each frame (or beginning of the
 * next) so that "this frame" flags reset.
 */
export class InputManager {
  private pressedKeys: Set<string> = new Set();
  private keyPressedThisFrame: string[] = [];
  private keyReleasedThisFrame: string[] = [];

  private mousePosition = { x: 0, y: 0 };
  private clickedThisFrame = false;
  private rightClickedThisFrame = false;
  private doubleClickedThisFrame = false;
  private dragStart: { x: number; y: number } | null = null;
  private dragEnd: { x: number; y: number } | null = null;
  private isDragging = false;

  private element: HTMLElement | null = null;

  // Bound handlers (so we can removeEventListener later).
  private boundKeyDown = this.onKeyDown.bind(this);
  private boundKeyUp = this.onKeyUp.bind(this);
  private boundMouseMove = this.onMouseMove.bind(this);
  private boundMouseDown = this.onMouseDown.bind(this);
  private boundMouseUp = this.onMouseUp.bind(this);
  private boundClick = this.onClick.bind(this);
  private boundContextMenu = this.onContextMenu.bind(this);
  private boundDblClick = this.onDblClick.bind(this);

  // ------------------------------------------------------------------
  // Public API
  // ------------------------------------------------------------------

  /** Start listening for events on the given element. */
  attach(element: HTMLElement): void {
    if (this.element) this.detach();
    this.element = element;

    // Keyboard – listen on window so keys are captured even when the
    // element itself isn't focused.
    window.addEventListener("keydown", this.boundKeyDown);
    window.addEventListener("keyup", this.boundKeyUp);

    // Mouse – listen on the element for position-relative coords.
    element.addEventListener("mousemove", this.boundMouseMove);
    element.addEventListener("mousedown", this.boundMouseDown);
    element.addEventListener("mouseup", this.boundMouseUp);
    element.addEventListener("click", this.boundClick);
    element.addEventListener("contextmenu", this.boundContextMenu);
    element.addEventListener("dblclick", this.boundDblClick);
  }

  /** Remove all event listeners. */
  detach(): void {
    if (!this.element) return;

    window.removeEventListener("keydown", this.boundKeyDown);
    window.removeEventListener("keyup", this.boundKeyUp);

    this.element.removeEventListener("mousemove", this.boundMouseMove);
    this.element.removeEventListener("mousedown", this.boundMouseDown);
    this.element.removeEventListener("mouseup", this.boundMouseUp);
    this.element.removeEventListener("click", this.boundClick);
    this.element.removeEventListener("contextmenu", this.boundContextMenu);
    this.element.removeEventListener("dblclick", this.boundDblClick);

    this.element = null;
  }

  /** Return the current input snapshot. */
  getInput(): GameInput {
    return {
      keyboard: {
        pressedKeys: new Set(this.pressedKeys),
        keyPressedThisFrame: [...this.keyPressedThisFrame],
        keyReleasedThisFrame: [...this.keyReleasedThisFrame],
      },
      mouse: {
        position: { ...this.mousePosition },
        clicked: this.clickedThisFrame,
        rightClicked: this.rightClickedThisFrame,
        doubleClicked: this.doubleClickedThisFrame,
        dragStart: this.dragStart ? { ...this.dragStart } : null,
        dragEnd: this.dragEnd ? { ...this.dragEnd } : null,
        isDragging: this.isDragging,
      },
    };
  }

  /** Reset per-frame flags. Call at the end of every game tick. */
  clearFrameState(): void {
    this.keyPressedThisFrame = [];
    this.keyReleasedThisFrame = [];
    this.clickedThisFrame = false;
    this.rightClickedThisFrame = false;
    this.doubleClickedThisFrame = false;
    // Keep dragEnd around for one frame so consumers can read it, then clear.
    // It will be nulled at the start of the *next* clearFrame if dragging is
    // no longer happening.
    if (!this.isDragging) {
      this.dragEnd = null;
    }
  }

  // ------------------------------------------------------------------
  // Private handlers
  // ------------------------------------------------------------------

  private onKeyDown(e: KeyboardEvent): void {
    const key = e.key;
    if (!this.pressedKeys.has(key)) {
      this.pressedKeys.add(key);
      this.keyPressedThisFrame.push(key);
    }
  }

  private onKeyUp(e: KeyboardEvent): void {
    const key = e.key;
    this.pressedKeys.delete(key);
    this.keyReleasedThisFrame.push(key);
  }

  private getRelativeCoords(e: EventWithCoords): { x: number; y: number } {
    if (!this.element) return { x: e.clientX, y: e.clientY };
    const rect = this.element.getBoundingClientRect();
    return {
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    };
  }

  private onMouseMove(e: MouseEvent): void {
    this.mousePosition = this.getRelativeCoords(e);
    if (this.isDragging) {
      // Drag continues; keep dragStart unchanged.
    }
  }

  private onMouseDown(e: MouseEvent): void {
    const pos = this.getRelativeCoords(e);
    if (e.button === 0) {
      this.dragStart = pos;
      this.isDragging = false; // not a drag yet until mouse moves
    }
  }

  private onMouseUp(e: MouseEvent): void {
    if (e.button === 0 && this.dragStart) {
      const pos = this.getRelativeCoords(e);
      const dx = pos.x - this.dragStart.x;
      const dy = pos.y - this.dragStart.y;
      const distance = Math.sqrt(dx * dx + dy * dy);
      if (distance > 5) {
        this.isDragging = false;
        this.dragEnd = pos;
      }
      // If distance <= 5 it was just a click, not a drag.
      this.dragStart = null;
    }
  }

  private onClick(_e: MouseEvent): void {
    this.clickedThisFrame = true;
  }

  private onContextMenu(e: MouseEvent): void {
    e.preventDefault();
    this.rightClickedThisFrame = true;
  }

  private onDblClick(_e: MouseEvent): void {
    this.doubleClickedThisFrame = true;
  }
}
