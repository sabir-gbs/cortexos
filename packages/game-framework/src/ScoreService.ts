import type { Difficulty, HighScoreEntry } from "./types";

/**
 * Manages the current game score and a per-difficulty high-score table.
 *
 * High scores are kept in memory only; callers can persist via
 * `getHighScores` / external storage if desired.
 */
export class ScoreService {
  private score = 0;
  private highScores: Map<string, HighScoreEntry[]> = new Map();
  private static MAX_PER_DIFFICULTY = 10;

  // ------------------------------------------------------------------
  // Score
  // ------------------------------------------------------------------

  getScore(): number {
    return this.score;
  }

  setScore(score: number): void {
    this.score = score;
  }

  addScore(points: number): void {
    this.score += points;
  }

  reset(): void {
    this.score = 0;
  }

  // ------------------------------------------------------------------
  // High scores
  // ------------------------------------------------------------------

  /** Return the high-score list for a difficulty, sorted descending. */
  getHighScores(difficulty: string): HighScoreEntry[] {
    const list = this.highScores.get(difficulty) ?? [];
    // Return a copy sorted by score descending.
    return [...list].sort((a, b) => b.score - a.score);
  }

  /**
   * Add a high-score entry. The list is trimmed to MAX_PER_DIFFICULTY
   * (keeping the highest scores). Duplicate entries (same player + score +
   * timestamp) are ignored.
   */
  addHighScore(entry: HighScoreEntry): void {
    const key = entry.difficulty;
    const list = this.highScores.get(key) ?? [];

    // De-duplicate.
    const isDuplicate = list.some(
      (e) =>
        e.playerName === entry.playerName &&
        e.score === entry.score &&
        e.achievedAt === entry.achievedAt,
    );
    if (isDuplicate) return;

    list.push(entry);
    list.sort((a, b) => b.score - a.score);

    if (list.length > ScoreService.MAX_PER_DIFFICULTY) {
      list.length = ScoreService.MAX_PER_DIFFICULTY;
    }

    this.highScores.set(key, list);
  }

  /** Return true if the given score would make it onto the board. */
  isHighScore(difficulty: string, score: number): boolean {
    const list = this.highScores.get(difficulty) ?? [];
    if (list.length < ScoreService.MAX_PER_DIFFICULTY) return true;
    // The list is sorted descending, so the last entry is the lowest.
    return score > list[list.length - 1].score;
  }

  // ------------------------------------------------------------------
  // Utility
  // ------------------------------------------------------------------

  /** Export all high scores for persistence. */
  exportAll(): Map<string, HighScoreEntry[]> {
    return new Map(this.highScores);
  }

  /** Import high scores (e.g. from localStorage). */
  importAll(data: Map<string, HighScoreEntry[]>): void {
    this.highScores = new Map(data);
  }
}
