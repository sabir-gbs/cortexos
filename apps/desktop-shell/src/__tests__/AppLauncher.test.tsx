import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";
import { AppLauncher } from "../components/AppLauncher";
import type { AppManifest } from "../types";

const mockApps: AppManifest[] = [
  {
    id: "calculator",
    name: "Calculator",
    description: "Basic calculator",
    category: "Utilities",
    icon: "\uD83E\uDDEE",
    version: "1.0.0",
  },
  {
    id: "notes",
    name: "Notes",
    description: "Take notes",
    category: "Productivity",
    icon: "\uD83D\uDCD3",
    version: "1.0.0",
  },
  {
    id: "snake",
    name: "Snake",
    description: "Classic snake game",
    category: "Games",
    icon: "\uD83D\uDC0D",
    version: "1.0.0",
  },
];

describe("AppLauncher", () => {
  it("renders search input", () => {
    render(<AppLauncher apps={mockApps} onLaunch={vi.fn()} onClose={vi.fn()} />);
    expect(screen.getByPlaceholderText(/search apps/i)).toBeInTheDocument();
  });

  it("renders all apps by default", () => {
    render(<AppLauncher apps={mockApps} onLaunch={vi.fn()} onClose={vi.fn()} />);
    expect(screen.getByText("Calculator")).toBeInTheDocument();
    expect(screen.getByText("Notes")).toBeInTheDocument();
    expect(screen.getByText("Snake")).toBeInTheDocument();
  });

  it("filters apps by search query", () => {
    render(<AppLauncher apps={mockApps} onLaunch={vi.fn()} onClose={vi.fn()} />);
    const search = screen.getByPlaceholderText(/search apps/i);
    fireEvent.change(search, { target: { value: "calc" } });
    expect(screen.getByText("Calculator")).toBeInTheDocument();
    expect(screen.queryByText("Notes")).not.toBeInTheDocument();
    expect(screen.queryByText("Snake")).not.toBeInTheDocument();
  });

  it("filters apps by description", () => {
    render(<AppLauncher apps={mockApps} onLaunch={vi.fn()} onClose={vi.fn()} />);
    const search = screen.getByPlaceholderText(/search apps/i);
    fireEvent.change(search, { target: { value: "snake game" } });
    expect(screen.getByText("Snake")).toBeInTheDocument();
  });

  it("shows empty state when no matches", () => {
    render(<AppLauncher apps={mockApps} onLaunch={vi.fn()} onClose={vi.fn()} />);
    const search = screen.getByPlaceholderText(/search apps/i);
    fireEvent.change(search, { target: { value: "nonexistent" } });
    expect(screen.getByText("No apps found")).toBeInTheDocument();
  });

  it("calls onLaunch when app is clicked", () => {
    const onLaunch = vi.fn();
    render(<AppLauncher apps={mockApps} onLaunch={onLaunch} onClose={vi.fn()} />);
    fireEvent.click(screen.getByText("Calculator"));
    expect(onLaunch).toHaveBeenCalledWith("calculator");
  });

  it("calls onClose on Escape", () => {
    const onClose = vi.fn();
    render(<AppLauncher apps={mockApps} onLaunch={vi.fn()} onClose={onClose} />);
    fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
  });

  it("renders category tabs", () => {
    render(<AppLauncher apps={mockApps} onLaunch={vi.fn()} onClose={vi.fn()} />);
    expect(screen.getByText("All")).toBeInTheDocument();
    expect(screen.getByText("Utilities")).toBeInTheDocument();
    expect(screen.getByText("Games")).toBeInTheDocument();
  });

  it("filters by category", () => {
    render(<AppLauncher apps={mockApps} onLaunch={vi.fn()} onClose={vi.fn()} />);
    fireEvent.click(screen.getByText("Games"));
    expect(screen.getByText("Snake")).toBeInTheDocument();
    expect(screen.queryByText("Calculator")).not.toBeInTheDocument();
  });
});
