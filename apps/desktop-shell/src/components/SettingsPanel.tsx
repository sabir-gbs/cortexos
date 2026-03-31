import React, { useState } from "react";
import type { ShellSettings } from "../types";

interface SettingsPanelProps {
  settings: ShellSettings;
  onUpdateSettings: (key: string, value: unknown) => void;
  onClose: () => void;
}

export function SettingsPanel({ settings, onUpdateSettings, onClose }: SettingsPanelProps) {
  const [tab, setTab] = useState<"appearance" | "taskbar" | "desktop" | "clock">("appearance");

  return (
    <div
      className="settings-overlay"
      role="dialog"
      aria-label="Settings"
      aria-modal="true"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="settings-panel">
        <div className="settings-panel__header">
          <h2 className="settings-panel__title">Settings</h2>
          <button className="settings-panel__close" onClick={onClose} aria-label="Close settings">
            &#x2715;
          </button>
        </div>

        <div className="settings-panel__tabs" role="tablist">
          {(["appearance", "taskbar", "desktop", "clock"] as const).map((t) => (
            <button
              key={t}
              className={`settings-tab${tab === t ? " active" : ""}`}
              role="tab"
              aria-selected={tab === t}
              onClick={() => setTab(t)}
            >
              {t.charAt(0).toUpperCase() + t.slice(1)}
            </button>
          ))}
        </div>

        <div className="settings-panel__content" role="tabpanel">
          {tab === "appearance" && (
            <div className="settings-group">
              <label className="settings-label">
                Theme
                <select
                  className="settings-select"
                  value={settings.theme.mode}
                  onChange={(e) => onUpdateSettings("theme.mode", e.target.value)}
                >
                  <option value="dark">Dark</option>
                  <option value="light">Light</option>
                </select>
              </label>
            </div>
          )}

          {tab === "taskbar" && (
            <div className="settings-group">
              <label className="settings-label">
                Auto-hide taskbar
                <input
                  type="checkbox"
                  className="settings-checkbox"
                  checked={settings.taskbar.auto_hide}
                  onChange={(e) => onUpdateSettings("taskbar.auto_hide", e.target.checked)}
                />
              </label>
              <label className="settings-label">
                Taskbar height
                <select
                  className="settings-select"
                  value={settings.taskbar.height}
                  onChange={(e) => onUpdateSettings("taskbar.height", Number(e.target.value))}
                >
                  <option value={40}>Compact (40px)</option>
                  <option value={48}>Default (48px)</option>
                  <option value={56}>Large (56px)</option>
                </select>
              </label>
            </div>
          )}

          {tab === "desktop" && (
            <div className="settings-group">
              <label className="settings-label">
                Background color
                <input
                  type="color"
                  className="settings-color"
                  value={
                    settings.desktop.background.type === "solid"
                      ? settings.desktop.background.color
                      : "#1a1a2e"
                  }
                  onChange={(e) =>
                    onUpdateSettings("desktop.background", { type: "solid", color: e.target.value })
                  }
                />
              </label>
            </div>
          )}

          {tab === "clock" && (
            <div className="settings-group">
              <label className="settings-label">
                Clock format
                <select
                  className="settings-select"
                  value={settings.clock.format}
                  onChange={(e) => onUpdateSettings("clock.format", e.target.value)}
                >
                  <option value="24h">24-hour</option>
                  <option value="12h">12-hour</option>
                </select>
              </label>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
