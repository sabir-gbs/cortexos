import React, { useState, useCallback, useRef, useEffect } from "react";
import type { AppManifest } from "../types";
import { APP_CATEGORIES } from "../types";

interface AppLauncherProps {
  apps: AppManifest[];
  onLaunch: (appId: string) => void;
  onClose: () => void;
}

export function AppLauncher({ apps, onLaunch, onClose }: AppLauncherProps) {
  const [search, setSearch] = useState("");
  const [category, setCategory] = useState<string>("All");
  const [selectedIdx, setSelectedIdx] = useState(0);
  const searchRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    searchRef.current?.focus();
  }, []);

  const filtered = apps.filter((app) => {
    const matchesCategory = category === "All" || app.category === category;
    if (!matchesCategory) return false;
    if (!search) return true;
    const q = search.toLowerCase();
    return app.name.toLowerCase().includes(q) || app.description.toLowerCase().includes(q);
  });

  const handleLaunch = useCallback(
    (appId: string) => {
      onLaunch(appId);
    },
    [onLaunch],
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case "Escape":
          onClose();
          break;
        case "ArrowDown":
          e.preventDefault();
          setSelectedIdx((i) => Math.min(i + 1, filtered.length - 1));
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIdx((i) => Math.max(i - 1, 0));
          break;
        case "Enter":
          if (filtered[selectedIdx]) {
            handleLaunch(filtered[selectedIdx].id);
          }
          break;
      }
    },
    [onClose, filtered, selectedIdx, handleLaunch],
  );

  return (
    <div
      className="launcher-overlay"
      role="dialog"
      aria-label="App launcher"
      aria-modal="true"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
      onKeyDown={handleKeyDown}
    >
      <div className="launcher-panel">
        {/* Search field */}
        <div className="launcher-search">
          <input
            ref={searchRef}
            type="text"
            className="launcher-search__input"
            placeholder="Search apps..."
            value={search}
            onChange={(e) => {
              setSearch(e.target.value);
              setSelectedIdx(0);
            }}
            aria-label="Search apps"
          />
        </div>

        {/* Category tabs */}
        <div className="launcher-categories" role="tablist" aria-label="App categories">
          {APP_CATEGORIES.map((cat) => (
            <button
              key={cat}
              className={`launcher-category${category === cat ? " active" : ""}`}
              role="tab"
              aria-selected={category === cat}
              onClick={() => {
                setCategory(cat);
                setSelectedIdx(0);
              }}
            >
              {cat}
            </button>
          ))}
        </div>

        {/* App grid */}
        <div className="launcher-grid" role="listbox" aria-label="Applications">
          {filtered.map((app, idx) => (
            <button
              key={app.id}
              className={`launcher-app${idx === selectedIdx ? " selected" : ""}`}
              role="option"
              aria-selected={idx === selectedIdx}
              onClick={() => handleLaunch(app.id)}
              onMouseEnter={() => setSelectedIdx(idx)}
            >
              <span className="launcher-app__icon" aria-hidden="true">
                {app.icon}
              </span>
              <span className="launcher-app__name">{app.name}</span>
            </button>
          ))}
          {filtered.length === 0 && <div className="launcher-empty">No apps found</div>}
        </div>
      </div>
    </div>
  );
}
