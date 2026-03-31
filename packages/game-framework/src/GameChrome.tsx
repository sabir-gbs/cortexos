import React from "react";

// ---------------------------------------------------------------------------
// Toolbar
// ---------------------------------------------------------------------------

export interface ToolbarProps {
  score: number;
  elapsed: string; // formatted MM:SS
  paused: boolean;
  onTogglePause: () => void;
  onOpenSettings?: () => void;
  onOpenHelp?: () => void;
}

export const Toolbar: React.FC<ToolbarProps> = ({
  score,
  elapsed,
  paused,
  onTogglePause,
  onOpenSettings,
  onOpenHelp,
}) => {
  const toolbarStyle: React.CSSProperties = {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    gap: "12px",
    padding: "8px 16px",
    backgroundColor: "var(--game-toolbar-bg)",
    borderBottom: "1px solid var(--game-toolbar-border)",
    fontFamily: "var(--game-font-sans)",
    color: "var(--game-text-primary)",
    userSelect: "none",
  };

  const buttonBase: React.CSSProperties = {
    padding: "4px 12px",
    border: "none",
    borderRadius: "var(--game-radius-sm)",
    backgroundColor: "var(--game-button-bg)",
    color: "var(--game-button-text)",
    cursor: "pointer",
    fontFamily: "var(--game-font-sans)",
    fontSize: "14px",
    transition: "background-color var(--game-transition-fast)",
  };

  return (
    <div style={toolbarStyle} role="toolbar" aria-label="Game toolbar">
      <div style={{ display: "flex", gap: "12px", alignItems: "center" }}>
        <ScoreDisplay score={score} />
        <TimerDisplay elapsed={elapsed} />
      </div>
      <div style={{ display: "flex", gap: "8px" }}>
        <button
          style={buttonBase}
          onClick={onTogglePause}
          aria-label={paused ? "Resume" : "Pause"}
        >
          {paused ? "\u25B6" : "\u23F8"}
        </button>
        {onOpenSettings && (
          <button style={buttonBase} onClick={onOpenSettings} aria-label="Settings">
            {"\u2699"}
          </button>
        )}
        {onOpenHelp && (
          <button style={buttonBase} onClick={onOpenHelp} aria-label="Help">
            {"\u2753"}
          </button>
        )}
      </div>
    </div>
  );
};

// ---------------------------------------------------------------------------
// ScoreDisplay
// ---------------------------------------------------------------------------

export interface ScoreDisplayProps {
  score: number;
}

export const ScoreDisplay: React.FC<ScoreDisplayProps> = ({ score }) => {
  const style: React.CSSProperties = {
    fontFamily: "var(--game-font-mono)",
    fontSize: "16px",
    fontWeight: "bold",
    color: "var(--game-text-accent)",
  };
  return (
    <span style={style} aria-label={`Score: ${score}`}>
      Score: {score}
    </span>
  );
};

// ---------------------------------------------------------------------------
// TimerDisplay
// ---------------------------------------------------------------------------

export interface TimerDisplayProps {
  elapsed: string;
}

export const TimerDisplay: React.FC<TimerDisplayProps> = ({ elapsed }) => {
  const style: React.CSSProperties = {
    fontFamily: "var(--game-font-mono)",
    fontSize: "16px",
    fontWeight: "bold",
    color: "var(--game-text-secondary)",
  };
  return (
    <span style={style} aria-label={`Time: ${elapsed}`}>
      {elapsed}
    </span>
  );
};

// ---------------------------------------------------------------------------
// GameOverlay (pause / game-over / won)
// ---------------------------------------------------------------------------

export interface GameOverlayProps {
  visible: boolean;
  title: string;
  message?: string;
  /** Optional primary action label & handler. */
  actionLabel?: string;
  onAction?: () => void;
}

export const GameOverlay: React.FC<GameOverlayProps> = ({
  visible,
  title,
  message,
  actionLabel,
  onAction,
}) => {
  if (!visible) return null;

  const overlayStyle: React.CSSProperties = {
    position: "absolute",
    inset: 0,
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
    backgroundColor: "var(--game-overlay-bg)",
    fontFamily: "var(--game-font-sans)",
    color: "var(--game-text-primary)",
    zIndex: 10,
  };

  const cardStyle: React.CSSProperties = {
    backgroundColor: "var(--game-toolbar-bg)",
    border: "1px solid var(--game-toolbar-border)",
    borderRadius: "var(--game-radius-lg)",
    padding: "32px 48px",
    textAlign: "center" as const,
  };

  return (
    <div style={overlayStyle} role="dialog" aria-modal="true" aria-label={title}>
      <div style={cardStyle}>
        <h2 style={{ margin: "0 0 8px", fontSize: "24px" }}>{title}</h2>
        {message && <p style={{ margin: "0 0 16px", color: "var(--game-text-secondary)" }}>{message}</p>}
        {actionLabel && onAction && (
          <button
            onClick={onAction}
            style={{
              padding: "8px 24px",
              border: "none",
              borderRadius: "var(--game-radius-sm)",
              backgroundColor: "var(--game-button-bg)",
              color: "var(--game-button-text)",
              cursor: "pointer",
              fontSize: "16px",
              fontFamily: "var(--game-font-sans)",
            }}
          >
            {actionLabel}
          </button>
        )}
      </div>
    </div>
  );
};

// ---------------------------------------------------------------------------
// HelpOverlay
// ---------------------------------------------------------------------------

export interface HelpOverlayProps {
  visible: boolean;
  title?: string;
  rules: string[];
  onClose: () => void;
}

export const HelpOverlay: React.FC<HelpOverlayProps> = ({
  visible,
  title = "How to Play",
  rules,
  onClose,
}) => {
  if (!visible) return null;

  const overlayStyle: React.CSSProperties = {
    position: "absolute",
    inset: 0,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    backgroundColor: "var(--game-overlay-bg)",
    fontFamily: "var(--game-font-sans)",
    color: "var(--game-text-primary)",
    zIndex: 10,
  };

  const cardStyle: React.CSSProperties = {
    backgroundColor: "var(--game-toolbar-bg)",
    border: "1px solid var(--game-toolbar-border)",
    borderRadius: "var(--game-radius-lg)",
    padding: "24px 32px",
    maxWidth: "420px",
    width: "90%",
  };

  return (
    <div style={overlayStyle} role="dialog" aria-modal="true" aria-label={title}>
      <div style={cardStyle}>
        <h2 style={{ margin: "0 0 12px" }}>{title}</h2>
        <ol style={{ margin: "0 0 16px", paddingLeft: "20px", lineHeight: 1.6 }}>
          {rules.map((rule, i) => (
            <li key={i}>{rule}</li>
          ))}
        </ol>
        <button
          onClick={onClose}
          style={{
            padding: "6px 20px",
            border: "none",
            borderRadius: "var(--game-radius-sm)",
            backgroundColor: "var(--game-button-bg)",
            color: "var(--game-button-text)",
            cursor: "pointer",
            fontFamily: "var(--game-font-sans)",
          }}
        >
          Got it
        </button>
      </div>
    </div>
  );
};
