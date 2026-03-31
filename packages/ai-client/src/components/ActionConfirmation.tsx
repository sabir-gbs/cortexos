import React from 'react';
import type { ActionConfirmationProps, AIActionTarget } from '../types';

/** CortexOS design tokens */
const COLORS = {
  bg: '#1a1a2e',
  bgLighter: '#16213e',
  bgSurface: '#0f3460',
  text: '#e0e0e0',
  textMuted: '#a0a0b0',
  accent: '#533483',
  accentLight: '#7b68ee',
  border: '#2a2a4a',
  danger: '#e74c3c',
  dangerLight: 'rgba(231, 76, 60, 0.15)',
  success: '#27ae60',
} as const;

/** Describe an action target for the user */
function describeTarget(target: AIActionTarget | undefined): string {
  if (!target) return 'the current content';
  if (target.type === 'selectedText') {
    const preview =
      target.content.length > 50
        ? target.content.slice(0, 50) + '...'
        : target.content;
    return `selected text: "${preview}"`;
  }
  if (target.type === 'selectedFiles') {
    const fileNames = target.paths.map((p) => p.split('/').pop()).join(', ');
    return `files: ${fileNames}`;
  }
  if (target.type === 'both') {
    const parts: string[] = [];
    if (target.text) parts.push('selected text');
    if (target.files.length > 0) parts.push('selected files');
    return parts.join(' and ');
  }
  return 'the current content';
}

/**
 * ActionConfirmation - Shows when AI wants to take an action
 *
 * Displays action details and asks for user confirmation before
 * executing modifying or destructive actions.
 */
export function ActionConfirmation({
  action,
  target,
  onConfirm,
  onDeny,
  isLoading = false,
}: ActionConfirmationProps) {
  const isDestructive = action.isDestructive ?? false;
  const targetDescription = describeTarget(target);

  const containerStyle: React.CSSProperties = {
    backgroundColor: isDestructive ? COLORS.dangerLight : COLORS.bgLighter,
    border: `1px solid ${isDestructive ? COLORS.danger : COLORS.border}`,
    borderRadius: '10px',
    padding: '16px',
    display: 'flex',
    flexDirection: 'column',
    gap: '12px',
    fontFamily:
      '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
  };

  const titleStyle: React.CSSProperties = {
    color: isDestructive ? COLORS.danger : COLORS.text,
    fontSize: '14px',
    fontWeight: 600,
    margin: 0,
  };

  const descriptionStyle: React.CSSProperties = {
    color: COLORS.text,
    fontSize: '13px',
    lineHeight: '1.5',
    margin: 0,
  };

  const actionLabelStyle: React.CSSProperties = {
    color: COLORS.textMuted,
    fontSize: '12px',
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
  };

  const buttonRowStyle: React.CSSProperties = {
    display: 'flex',
    gap: '8px',
    justifyContent: 'flex-end',
  };

  const baseButtonStyle: React.CSSProperties = {
    padding: '8px 16px',
    borderRadius: '6px',
    fontSize: '13px',
    fontWeight: 500,
    cursor: isLoading ? 'wait' : 'pointer',
    border: 'none',
    transition: 'opacity 0.15s',
  };

  const confirmButtonStyle: React.CSSProperties = {
    ...baseButtonStyle,
    backgroundColor: isDestructive ? COLORS.danger : COLORS.success,
    color: '#ffffff',
    opacity: isLoading ? 0.6 : 1,
  };

  const denyButtonStyle: React.CSSProperties = {
    ...baseButtonStyle,
    backgroundColor: 'transparent',
    color: COLORS.textMuted,
    border: `1px solid ${COLORS.border}`,
    opacity: isLoading ? 0.6 : 1,
  };

  return (
    <div
      style={containerStyle}
      role="alertdialog"
      aria-label={`Confirm AI action: ${action.label}`}
    >
      <h3 style={titleStyle}>
        {isDestructive ? 'Destructive Action' : 'Confirm AI Action'}
      </h3>

      <p style={descriptionStyle}>
        {action.label} will modify {targetDescription}.
      </p>

      {action.description && (
        <div style={actionLabelStyle}>
          <span style={{ color: COLORS.textMuted, fontSize: '12px' }}>
            {action.description}
          </span>
        </div>
      )}

      <div style={buttonRowStyle}>
        <button
          style={denyButtonStyle}
          onClick={onDeny}
          disabled={isLoading}
          aria-label="Cancel action"
        >
          Cancel
        </button>
        <button
          style={confirmButtonStyle}
          onClick={onConfirm}
          disabled={isLoading}
          aria-label={isDestructive ? 'Confirm destructive action' : 'Apply action'}
        >
          {isLoading ? 'Processing...' : isDestructive ? 'Confirm' : 'Apply'}
        </button>
      </div>
    </div>
  );
}

export default ActionConfirmation;
