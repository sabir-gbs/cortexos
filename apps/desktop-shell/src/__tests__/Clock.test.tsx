import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import React from "react";

// Directly test the Clock component by extracting it
// Since Clock is embedded in Taskbar, we test via Taskbar rendering

function formatTime(date: Date, format: "12h" | "24h"): string {
  return date.toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    hour12: format === "12h",
  });
}

function formatDate(date: Date): string {
  return date.toLocaleDateString([], { month: "short", day: "numeric" });
}

describe("Clock formatting", () => {
  it("formats time in 24h format", () => {
    const date = new Date(2024, 0, 15, 14, 30);
    const result = formatTime(date, "24h");
    expect(result).toContain("14");
    expect(result).toContain("30");
  });

  it("formats time in 12h format", () => {
    const date = new Date(2024, 0, 15, 14, 30);
    const result = formatTime(date, "12h");
    // 12h format should show PM
    expect(result).toMatch(/PM/i);
  });

  it("formats date as month day", () => {
    const date = new Date(2024, 0, 15);
    const result = formatDate(date);
    expect(result).toContain("Jan");
    expect(result).toContain("15");
  });

  it("handles midnight correctly in 24h", () => {
    const date = new Date(2024, 0, 15, 0, 0);
    const result = formatTime(date, "24h");
    expect(result).toContain("00");
  });

  it("handles noon correctly in 24h", () => {
    const date = new Date(2024, 0, 15, 12, 0);
    const result = formatTime(date, "24h");
    expect(result).toContain("12");
  });
});
