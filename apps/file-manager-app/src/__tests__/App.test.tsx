import { describe, test, expect, afterEach, vi } from 'vitest';
import { render, screen, cleanup, waitFor } from '@testing-library/react';
import App from '../App';

// Mock fetch for the file system API
const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

afterEach(() => {
  cleanup();
  mockFetch.mockReset();
});

const mockDirectoryResponse = {
  data: [
    { name: 'Documents', is_directory: true, size_bytes: 0, updated_at: '2026-03-28T00:00:00Z' },
    { name: 'Pictures', is_directory: true, size_bytes: 0, updated_at: '2026-03-25T00:00:00Z' },
    { name: 'Downloads', is_directory: true, size_bytes: 0, updated_at: '2026-03-30T00:00:00Z' },
    { name: 'Music', is_directory: true, size_bytes: 0, updated_at: '2026-03-20T00:00:00Z' },
    { name: 'readme.txt', is_directory: false, size_bytes: 2457, updated_at: '2026-03-15T00:00:00Z' },
    { name: 'config.json', is_directory: false, size_bytes: 819, updated_at: '2026-03-29T00:00:00Z' },
  ],
};

function setupFetchMock(response: Record<string, unknown> = mockDirectoryResponse) {
  mockFetch.mockResolvedValueOnce({
    ok: true,
    status: 200,
    json: () => Promise.resolve(response),
  });
}

describe('File Manager App', () => {
  test('renders breadcrumb with "Root"', async () => {
    setupFetchMock();
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText('Root')).toBeInTheDocument();
    });
  });

  test('renders file items from root directory', async () => {
    setupFetchMock();
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText('readme.txt')).toBeInTheDocument();
      expect(screen.getByText('config.json')).toBeInTheDocument();
    });
  });

  test('renders folder items', async () => {
    setupFetchMock();
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText('Documents')).toBeInTheDocument();
      expect(screen.getByText('Pictures')).toBeInTheDocument();
      expect(screen.getByText('Downloads')).toBeInTheDocument();
      expect(screen.getByText('Music')).toBeInTheDocument();
    });
  });

  test('shows column headers (Name, Size, Modified)', async () => {
    setupFetchMock();
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText('Name')).toBeInTheDocument();
      expect(screen.getByText('Size')).toBeInTheDocument();
      expect(screen.getByText('Modified')).toBeInTheDocument();
    });
  });

  test('shows loading state', () => {
    mockFetch.mockReturnValue(new Promise(() => {})); // never resolves
    render(<App />);
    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  test('shows error state on fetch failure', async () => {
    mockFetch.mockRejectedValueOnce(new Error('Network error'));
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText(/Error: Network error/)).toBeInTheDocument();
    });
  });

  test('shows empty state when directory has no items', async () => {
    setupFetchMock({
      data: [],
    });
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText('This folder is empty')).toBeInTheDocument();
    });
  });

  test('renders Refresh button', async () => {
    setupFetchMock();
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText('Refresh')).toBeInTheDocument();
    });
  });
});
