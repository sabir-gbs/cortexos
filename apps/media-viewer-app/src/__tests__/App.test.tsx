import { describe, test, expect, afterEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/react';
import App from '../App';

afterEach(() => {
  cleanup();
});

describe('Media Viewer App', () => {
  test('renders placeholder when no media', () => {
    render(<App />);
    expect(screen.getByText('No media selected')).toBeInTheDocument();
  });

  test('shows "No media selected" text', () => {
    render(<App />);
    expect(screen.getByText('No media selected')).toBeInTheDocument();
  });

  test('shows "Drag and drop" hint', () => {
    render(<App />);
    expect(screen.getByText(/drag and drop a file here/i)).toBeInTheDocument();
  });

  test('renders drop zone', () => {
    render(<App />);
    // The drop zone contains the placeholder text
    const placeholder = screen.getByText('No media selected');
    const dropZone = placeholder.closest('div');
    expect(dropZone).toBeInTheDocument();
  });
});
