import { describe, test, expect, afterEach, vi } from 'vitest';
import { render, screen, fireEvent, cleanup, waitFor } from '@testing-library/react';
import App from '../App';

// Mock fetch for the file API
const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

afterEach(() => {
  cleanup();
  mockFetch.mockReset();
});

// Default mock: file doesn't exist yet (404), so editor starts empty
function setupFileNotFoundMock() {
  mockFetch.mockRejectedValueOnce(new Error('Failed to load file: 404'));
}

function setupFileLoadMock(content: string) {
  mockFetch.mockResolvedValueOnce({
    ok: true,
    status: 200,
    json: () => Promise.resolve({ data: { content } }),
  });
}

describe('Text Editor App', () => {
  test('renders filename in title bar', async () => {
    setupFileNotFoundMock();
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText(/untitled\.txt/)).toBeInTheDocument();
    });
  });

  test('shows stats (Ln, Ch, Words) after loading', async () => {
    setupFileNotFoundMock();
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText(/Ln 1/)).toBeInTheDocument();
      expect(screen.getByText(/Ch 0/)).toBeInTheDocument();
      expect(screen.getByText(/Words 0/)).toBeInTheDocument();
    });
  });

  test('shows toolbar buttons', async () => {
    setupFileNotFoundMock();
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText('New')).toBeInTheDocument();
      expect(screen.getByText('Save')).toBeInTheDocument();
      expect(screen.getByText('Find')).toBeInTheDocument();
    });
  });

  test('has a textarea for editing', async () => {
    setupFileNotFoundMock();
    render(<App />);
    await waitFor(() => {
      const textarea = screen.getByPlaceholderText('Start typing...');
      expect(textarea).toBeInTheDocument();
    });
  });

  test('typing updates content and stats', async () => {
    setupFileNotFoundMock();
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText(/Ln 1/)).toBeInTheDocument();
    });
    const textarea = screen.getByPlaceholderText('Start typing...');
    fireEvent.change(textarea, { target: { value: 'Hello world' } });
    expect(screen.getByText(/Ch 11/)).toBeInTheDocument();
    expect(screen.getByText(/Words 2/)).toBeInTheDocument();
  });

  test('shows line numbers by default', async () => {
    setupFileNotFoundMock();
    render(<App />);
    await waitFor(() => {
      // Line number "1" should be visible
      const lineNumbers = screen.getByText('1');
      expect(lineNumbers).toBeInTheDocument();
    });
  });

  test('toggles line numbers off', async () => {
    setupFileNotFoundMock();
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText('1')).toBeInTheDocument();
    });
    const linesBtn = screen.getByText('Lines');
    fireEvent.click(linesBtn);
    // After toggling off, line numbers should disappear
    expect(screen.queryByText('1')).not.toBeInTheDocument();
  });

  test('shows find bar when Find is clicked', async () => {
    setupFileNotFoundMock();
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText('Save')).toBeInTheDocument();
    });
    fireEvent.click(screen.getByText('Find'));
    expect(screen.getByPlaceholderText('Find...')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Replace...')).toBeInTheDocument();
  });

  test('find highlights matches', async () => {
    setupFileNotFoundMock();
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText(/Ln 1/)).toBeInTheDocument();
    });
    const textarea = screen.getByPlaceholderText('Start typing...');
    fireEvent.change(textarea, { target: { value: 'hello hello hello' } });
    fireEvent.click(screen.getByText('Find'));
    const findInput = screen.getByPlaceholderText('Find...');
    fireEvent.change(findInput, { target: { value: 'hello' } });
    expect(screen.getByText('1/3')).toBeInTheDocument();
  });

  test('New button clears content', async () => {
    setupFileNotFoundMock();
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText(/Ln 1/)).toBeInTheDocument();
    });
    const textarea = screen.getByPlaceholderText('Start typing...');
    fireEvent.change(textarea, { target: { value: 'Some text' } });
    fireEvent.click(screen.getByText('New'));
    expect(screen.getByPlaceholderText('Start typing...')).toBeInTheDocument();
    expect(screen.getByText(/Ch 0/)).toBeInTheDocument();
  });

  test('shows status bar with encoding info', async () => {
    setupFileNotFoundMock();
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText(/UTF-8/)).toBeInTheDocument();
    });
  });

  test('shows Edit and Preview tabs', async () => {
    setupFileNotFoundMock();
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText('Edit')).toBeInTheDocument();
      expect(screen.getByText('Preview')).toBeInTheDocument();
    });
  });

  test('loads content from API on mount', async () => {
    setupFileLoadMock('Pre-existing content');
    render(<App />);
    await waitFor(() => {
      const textarea = screen.getByPlaceholderText('Start typing...');
      expect(textarea).toHaveValue('Pre-existing content');
    });
  });
});
