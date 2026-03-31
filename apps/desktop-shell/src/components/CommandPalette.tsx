/**
 * Global Command Palette for CortexOS.
 *
 * Triggered by Ctrl+Space. Provides instant search across apps, commands,
 * files, and settings with keyboard navigation and real-time filtering.
 */

import { useState, useEffect, useRef, useCallback } from "react";
import type { SearchResult, AppManifest } from "../types";
import * as api from "../api";

interface PaletteProps {
  onClose: () => void;
  onLaunchApp: (appId: string) => void;
  onOpenSetting: (key: string) => void;
  apps: AppManifest[];
}

/** Built-in commands always available in the palette. */
const BUILTIN_COMMANDS: PaletteItem[] = [
  {
    id: "cmd:toggle-theme",
    title: "Toggle Theme",
    subtitle: "Switch light/dark mode",
    type: "command",
    score: 1,
    action: { type: "execute_command", command_id: "toggle-theme" },
  },
  {
    id: "cmd:toggle-fullscreen",
    title: "Toggle Fullscreen",
    subtitle: "F11",
    type: "command",
    score: 1,
    action: { type: "execute_command", command_id: "toggle-fullscreen" },
  },
  {
    id: "cmd:new-workspace",
    title: "New Workspace",
    subtitle: "Create a new workspace",
    type: "command",
    score: 1,
    action: { type: "execute_command", command_id: "new-workspace" },
  },
  {
    id: "cmd:open-launcher",
    title: "Open App Launcher",
    subtitle: "Meta key",
    type: "command",
    score: 1,
    action: { type: "execute_command", command_id: "open-launcher" },
  },
  {
    id: "cmd:close-window",
    title: "Close Focused Window",
    subtitle: "Close current window",
    type: "command",
    score: 1,
    action: { type: "execute_command", command_id: "close-window" },
  },
];

type SearchAction =
  | { type: "open_app"; app_id: string }
  | { type: "open_file"; file_id: string; app_id: string }
  | { type: "open_setting"; key: string }
  | { type: "execute_command"; command_id: string };

interface PaletteItem {
  id: string;
  title: string;
  subtitle?: string;
  icon?: string;
  type: "app" | "command" | "file" | "setting";
  score: number;
  action: SearchAction;
}

/** Type priority weights for ranking. */
const TYPE_WEIGHT: Record<string, number> = {
  app: 1.5,
  command: 1.3,
  file: 1.0,
  setting: 0.8,
};

/** Normalize a search term for matching. */
function normalize(s: string): string {
  return s.toLowerCase().trim();
}

/** Check if query is a prefix match for title. */
function isPrefixMatch(title: string, query: string): boolean {
  const words = normalize(title).split(/\s+/);
  const q = normalize(query);
  return words.some((w) => w.startsWith(q));
}

/** Rank a local item by query. */
function rankItem(title: string, type: string, query: string): number {
  const q = normalize(query);
  const t = normalize(title);
  let base = 0;
  if (t === q) base = 1.0;
  else if (t.startsWith(q)) base = 0.9;
  else if (isPrefixMatch(title, query)) base = 0.7;
  else if (t.includes(q)) base = 0.5;

  // Prefix bonus
  const bonus = t.startsWith(q) ? 0.3 : 0;

  // Type weight
  const score = (base + bonus) * (TYPE_WEIGHT[type] ?? 1.0);

  return Math.min(score, 1.0);
}

export default function CommandPalette({
  onClose,
  onLaunchApp,
  onOpenSetting,
  apps,
}: PaletteProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<PaletteItem[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [loading, setLoading] = useState(false);
  const [recentIds, setRecentIds] = useState<string[]>(() => {
    try {
      const stored = localStorage.getItem("cortexos-palette-recent");
      return stored ? JSON.parse(stored) : [];
    } catch (err) {
      console.warn("Failed to load recent searches from localStorage:", err);
      return [];
    }
  });
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const searchTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Build local items (apps + builtin commands)
  const localItems: PaletteItem[] = [
    ...apps.map((app) => ({
      id: `app:${app.id}`,
      title: app.name,
      subtitle: app.description,
      icon: app.icon,
      type: "app" as const,
      score: 1,
      action: { type: "open_app" as const, app_id: app.id },
    })),
    ...BUILTIN_COMMANDS,
  ];

  // Search handler with debounce
  const performSearch = useCallback(
    async (q: string) => {
      if (!q.trim()) {
        // Show recent + all items when no query
        const recents = recentIds
          .map((id) => localItems.find((item) => item.id === id))
          .filter((item): item is PaletteItem => item !== undefined);

        const unranked = localItems.filter((item) => !recentIds.includes(item.id));
        setResults([...recents, ...unranked]);
        setSelectedIndex(0);
        return;
      }

      setLoading(true);
      try {
        // Fetch remote results
        const remoteResults = await api.search(q, 20);

        // Convert remote results to palette items
        const remote: PaletteItem[] = remoteResults.map((r) => ({
          id: `search:${r.source_id}`,
          title: r.snippet.replace(/<<|>>/g, ""),
          subtitle: r.content_type,
          type: (r.content_type as PaletteItem["type"]) || "file",
          score: r.relevance,
          action: deriveAction(r),
        }));

        // Rank local items
        const rankedLocal = localItems
          .map((item) => ({ ...item, score: rankItem(item.title, item.type, q) }))
          .filter((item) => item.score > 0)
          .sort((a, b) => b.score - a.score);

        // Merge: local first (higher priority), then remote (deduped)
        const localIds = new Set(rankedLocal.map((i) => i.id));
        const dedupedRemote = remote.filter((i) => !localIds.has(i.id));

        const merged = [...rankedLocal, ...dedupedRemote].slice(0, 20);
        setResults(merged);
        setSelectedIndex(0);
      } catch (err) {
        // Fallback: show local results only
        console.warn("Search request failed, falling back to local results:", err);
        const rankedLocal = localItems
          .map((item) => ({ ...item, score: rankItem(item.title, item.type, q) }))
          .filter((item) => item.score > 0)
          .sort((a, b) => b.score - a.score);
        setResults(rankedLocal);
        setSelectedIndex(0);
      } finally {
        setLoading(false);
      }
    },
    [recentIds, apps],
  );

  // Debounced search on query change
  useEffect(() => {
    if (searchTimerRef.current) clearTimeout(searchTimerRef.current);
    searchTimerRef.current = setTimeout(() => performSearch(query), 80);
    return () => {
      if (searchTimerRef.current) clearTimeout(searchTimerRef.current);
    };
  }, [query, performSearch]);

  // Auto-focus input on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Scroll selected item into view
  useEffect(() => {
    const list = listRef.current;
    if (!list) return;
    const selected = list.children[selectedIndex] as HTMLElement | undefined;
    if (selected) {
      selected.scrollIntoView({ block: "nearest" });
    }
  }, [selectedIndex]);

  // Track recent selection
  const trackRecent = useCallback((id: string) => {
    setRecentIds((prev) => {
      const next = [id, ...prev.filter((r) => r !== id)].slice(0, 5);
      try {
        localStorage.setItem("cortexos-palette-recent", JSON.stringify(next));
      } catch (err) {
        console.warn("Failed to persist recent searches:", err);
      }
      return next;
    });
  }, []);

  // Execute selected item
  const executeItem = useCallback(
    (item: PaletteItem) => {
      trackRecent(item.id);
      switch (item.action.type) {
        case "open_app":
          onLaunchApp(item.action.app_id);
          break;
        case "open_setting":
          onOpenSetting(item.action.key);
          break;
        case "execute_command":
          // Dispatch custom event for App.tsx to handle
          window.dispatchEvent(
            new CustomEvent("cortexos:command", { detail: item.action.command_id }),
          );
          break;
        case "open_file":
          onLaunchApp(item.action.app_id);
          break;
      }
      onClose();
    },
    [onClose, onLaunchApp, onOpenSetting, trackRecent],
  );

  // Keyboard handler
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((i) => Math.min(i + 1, results.length - 1));
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((i) => Math.max(i - 1, 0));
          break;
        case "Enter":
          e.preventDefault();
          if (results[selectedIndex]) {
            executeItem(results[selectedIndex]);
          }
          break;
        case "Escape":
          e.preventDefault();
          onClose();
          break;
      }
    },
    [results, selectedIndex, executeItem, onClose],
  );

  const typeIcon = (type: string): string => {
    switch (type) {
      case "app":
        return "\u{1F4BB}";
      case "command":
        return "\u2318";
      case "file":
        return "\u{1F4C4}";
      case "setting":
        return "\u2699\uFE0F";
      default:
        return "\u{1F50D}";
    }
  };

  const recentLabel = query.trim() === "" && recentIds.length > 0;

  return (
    <div
      className="palette-overlay"
      role="dialog"
      aria-label="Command palette"
      aria-modal="true"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="palette-panel">
        <div className="palette-input-wrapper">
          <span className="palette-search-icon" aria-hidden="true">
            &#x1F50D;
          </span>
          <input
            ref={inputRef}
            className="palette-input"
            placeholder="Type a command or search..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            aria-label="Command palette search"
            aria-activedescendant={
              results[selectedIndex] ? `palette-item-${selectedIndex}` : undefined
            }
            aria-expanded="true"
            role="combobox"
            aria-controls="palette-results-list"
          />
          {loading && (
            <span className="palette-loading" aria-label="Searching">
              ...
            </span>
          )}
        </div>
        <div
          className="palette-results"
          id="palette-results-list"
          ref={listRef}
          role="listbox"
          aria-busy={loading}
        >
          {recentLabel && (
            <div className="palette-section-label" role="presentation">
              Recent
            </div>
          )}
          {results.length === 0 && !loading && query.trim() !== "" && (
            <div className="palette-empty" role="option">
              No results found
            </div>
          )}
          {results.map((item, i) => (
            <div
              key={item.id}
              id={`palette-item-${i}`}
              className={`palette-result-item ${i === selectedIndex ? "selected" : ""}`}
              role="option"
              aria-selected={i === selectedIndex}
              onClick={() => executeItem(item)}
              onMouseEnter={() => setSelectedIndex(i)}
            >
              <span className="palette-result-icon" aria-hidden="true">
                {item.icon || typeIcon(item.type)}
              </span>
              <div className="palette-result-text">
                <div className="palette-result-title">{highlightMatch(item.title, query)}</div>
                {item.subtitle && <div className="palette-result-subtitle">{item.subtitle}</div>}
              </div>
              <span className="palette-result-type">{item.type}</span>
            </div>
          ))}
        </div>
        <div className="palette-footer" aria-hidden="true">
          <span>&#x2191;&#x2193; Navigate</span>
          <span>&#x21B5; Select</span>
          <span>Esc Close</span>
        </div>
      </div>
    </div>
  );
}

/** Derive a search action from a backend SearchResult. */
function deriveAction(r: SearchResult): SearchAction {
  switch (r.content_type) {
    case "app":
      return { type: "open_app", app_id: r.source_id };
    case "setting":
      return { type: "open_setting", key: r.source_id };
    default:
      return { type: "open_file", file_id: r.source_id, app_id: "file-manager" };
  }
}

/** Highlight matching text in a title. */
function highlightMatch(title: string, query: string): React.ReactNode {
  if (!query.trim()) return title;
  const q = normalize(query);
  const lower = title.toLowerCase();
  const idx = lower.indexOf(q);
  if (idx === -1) return title;
  return (
    <>
      {title.slice(0, idx)}
      <mark className="palette-highlight">{title.slice(idx, idx + query.length)}</mark>
      {title.slice(idx + query.length)}
    </>
  );
}
