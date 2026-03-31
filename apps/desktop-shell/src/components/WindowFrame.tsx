import React, { useCallback, useRef, useState, useEffect } from "react";
import type { WindowState } from "../types";

interface WindowFrameProps {
  window: WindowState;
  entryPoint?: string;
  onFocus: (id: string) => void;
  onClose: (id: string) => void;
  onMinimize: (id: string) => void;
  onMaximize: (id: string) => void;
  onMove: (id: string, x: number, y: number) => void;
  onResize: (id: string, w: number, h: number) => void;
}

export function WindowFrame({
  window: win,
  entryPoint,
  onFocus,
  onClose,
  onMinimize,
  onMaximize,
  onMove,
  onResize,
}: WindowFrameProps) {
  const [pos, setPos] = useState({ x: win.x, y: win.y });
  const [size, setSize] = useState({ w: win.width, h: win.height });
  const [maximized, setMaximized] = useState(win.state === "maximized");
  const [preMaxState, setPreMaxState] = useState({
    x: win.x,
    y: win.y,
    w: win.width,
    h: win.height,
  });
  const dragRef = useRef<{ startX: number; startY: number; origX: number; origY: number } | null>(
    null,
  );
  const resizeRef = useRef<{
    startX: number;
    startY: number;
    origW: number;
    origH: number;
    origX: number;
    origY: number;
    dir: string;
  } | null>(null);

  useEffect(() => {
    setPos({ x: win.x, y: win.y });
    setSize({ w: win.width, h: win.height });
    setMaximized(win.state === "maximized");
  }, [win.x, win.y, win.width, win.height, win.state]);

  // Drag handlers
  const handleTitlePointerDown = useCallback(
    (e: React.PointerEvent) => {
      if (e.button !== 0 || maximized) return;
      e.preventDefault();
      (e.target as HTMLElement).setPointerCapture(e.pointerId);
      dragRef.current = { startX: e.clientX, startY: e.clientY, origX: pos.x, origY: pos.y };
    },
    [pos.x, pos.y, maximized],
  );

  const handleTitlePointerMove = useCallback((e: React.PointerEvent) => {
    if (!dragRef.current) return;
    const dx = e.clientX - dragRef.current.startX;
    const dy = e.clientY - dragRef.current.startY;
    setPos({ x: dragRef.current.origX + dx, y: dragRef.current.origY + dy });
  }, []);

  const handleTitlePointerUp = useCallback(() => {
    if (!dragRef.current) return;
    const newPos = { x: pos.x, y: pos.y };
    onMove(win.id, newPos.x, newPos.y);
    dragRef.current = null;
  }, [pos, win.id, onMove]);

  // Resize handlers
  const handleResizePointerDown = useCallback(
    (e: React.PointerEvent, dir: string) => {
      if (e.button !== 0 || maximized) return;
      e.preventDefault();
      e.stopPropagation();
      (e.target as HTMLElement).setPointerCapture(e.pointerId);
      resizeRef.current = {
        startX: e.clientX,
        startY: e.clientY,
        origW: size.w,
        origH: size.h,
        origX: pos.x,
        origY: pos.y,
        dir,
      };
    },
    [size.w, size.h, pos.x, pos.y, maximized],
  );

  const handleResizePointerMove = useCallback((e: React.PointerEvent) => {
    if (!resizeRef.current) return;
    const dx = e.clientX - resizeRef.current.startX;
    const dy = e.clientY - resizeRef.current.startY;
    const dir = resizeRef.current.dir;
    const minW = 200;
    const minH = 100;

    let newW = resizeRef.current.origW;
    let newH = resizeRef.current.origH;
    let newX = resizeRef.current.origX;
    let newY = resizeRef.current.origY;

    if (dir.includes("e")) newW = Math.max(minW, resizeRef.current.origW + dx);
    if (dir.includes("s")) newH = Math.max(minH, resizeRef.current.origH + dy);
    if (dir.includes("w")) {
      newW = Math.max(minW, resizeRef.current.origW - dx);
      if (newW > minW) newX = resizeRef.current.origX + dx;
    }
    if (dir.includes("n")) {
      newH = Math.max(minH, resizeRef.current.origH - dy);
      if (newH > minH) newY = resizeRef.current.origY + dy;
    }

    setSize({ w: newW, h: newH });
    setPos({ x: newX, y: newY });
  }, []);

  const handleResizePointerUp = useCallback(() => {
    if (!resizeRef.current) return;
    onResize(win.id, size.w, size.h);
    onMove(win.id, pos.x, pos.y);
    resizeRef.current = null;
  }, [win.id, size, pos, onResize, onMove]);

  const handleMaximizeToggle = useCallback(() => {
    if (maximized) {
      setPos({ x: preMaxState.x, y: preMaxState.y });
      setSize({ w: preMaxState.w, h: preMaxState.h });
      setMaximized(false);
    } else {
      setPreMaxState({ x: pos.x, y: pos.y, w: size.w, h: size.h });
      setPos({ x: 0, y: 0 });
      const desktop = document.querySelector(".desktop") as HTMLElement;
      setSize({
        w: desktop?.clientWidth || window.innerWidth,
        h: desktop?.clientHeight || window.innerHeight - 48,
      });
      setMaximized(true);
    }
    onMaximize(win.id);
  }, [maximized, preMaxState, pos, size, win.id, onMaximize]);

  if (win.state === "minimized" || win.state === "closed") return null;

  const isMax = maximized;
  const style: React.CSSProperties = {
    position: "absolute",
    left: isMax ? 0 : pos.x,
    top: isMax ? 0 : pos.y,
    width: isMax ? "100%" : size.w,
    height: isMax ? "100%" : size.h,
    zIndex: win.z_index + 100,
  };

  return (
    <div
      className={`window-frame${win.focused ? " focused" : ""}`}
      style={style}
      onMouseDown={() => onFocus(win.id)}
      role="dialog"
      aria-label={win.title}
    >
      {/* Resize handles */}
      {!isMax && (
        <>
          <div
            className="resize-handle n"
            onPointerDown={(e) => handleResizePointerDown(e, "n")}
            onPointerMove={handleResizePointerMove}
            onPointerUp={handleResizePointerUp}
          />
          <div
            className="resize-handle s"
            onPointerDown={(e) => handleResizePointerDown(e, "s")}
            onPointerMove={handleResizePointerMove}
            onPointerUp={handleResizePointerUp}
          />
          <div
            className="resize-handle e"
            onPointerDown={(e) => handleResizePointerDown(e, "e")}
            onPointerMove={handleResizePointerMove}
            onPointerUp={handleResizePointerUp}
          />
          <div
            className="resize-handle w"
            onPointerDown={(e) => handleResizePointerDown(e, "w")}
            onPointerMove={handleResizePointerMove}
            onPointerUp={handleResizePointerUp}
          />
          <div
            className="resize-handle ne"
            onPointerDown={(e) => handleResizePointerDown(e, "ne")}
            onPointerMove={handleResizePointerMove}
            onPointerUp={handleResizePointerUp}
          />
          <div
            className="resize-handle nw"
            onPointerDown={(e) => handleResizePointerDown(e, "nw")}
            onPointerMove={handleResizePointerMove}
            onPointerUp={handleResizePointerUp}
          />
          <div
            className="resize-handle se"
            onPointerDown={(e) => handleResizePointerDown(e, "se")}
            onPointerMove={handleResizePointerMove}
            onPointerUp={handleResizePointerUp}
          />
          <div
            className="resize-handle sw"
            onPointerDown={(e) => handleResizePointerDown(e, "sw")}
            onPointerMove={handleResizePointerMove}
            onPointerUp={handleResizePointerUp}
          />
        </>
      )}

      {/* Title bar */}
      <div
        className="window-titlebar"
        onPointerDown={handleTitlePointerDown}
        onPointerMove={handleTitlePointerMove}
        onPointerUp={handleTitlePointerUp}
        onDoubleClick={handleMaximizeToggle}
      >
        <span className="window-titlebar__title">{win.title}</span>
        <div className="window-titlebar__controls">
          <button
            className="window-btn window-btn--minimize"
            onClick={(e) => {
              e.stopPropagation();
              onMinimize(win.id);
            }}
            aria-label="Minimize"
            title="Minimize"
          >
            &#x2500;
          </button>
          <button
            className="window-btn window-btn--maximize"
            onClick={(e) => {
              e.stopPropagation();
              handleMaximizeToggle();
            }}
            aria-label={maximized ? "Restore" : "Maximize"}
            title={maximized ? "Restore" : "Maximize"}
          >
            {maximized ? "\u2750" : "\u25A1"}
          </button>
          <button
            className="window-btn window-btn--close"
            onClick={(e) => {
              e.stopPropagation();
              onClose(win.id);
            }}
            aria-label="Close"
            title="Close"
          >
            &#x2715;
          </button>
        </div>
      </div>

      {/* Window content */}
      <div className="window-content">
        {entryPoint ? (
          <iframe
            src={entryPoint}
            className="window-iframe"
            title={win.title}
            sandbox="allow-scripts allow-same-origin allow-forms allow-popups allow-modals"
            allow="clipboard-read; clipboard-write"
          />
        ) : (
          <div className="window-placeholder">
            <span className="window-placeholder__icon" aria-hidden="true">
              &#x1F4BB;
            </span>
            <span>{win.title}</span>
            <span className="window-placeholder__hint">No entry point configured for this app</span>
          </div>
        )}
      </div>
    </div>
  );
}
