import React, { useCallback, useRef, useState } from "react";
import type { DesktopIcon, GridPosition } from "../types";

const GRID_CELL = 96;

interface DesktopIconGridProps {
  icons: DesktopIcon[];
  onLaunch: (app: DesktopIcon) => void;
  onContextMenu: (e: React.MouseEvent, icon?: DesktopIcon) => void;
  onMoveIcon: (id: string, position: GridPosition) => void;
}

export function DesktopIconGrid({
  icons,
  onLaunch,
  onContextMenu,
  onMoveIcon,
}: DesktopIconGridProps) {
  const [dragging, setDragging] = useState<string | null>(null);
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });
  const [dragPos, setDragPos] = useState({ x: 0, y: 0 });
  const containerRef = useRef<HTMLDivElement>(null);

  const handlePointerDown = useCallback((e: React.PointerEvent, icon: DesktopIcon) => {
    if (e.button !== 0) return;
    e.preventDefault();
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
    setDragging(icon.id);
    setDragOffset({
      x: e.clientX - icon.position.column * GRID_CELL,
      y: e.clientY - icon.position.row * GRID_CELL,
    });
    setDragPos({
      x: icon.position.column * GRID_CELL,
      y: icon.position.row * GRID_CELL,
    });
  }, []);

  const handlePointerMove = useCallback(
    (e: React.PointerEvent) => {
      if (!dragging) return;
      setDragPos({
        x: e.clientX - dragOffset.x,
        y: e.clientY - dragOffset.y,
      });
    },
    [dragging, dragOffset],
  );

  const handlePointerUp = useCallback(() => {
    if (!dragging) return;
    const col = Math.max(0, Math.round(dragPos.x / GRID_CELL));
    const row = Math.max(0, Math.round(dragPos.y / GRID_CELL));
    onMoveIcon(dragging, { row, column: col });
    setDragging(null);
  }, [dragging, dragPos, onMoveIcon]);

  const handleDoubleClick = useCallback(
    (icon: DesktopIcon) => {
      onLaunch(icon);
    },
    [onLaunch],
  );

  const handleDesktopContextMenu = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      onContextMenu(e);
    },
    [onContextMenu],
  );

  const handleIconContextMenu = useCallback(
    (e: React.MouseEvent, icon: DesktopIcon) => {
      e.preventDefault();
      e.stopPropagation();
      onContextMenu(e, icon);
    },
    [onContextMenu],
  );

  return (
    <div
      ref={containerRef}
      className="desktop-icon-grid"
      onContextMenu={handleDesktopContextMenu}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      role="grid"
      aria-label="Desktop icons"
    >
      {icons.map((icon) => {
        const isDragging = dragging === icon.id;
        const x = isDragging ? dragPos.x : icon.position.column * GRID_CELL;
        const y = isDragging ? dragPos.y : icon.position.row * GRID_CELL;

        return (
          <div
            key={icon.id}
            className={`desktop-icon${isDragging ? " dragging" : ""}`}
            style={{
              transform: `translate(${x}px, ${y}px)`,
              width: GRID_CELL,
              height: GRID_CELL,
            }}
            onPointerDown={(e) => handlePointerDown(e, icon)}
            onDoubleClick={() => handleDoubleClick(icon)}
            onContextMenu={(e) => handleIconContextMenu(e, icon)}
            role="gridcell"
            aria-label={icon.label}
            tabIndex={0}
            onKeyDown={(e) => {
              if (e.key === "Enter") onLaunch(icon);
            }}
          >
            <span className="desktop-icon__icon" aria-hidden="true">
              {icon.icon_url}
            </span>
            <span className="desktop-icon__label">{icon.label}</span>
          </div>
        );
      })}
    </div>
  );
}
