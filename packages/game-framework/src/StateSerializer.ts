import type { GameStateBase } from "./types";

/**
 * JSON serialization / deserialization with schema-version validation.
 *
 * The wire format is `{ v: number, s: <state> }` so that consumers can
 * reject data produced by a newer version of the game.
 */
export class StateSerializer {
  /**
   * Serialize a game state to a JSON string.
   *
   * Handles Set objects by converting them to arrays (useful when a game
   * state contains Set fields).  Map objects are converted to arrays of
   * [key, value] pairs.
   */
  serialize(state: GameStateBase): string {
    const portable = this.makePortable(state);
    return JSON.stringify({ v: state.schemaVersion, s: portable });
  }

  /**
   * Deserialize a JSON string back into a game state.
   *
   * Returns `null` when:
   *  - The JSON is malformed.
   *  - The envelope is missing required fields.
   *  - The schema version does not match `expectedVersion`.
   */
  deserialize(json: string, expectedVersion: number): GameStateBase | null {
    try {
      const envelope: unknown = JSON.parse(json);
      if (
        typeof envelope !== "object" ||
        envelope === null ||
        !("v" in envelope) ||
        !("s" in envelope)
      ) {
        return null;
      }
      const { v, s } = envelope as { v: unknown; s: unknown };
      if (typeof v !== "number" || v !== expectedVersion) {
        return null;
      }
      // Basic shape validation.
      if (typeof s !== "object" || s === null) {
        return null;
      }
      return s as GameStateBase;
    } catch {
      return null;
    }
  }

  // ------------------------------------------------------------------
  // Internals
  // ------------------------------------------------------------------

  /**
   * Recursively convert non-JSON-safe values into JSON-safe equivalents.
   */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  private makePortable(value: any): any {
    if (value === null || value === undefined) return value;
    if (typeof value === "function" || typeof value === "symbol") return undefined;

    if (value instanceof Set) {
      return { __type: "Set", values: Array.from(value).map((v) => this.makePortable(v)) };
    }
    if (value instanceof Map) {
      return {
        __type: "Map",
        entries: Array.from(value.entries()).map(([k, v]) => [
          this.makePortable(k),
          this.makePortable(v),
        ]),
      };
    }

    if (Array.isArray(value)) {
      return value.map((item) => this.makePortable(item));
    }

    if (typeof value === "object") {
      const out: Record<string, unknown> = {};
      for (const key of Object.keys(value)) {
        out[key] = this.makePortable(value[key]);
      }
      return out;
    }

    return value;
  }
}
