import { render, screen, fireEvent, cleanup, act } from '@testing-library/react';
import { describe, it, expect, afterEach, vi } from 'vitest';
import App from '../App';

describe('Games Platform App', () => {
  afterEach(() => {
    cleanup();
    vi.useRealTimers();
  });

  it('renders the Games header', () => {
    render(<App />);
    expect(screen.getByText('CortexOS Games')).toBeInTheDocument();
  });

  it('shows all 5 game cards', () => {
    render(<App />);
    expect(screen.getByText('Solitaire')).toBeInTheDocument();
    expect(screen.getByText('Minesweeper')).toBeInTheDocument();
    expect(screen.getByText('Snake')).toBeInTheDocument();
    expect(screen.getByText('Tetris')).toBeInTheDocument();
    expect(screen.getByText('Chess')).toBeInTheDocument();
  });

  it('shows a Play button for each game', () => {
    render(<App />);
    const playButtons = screen.getAllByRole('button', { name: /^Play / });
    expect(playButtons).toHaveLength(5);
  });

  it('shows a Play button for Solitaire', () => {
    render(<App />);
    expect(screen.getByRole('button', { name: 'Play Solitaire' })).toBeInTheDocument();
  });

  it('shows a Play button for Minesweeper', () => {
    render(<App />);
    expect(screen.getByRole('button', { name: 'Play Minesweeper' })).toBeInTheDocument();
  });

  it('shows a Play button for Snake', () => {
    render(<App />);
    expect(screen.getByRole('button', { name: 'Play Snake' })).toBeInTheDocument();
  });

  it('shows a Play button for Tetris', () => {
    render(<App />);
    expect(screen.getByRole('button', { name: 'Play Tetris' })).toBeInTheDocument();
  });

  it('shows a Play button for Chess', () => {
    render(<App />);
    expect(screen.getByRole('button', { name: 'Play Chess' })).toBeInTheDocument();
  });

  it('shows a toast when Play is clicked', () => {
    vi.useFakeTimers();
    render(<App />);
    const playButton = screen.getByRole('button', { name: 'Play Snake' });
    act(() => {
      fireEvent.click(playButton);
    });
    expect(screen.getByText('Launching Snake...')).toBeInTheDocument();
  });

  it('hides the toast after a timeout', () => {
    vi.useFakeTimers();
    render(<App />);
    const playButton = screen.getByRole('button', { name: 'Play Chess' });
    act(() => {
      fireEvent.click(playButton);
    });
    expect(screen.getByText('Launching Chess...')).toBeInTheDocument();
    act(() => {
      vi.advanceTimersByTime(2000);
    });
    expect(screen.queryByText('Launching Chess...')).not.toBeInTheDocument();
  });

  it('displays game descriptions', () => {
    render(<App />);
    expect(screen.getByText(/Classic Klondike solitaire/)).toBeInTheDocument();
    expect(screen.getByText(/Clear the minefield/)).toBeInTheDocument();
    expect(screen.getByText(/Guide the snake/)).toBeInTheDocument();
    expect(screen.getByText(/Rotate and stack falling blocks/)).toBeInTheDocument();
    expect(screen.getByText(/classic strategy board game/)).toBeInTheDocument();
  });
});
