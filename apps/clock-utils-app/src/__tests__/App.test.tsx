import { render, screen, act, cleanup } from '@testing-library/react';
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import App from '../App';

describe('Clock Utils App', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    cleanup();
    vi.useRealTimers();
  });

  it('renders time display with separators', () => {
    render(<App />);
    const separators = screen.getAllByText(':');
    expect(separators.length).toBeGreaterThanOrEqual(2);
  });

  it('renders date display', () => {
    render(<App />);
    const now = new Date();
    const dateOptions: Intl.DateTimeFormatOptions = {
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    };
    const expectedDate = now.toLocaleDateString(undefined, dateOptions);
    const dateElements = screen.getAllByText(expectedDate);
    expect(dateElements.length).toBeGreaterThanOrEqual(1);
  });

  it('updates time after one second', () => {
    vi.setSystemTime(new Date('2025-01-15T12:00:00'));
    render(<App />);

    // Verify initial seconds (00)
    const initialSeconds = screen.getAllByText('00');
    expect(initialSeconds.length).toBeGreaterThanOrEqual(1);

    // Advance time by 5 seconds
    act(() => {
      vi.advanceTimersByTime(5000);
    });

    // The seconds should now show 05
    const updatedSeconds = screen.getAllByText('05');
    expect(updatedSeconds.length).toBeGreaterThanOrEqual(1);
  });
});
