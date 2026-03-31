import { render, screen, fireEvent } from '@testing-library/react';
import App from '../App';

describe('Notes App', () => {
  test('renders "Notes" header', () => {
    render(<App />);
    expect(screen.getByRole('heading', { name: /notes/i })).toBeInTheDocument();
  });

  test('shows empty state message when no notes', () => {
    render(<App />);
    expect(screen.getByText(/no notes yet/i)).toBeInTheDocument();
  });

  test('adds a note when clicking Add button with text', () => {
    render(<App />);
    const input = screen.getByPlaceholderText(/write a note/i);
    fireEvent.change(input, { target: { value: 'My first note' } });
    fireEvent.click(screen.getByRole('button', { name: /add/i }));

    expect(screen.getByText('My first note')).toBeInTheDocument();
    expect(screen.queryByText(/no notes yet/i)).not.toBeInTheDocument();
  });

  test('adds a note on Enter key', () => {
    render(<App />);
    const input = screen.getByPlaceholderText(/write a note/i);
    fireEvent.change(input, { target: { value: 'Enter note' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(screen.getByText('Enter note')).toBeInTheDocument();
  });

  test('does not add empty notes', () => {
    render(<App />);
    const input = screen.getByPlaceholderText(/write a note/i);

    // Try adding with empty input
    fireEvent.click(screen.getByRole('button', { name: /add/i }));
    expect(screen.getByText(/no notes yet/i)).toBeInTheDocument();

    // Try adding with whitespace only
    fireEvent.change(input, { target: { value: '   ' } });
    fireEvent.click(screen.getByRole('button', { name: /add/i }));
    expect(screen.getByText(/no notes yet/i)).toBeInTheDocument();
  });

  test('deletes a note when clicking delete button', () => {
    render(<App />);
    const input = screen.getByPlaceholderText(/write a note/i);
    fireEvent.change(input, { target: { value: 'Note to delete' } });
    fireEvent.click(screen.getByRole('button', { name: /add/i }));

    expect(screen.getByText('Note to delete')).toBeInTheDocument();

    // Each note has a delete button with title "Delete note"
    fireEvent.click(screen.getByTitle(/delete note/i));
    expect(screen.queryByText('Note to delete')).not.toBeInTheDocument();
    expect(screen.getByText(/no notes yet/i)).toBeInTheDocument();
  });
});
