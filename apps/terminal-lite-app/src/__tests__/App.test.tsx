import { render, screen, fireEvent } from '@testing-library/react';
import App from '../App';

describe('Terminal Lite App', () => {
  test('renders initial welcome message', () => {
    render(<App />);
    expect(screen.getByText(/CortexOS Terminal v0\.1\.0/)).toBeInTheDocument();
    expect(screen.getByText(/Type "help" for available commands/)).toBeInTheDocument();
  });

  test('shows prompt indicator', () => {
    render(<App />);
    const prompts = screen.getAllByText('$');
    // At minimum the active input prompt should be present
    expect(prompts.length).toBeGreaterThanOrEqual(1);
  });

  test('processes echo command', () => {
    render(<App />);
    const input = screen.getByRole('textbox');

    fireEvent.change(input, { target: { value: 'echo hello world' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(screen.getByText('hello world')).toBeInTheDocument();
  });

  test('processes help command', () => {
    render(<App />);
    const input = screen.getByRole('textbox');

    fireEvent.change(input, { target: { value: 'help' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    // The help text is rendered as a single pre-wrapped string with newlines
    const helpOutput = screen.getByText(/Available commands:/);
    expect(helpOutput).toBeInTheDocument();
    expect(helpOutput.textContent).toContain('help');
    expect(helpOutput.textContent).toContain('echo');
    expect(helpOutput.textContent).toContain('clear');
  });

  test('processes whoami command', () => {
    render(<App />);
    const input = screen.getByRole('textbox');

    fireEvent.change(input, { target: { value: 'whoami' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(screen.getByText('cortexuser')).toBeInTheDocument();
  });

  test('clear command clears output', () => {
    render(<App />);
    const input = screen.getByRole('textbox');

    // First run a command to produce output
    fireEvent.change(input, { target: { value: 'whoami' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(screen.getByText('cortexuser')).toBeInTheDocument();

    // Now clear
    fireEvent.change(input, { target: { value: 'clear' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    // The welcome message and whoami output should be gone
    expect(screen.queryByText(/CortexOS Terminal v0\.1\.0/)).not.toBeInTheDocument();
    expect(screen.queryByText('cortexuser')).not.toBeInTheDocument();
  });
});
