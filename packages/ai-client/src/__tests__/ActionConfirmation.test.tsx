import { render, screen, fireEvent, cleanup } from '@testing-library/react';
import { describe, it, expect, afterEach, vi } from 'vitest';
import React from 'react';
import { ActionConfirmation } from '../components/ActionConfirmation';
import type { AIAction, AIActionTarget } from '../types';

function makeAction(overrides: Partial<AIAction> = {}): AIAction {
  return {
    actionId: 'action-improve',
    appId: 'text-editor',
    label: 'Improve Writing',
    description: 'Improve the style and clarity of the selected text',
    category: { type: 'text', operation: 'improve' },
    requiresConfirmation: true,
    isDestructive: false,
    ...overrides,
  };
}

const defaultTarget: AIActionTarget = {
  type: 'selectedText',
  content: 'This is some selected text that needs improvement.',
};

describe('ActionConfirmation', () => {
  afterEach(() => {
    cleanup();
  });

  const defaultProps = {
    action: makeAction(),
    target: defaultTarget,
    onConfirm: vi.fn(),
    onDeny: vi.fn(),
    isLoading: false,
  };

  it('renders the confirmation dialog', () => {
    render(<ActionConfirmation {...defaultProps} />);
    expect(screen.getByRole('alertdialog')).toBeInTheDocument();
  });

  it('shows the action label', () => {
    render(<ActionConfirmation {...defaultProps} />);
    expect(screen.getByText('Confirm AI Action')).toBeInTheDocument();
  });

  it('shows the action description in the message', () => {
    render(<ActionConfirmation {...defaultProps} />);
    expect(
      screen.getByText(/Improve Writing will modify/),
    ).toBeInTheDocument();
  });

  it('shows the action description text', () => {
    render(<ActionConfirmation {...defaultProps} />);
    expect(
      screen.getByText('Improve the style and clarity of the selected text'),
    ).toBeInTheDocument();
  });

  it('has Apply and Cancel buttons', () => {
    render(<ActionConfirmation {...defaultProps} />);
    expect(screen.getByRole('button', { name: /apply action/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /cancel action/i })).toBeInTheDocument();
  });

  it('calls onConfirm when Apply is clicked', () => {
    const onConfirm = vi.fn();
    render(<ActionConfirmation {...defaultProps} onConfirm={onConfirm} />);
    fireEvent.click(screen.getByRole('button', { name: /apply action/i }));
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it('calls onDeny when Cancel is clicked', () => {
    const onDeny = vi.fn();
    render(<ActionConfirmation {...defaultProps} onDeny={onDeny} />);
    fireEvent.click(screen.getByRole('button', { name: /cancel action/i }));
    expect(onDeny).toHaveBeenCalledTimes(1);
  });

  it('shows destructive styling for destructive actions', () => {
    const destructiveAction = makeAction({ isDestructive: true, label: 'Replace All' });
    const { container } = render(
      <ActionConfirmation {...defaultProps} action={destructiveAction} />,
    );

    // Title should say "Destructive Action"
    expect(screen.getByText('Destructive Action')).toBeInTheDocument();

    // Container should have red-ish border
    const dialog = container.firstElementChild as HTMLElement;
    expect(dialog.style.borderColor).toBe('rgb(231, 76, 60)'); // #e74c3c
  });

  it('shows "Confirm" button text for destructive actions', () => {
    const destructiveAction = makeAction({ isDestructive: true });
    render(<ActionConfirmation {...defaultProps} action={destructiveAction} />);
    expect(screen.getByRole('button', { name: /confirm destructive action/i })).toBeInTheDocument();
    expect(screen.getByText('Confirm')).toBeInTheDocument();
  });

  it('shows "Apply" button text for non-destructive actions', () => {
    render(<ActionConfirmation {...defaultProps} />);
    expect(screen.getByText('Apply')).toBeInTheDocument();
  });

  it('disables buttons when loading', () => {
    render(<ActionConfirmation {...defaultProps} isLoading={true} />);
    expect(screen.getByRole('button', { name: /apply action/i })).toBeDisabled();
    expect(screen.getByRole('button', { name: /cancel action/i })).toBeDisabled();
  });

  it('shows loading text when loading', () => {
    render(<ActionConfirmation {...defaultProps} isLoading={true} />);
    expect(screen.getByText('Processing...')).toBeInTheDocument();
  });

  it('shows selected text preview in message', () => {
    const target: AIActionTarget = {
      type: 'selectedText',
      content: 'Short text',
    };
    render(<ActionConfirmation {...defaultProps} target={target} />);
    expect(screen.getByText(/Short text/)).toBeInTheDocument();
  });

  it('truncates long selected text in preview', () => {
    const longText = 'A'.repeat(100);
    const target: AIActionTarget = {
      type: 'selectedText',
      content: longText,
    };
    render(<ActionConfirmation {...defaultProps} target={target} />);
    // Should show first 50 chars with ellipsis
    expect(screen.getByText(new RegExp(`"${'A'.repeat(50)}\\.\\.\\."`))).toBeInTheDocument();
  });

  it('shows file names for file targets', () => {
    const target: AIActionTarget = {
      type: 'selectedFiles',
      paths: ['/home/user/docs/report.md', '/home/user/docs/notes.txt'],
    };
    render(<ActionConfirmation {...defaultProps} target={target} />);
    expect(screen.getByText(/report\.md, notes\.txt/)).toBeInTheDocument();
  });

  it('uses non-destructive styling by default', () => {
    const { container } = render(<ActionConfirmation {...defaultProps} />);
    const dialog = container.firstElementChild as HTMLElement;
    // Should not have red border for non-destructive actions
    expect(dialog.style.borderColor).not.toBe('rgb(231, 76, 60)');
  });

  it('has proper aria-label on the dialog', () => {
    render(<ActionConfirmation {...defaultProps} />);
    expect(
      screen.getByRole('alertdialog', { name: /Confirm AI action: Improve Writing/ }),
    ).toBeInTheDocument();
  });
});
