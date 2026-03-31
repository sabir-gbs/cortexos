import { describe, it, expect, beforeEach } from "vitest";
import { ScoreService } from "../ScoreService";
import type { HighScoreEntry } from "../types";

function entry(score: number, name = "Player", difficulty = "medium"): HighScoreEntry {
  return { playerName: name, score, difficulty: difficulty as HighScoreEntry["difficulty"], achievedAt: Date.now() };
}

describe("ScoreService", () => {
  let service: ScoreService;

  beforeEach(() => {
    service = new ScoreService();
  });

  // ----------------------------------------------------------------
  // Score
  // ----------------------------------------------------------------

  it("starts at 0", () => {
    expect(service.getScore()).toBe(0);
  });

  it("setScore sets the score", () => {
    service.setScore(42);
    expect(service.getScore()).toBe(42);
  });

  it("addScore adds to the current score", () => {
    service.addScore(10);
    service.addScore(5);
    expect(service.getScore()).toBe(15);
  });

  it("addScore works with negative values", () => {
    service.setScore(20);
    service.addScore(-5);
    expect(service.getScore()).toBe(15);
  });

  it("reset brings score back to 0", () => {
    service.addScore(100);
    service.reset();
    expect(service.getScore()).toBe(0);
  });

  // ----------------------------------------------------------------
  // High scores
  // ----------------------------------------------------------------

  it("returns empty list for unknown difficulty", () => {
    expect(service.getHighScores("easy")).toEqual([]);
  });

  it("stores and retrieves high scores", () => {
    service.addHighScore(entry(100));
    const scores = service.getHighScores("medium");
    expect(scores).toHaveLength(1);
    expect(scores[0].score).toBe(100);
  });

  it("sorts high scores descending", () => {
    service.addHighScore(entry(50));
    service.addHighScore(entry(200));
    service.addHighScore(entry(100));
    const scores = service.getHighScores("medium");
    expect(scores.map((s) => s.score)).toEqual([200, 100, 50]);
  });

  it("limits high scores to MAX_PER_DIFFICULTY", () => {
    for (let i = 1; i <= 15; i++) {
      service.addHighScore(entry(i * 10, `P${i}`));
    }
    const scores = service.getHighScores("medium");
    expect(scores).toHaveLength(10);
    // Highest 10 scores kept.
    expect(scores[0].score).toBe(150);
    expect(scores[9].score).toBe(60);
  });

  it("high scores are per-difficulty", () => {
    service.addHighScore(entry(100, "A", "easy"));
    service.addHighScore(entry(200, "B", "hard"));
    expect(service.getHighScores("easy")).toHaveLength(1);
    expect(service.getHighScores("hard")).toHaveLength(1);
    expect(service.getHighScores("medium")).toHaveLength(0);
  });

  it("isHighScore returns true when list is not full", () => {
    expect(service.isHighScore("medium", 1)).toBe(true);
  });

  it("isHighScore returns true when score beats the lowest", () => {
    for (let i = 1; i <= 10; i++) {
      service.addHighScore(entry(i * 100));
    }
    // Lowest on the board is 100.  Score of 200 should make it.
    expect(service.isHighScore("medium", 200)).toBe(true);
  });

  it("isHighScore returns false when score does not beat the lowest", () => {
    for (let i = 1; i <= 10; i++) {
      service.addHighScore(entry(i * 100));
    }
    expect(service.isHighScore("medium", 50)).toBe(false);
  });

  it("ignores duplicate entries", () => {
    const e = entry(100);
    service.addHighScore(e);
    service.addHighScore(e);
    expect(service.getHighScores("medium")).toHaveLength(1);
  });

  it("exportAll / importAll round-trips data", () => {
    service.addHighScore(entry(100, "A", "easy"));
    service.addHighScore(entry(200, "B", "hard"));

    const exported = service.exportAll();
    const newService = new ScoreService();
    newService.importAll(exported);

    expect(newService.getHighScores("easy")).toHaveLength(1);
    expect(newService.getHighScores("hard")).toHaveLength(1);
  });
});
