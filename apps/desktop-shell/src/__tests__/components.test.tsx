import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

// ── ConnectionBanner ─────────────────────────────────────────────────────────

import { ConnectionBanner } from "../components/ConnectionBanner";

describe("ConnectionBanner", () => {
  it("renders nothing when connected", () => {
    const { container } = render(<ConnectionBanner connected={true} />);
    expect(container.innerHTML).toBe("");
  });

  it("renders banner when disconnected", () => {
    render(<ConnectionBanner connected={false} />);
    expect(screen.getByRole("alert")).toHaveTextContent("Connection lost");
  });
});

// ── DesktopBackground ────────────────────────────────────────────────────────

import { DesktopBackground } from "../components/DesktopBackground";

describe("DesktopBackground", () => {
  it("renders solid background", () => {
    render(<DesktopBackground background={{ type: "solid", color: "#ff0000" }} />);
    const el = document.querySelector(".desktop-background") as HTMLElement;
    expect(el).toBeTruthy();
    expect(el.style.backgroundColor).toBe("rgb(255, 0, 0)");
  });

  it("renders gradient background", () => {
    render(
      <DesktopBackground
        background={{ type: "gradient", from: "#ff0000", to: "#0000ff", direction: "90deg" }}
      />,
    );
    const el = document.querySelector(".desktop-background") as HTMLElement;
    expect(el.style.background).toContain("linear-gradient");
  });
});

// ── ContextMenu ──────────────────────────────────────────────────────────────

import { ContextMenu } from "../components/ContextMenu";
import type { ContextMenuState } from "../types";

describe("ContextMenu", () => {
  const menu: ContextMenuState = {
    x: 100,
    y: 200,
    items: [
      { id: "open", label: "Open", action: vi.fn() },
      { id: "sep", label: "", separator: true },
      { id: "delete", label: "Delete", action: vi.fn() },
    ],
  };

  it("renders menu items", () => {
    render(<ContextMenu menu={menu} onClose={vi.fn()} />);
    expect(screen.getByText("Open")).toBeInTheDocument();
    expect(screen.getByText("Delete")).toBeInTheDocument();
  });

  it("calls action on item click and closes", () => {
    const onClose = vi.fn();
    const action = vi.fn();
    const menuWithAction: ContextMenuState = {
      x: 100,
      y: 200,
      items: [{ id: "test", label: "Test Item", action }],
    };
    render(<ContextMenu menu={menuWithAction} onClose={onClose} />);
    fireEvent.click(screen.getByText("Test Item"));
    expect(action).toHaveBeenCalled();
    expect(onClose).toHaveBeenCalled();
  });

  it("closes on Escape", () => {
    const onClose = vi.fn();
    render(<ContextMenu menu={menu} onClose={onClose} />);
    fireEvent.keyDown(document, { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
  });
});

// ── DesktopIconGrid ──────────────────────────────────────────────────────────

import { DesktopIconGrid } from "../components/DesktopIconGrid";
import type { DesktopIcon } from "../types";

const testIcons: DesktopIcon[] = [
  {
    id: "i1",
    app_id: "calculator",
    file_path: null,
    label: "Calculator",
    icon_url: "\uD83E\uDDEE",
    position: { row: 0, column: 0 },
    created_at: "",
  },
  {
    id: "i2",
    app_id: "notes",
    file_path: null,
    label: "Notes",
    icon_url: "\uD83D\uDCD3",
    position: { row: 1, column: 0 },
    created_at: "",
  },
];

describe("DesktopIconGrid", () => {
  it("renders icons with labels", () => {
    render(
      <DesktopIconGrid
        icons={testIcons}
        onLaunch={vi.fn()}
        onContextMenu={vi.fn()}
        onMoveIcon={vi.fn()}
      />,
    );
    expect(screen.getByText("Calculator")).toBeInTheDocument();
    expect(screen.getByText("Notes")).toBeInTheDocument();
  });

  it("calls onLaunch on double-click", () => {
    const onLaunch = vi.fn();
    render(
      <DesktopIconGrid
        icons={testIcons}
        onLaunch={onLaunch}
        onContextMenu={vi.fn()}
        onMoveIcon={vi.fn()}
      />,
    );
    fireEvent.doubleClick(screen.getByText("Calculator").closest('[role="gridcell"]')!);
    expect(onLaunch).toHaveBeenCalledWith(testIcons[0]);
  });

  it("renders grid with proper ARIA role", () => {
    render(
      <DesktopIconGrid
        icons={testIcons}
        onLaunch={vi.fn()}
        onContextMenu={vi.fn()}
        onMoveIcon={vi.fn()}
      />,
    );
    expect(screen.getByRole("grid", { name: /desktop icons/i })).toBeInTheDocument();
  });
});

// ── SettingsPanel ────────────────────────────────────────────────────────────

import { SettingsPanel } from "../components/SettingsPanel";
import type { ShellSettings } from "../types";
import { DEFAULT_SETTINGS } from "../types";

describe("SettingsPanel", () => {
  it("renders settings panel with tabs", () => {
    render(
      <SettingsPanel settings={DEFAULT_SETTINGS} onUpdateSettings={vi.fn()} onClose={vi.fn()} />,
    );
    expect(screen.getByText("Appearance")).toBeInTheDocument();
    expect(screen.getByText("Taskbar")).toBeInTheDocument();
    expect(screen.getByText("Desktop")).toBeInTheDocument();
    expect(screen.getByText("Clock")).toBeInTheDocument();
  });

  it("calls onClose when close button clicked", () => {
    const onClose = vi.fn();
    render(
      <SettingsPanel settings={DEFAULT_SETTINGS} onUpdateSettings={vi.fn()} onClose={onClose} />,
    );
    fireEvent.click(screen.getByLabelText(/close settings/i));
    expect(onClose).toHaveBeenCalled();
  });
});
