import React, { useEffect, useRef } from "react";
import type { ContextMenuState, ContextMenuItem } from "../types";

interface ContextMenuProps {
  menu: ContextMenuState;
  onClose: () => void;
}

export function ContextMenu({ menu, onClose }: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    }
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [onClose]);

  // Clamp position to viewport
  const x = Math.min(menu.x, window.innerWidth - 200);
  const y = Math.min(menu.y, window.innerHeight - menu.items.length * 32 - 20);

  return (
    <div
      ref={menuRef}
      className="context-menu"
      style={{ left: x, top: y }}
      role="menu"
      aria-label="Context menu"
    >
      {menu.items.map((item) =>
        item.separator ? (
          <div key={item.id} className="context-menu__separator" role="separator" />
        ) : (
          <button
            key={item.id}
            className="context-menu__item"
            role="menuitem"
            disabled={item.disabled}
            onClick={() => {
              item.action?.();
              onClose();
            }}
          >
            {item.icon && (
              <span className="context-menu__item-icon" aria-hidden="true">
                {item.icon}
              </span>
            )}
            <span className="context-menu__item-label">{item.label}</span>
          </button>
        ),
      )}
    </div>
  );
}

/** Helper to build desktop context menu items. */
export function buildDesktopContextMenu(
  onChangeBackground?: () => void,
  onDisplaySettings?: () => void,
  onSortByName?: () => void,
  onRefresh?: () => void,
): ContextMenuItem[] {
  return [
    { id: "new-shortcut", label: "New Shortcut", icon: "\u2795", action: () => {} },
    { id: "sort-name", label: "Sort Icons By Name", icon: "\uD83D\uDDC3", action: onSortByName },
    { id: "sep-1", label: "", separator: true },
    {
      id: "change-bg",
      label: "Change Background",
      icon: "\uD83C\uDFA8",
      action: onChangeBackground,
    },
    {
      id: "display-settings",
      label: "Display Settings",
      icon: "\uD83D\uDDA5",
      action: onDisplaySettings,
    },
    { id: "sep-2", label: "", separator: true },
    { id: "refresh", label: "Refresh", icon: "\uD83D\uDD04", action: onRefresh },
  ];
}

/** Helper to build desktop icon context menu items. */
export function buildIconContextMenu(
  iconLabel: string,
  onOpen?: () => void,
  onRename?: () => void,
  onDelete?: () => void,
): ContextMenuItem[] {
  return [
    { id: "open", label: "Open", icon: "\uD83D\uDCC2", action: onOpen },
    { id: "sep-1", label: "", separator: true },
    { id: "rename", label: "Rename", icon: "\u270F\uFE0F", action: onRename },
    { id: "delete", label: `Delete "${iconLabel}"`, icon: "\uD83D\uDDD1", action: onDelete },
  ];
}
