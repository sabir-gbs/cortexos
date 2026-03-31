import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import CommandPalette from "../components/CommandPalette";
import { BUILTIN_APPS } from "../types";

// Mock the API module
vi.mock("../api", () => ({
  search: vi.fn().mockResolvedValue([]),
}));

import * as api from "../api";

const mockProps = {
  onClose: vi.fn(),
  onLaunchApp: vi.fn(),
  onOpenSetting: vi.fn(),
  apps: BUILTIN_APPS,
};

describe("CommandPalette", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.search).mockResolvedValue([]);
  });

  it("renders the search input", () => {
    render(<CommandPalette {...mockProps} />);
    expect(screen.getByRole("combobox")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("Type a command or search...")).toBeInTheDocument();
  });

  it("renders app results by default", async () => {
    render(<CommandPalette {...mockProps} />);
    await waitFor(() => {
      expect(screen.getByText("Calculator")).toBeInTheDocument();
    });
  });

  it("renders builtin commands", async () => {
    render(<CommandPalette {...mockProps} />);
    await waitFor(() => {
      expect(screen.getByText("Toggle Theme")).toBeInTheDocument();
    });
  });

  it("filters results based on query", async () => {
    render(<CommandPalette {...mockProps} />);
    const input = screen.getByRole("combobox");
    await act(async () => {
      fireEvent.change(input, { target: { value: "calc" } });
    });

    await waitFor(() => {
      // Title text is split by <mark> highlight tags, match by class
      const titles = document.querySelectorAll(".palette-result-title");
      expect(titles.length).toBeGreaterThanOrEqual(1);
      // The title should contain "Calculator" (possibly split across mark + text nodes)
      const hasMatch = Array.from(titles).some((el) => el.textContent?.includes("Calculator"));
      expect(hasMatch).toBe(true);
    });
  });

  it("shows no results message for unmatched query", async () => {
    vi.mocked(api.search).mockResolvedValue([]);
    render(<CommandPalette {...mockProps} />);
    const input = screen.getByRole("combobox");
    await act(async () => {
      fireEvent.change(input, { target: { value: "zzzzz-nonexistent" } });
    });

    await waitFor(() => {
      expect(screen.getByText("No results found")).toBeInTheDocument();
    });
  });

  it("calls onLaunchApp when an app item is clicked", async () => {
    render(<CommandPalette {...mockProps} />);
    await waitFor(() => {
      expect(screen.getByText("Calculator")).toBeInTheDocument();
    });
    fireEvent.click(screen.getByText("Calculator"));
    expect(mockProps.onLaunchApp).toHaveBeenCalledWith("calculator");
    expect(mockProps.onClose).toHaveBeenCalled();
  });

  it("closes on Escape key", () => {
    render(<CommandPalette {...mockProps} />);
    const input = screen.getByRole("combobox");
    fireEvent.keyDown(input, { key: "Escape" });
    expect(mockProps.onClose).toHaveBeenCalled();
  });

  it("navigates with arrow keys", async () => {
    render(<CommandPalette {...mockProps} />);
    await waitFor(() => {
      expect(screen.getByText("Calculator")).toBeInTheDocument();
    });

    const input = screen.getByRole("combobox");
    fireEvent.keyDown(input, { key: "ArrowDown" });

    // The selected item should have aria-selected="true"
    const items = screen.getAllByRole("option");
    const selectedItem = items.find((el) => el.getAttribute("aria-selected") === "true");
    expect(selectedItem).toBeTruthy();
  });

  it("executes selected item on Enter", async () => {
    render(<CommandPalette {...mockProps} />);
    await waitFor(() => {
      expect(screen.getByText("Calculator")).toBeInTheDocument();
    });

    const input = screen.getByRole("combobox");
    fireEvent.keyDown(input, { key: "Enter" });
    expect(mockProps.onClose).toHaveBeenCalled();
  });

  it("renders footer with keyboard hints", () => {
    render(<CommandPalette {...mockProps} />);
    expect(screen.getByText(/Navigate/)).toBeInTheDocument();
    expect(screen.getByText(/Select/)).toBeInTheDocument();
    expect(screen.getByText(/Close/)).toBeInTheDocument();
  });

  it("calls search API when typing a query", async () => {
    render(<CommandPalette {...mockProps} />);
    const input = screen.getByRole("combobox");
    await act(async () => {
      fireEvent.change(input, { target: { value: "test query" } });
    });

    await waitFor(() => {
      expect(api.search).toHaveBeenCalledWith("test query", 20);
    });
  });
});
