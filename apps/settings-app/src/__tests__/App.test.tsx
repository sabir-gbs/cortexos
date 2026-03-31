import { describe, test, expect, afterEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/react';
import App from '../App';

afterEach(() => {
  cleanup();
});

describe('Settings App', () => {
  test('renders "Settings" header', () => {
    render(<App />);
    expect(screen.getByRole('heading', { name: /settings/i })).toBeInTheDocument();
  });

  test('renders Appearance section', () => {
    render(<App />);
    expect(screen.getByText('Appearance')).toBeInTheDocument();
  });

  test('renders AI section with provider dropdown', () => {
    render(<App />);
    expect(screen.getByText('AI')).toBeInTheDocument();
    const select = screen.getByRole('combobox');
    expect(select).toBeInTheDocument();
    expect(screen.getByText('OpenAI')).toBeInTheDocument();
  });

  test('renders About section with version', () => {
    render(<App />);
    expect(screen.getByText('About')).toBeInTheDocument();
    expect(screen.getByText('0.1.0')).toBeInTheDocument();
  });

  test('theme toggle buttons exist', () => {
    render(<App />);
    expect(screen.getByRole('button', { name: /light/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /dark/i })).toBeInTheDocument();
  });
});
