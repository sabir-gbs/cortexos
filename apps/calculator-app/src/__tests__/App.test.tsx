import { render, screen, fireEvent, cleanup } from '@testing-library/react';
import { describe, it, expect, afterEach } from 'vitest';
import App from '../App';

describe('Calculator App', () => {
  afterEach(() => {
    cleanup();
  });

  function renderApp(): void {
    render(<App />);
  }

  function clickButton(label: string): void {
    const buttons = screen.getAllByRole('button', { name: label });
    fireEvent.click(buttons[buttons.length - 1]);
  }

  function getDisplay(): HTMLElement {
    const container = document.querySelector('[class]')?.parentElement;
    // The display is the first span inside the display div
    // Find the span that contains the current display value
    const allSpans = screen.getAllByText(/^[\d.]+$/, { selector: 'span' });
    // Return the last one (most recent render)
    return allSpans[allSpans.length - 1];
  }

  it('shows initial display of "0"', () => {
    renderApp();
    expect(getDisplay()).toHaveTextContent('0');
  });

  it('allows digit input', () => {
    renderApp();
    clickButton('5');
    expect(getDisplay()).toHaveTextContent('5');
  });

  it('performs addition (2 + 3 = 5)', () => {
    renderApp();
    clickButton('2');
    clickButton('+');
    clickButton('3');
    clickButton('=');
    expect(getDisplay()).toHaveTextContent('5');
  });

  it('performs division (6 / 2 = 3)', () => {
    renderApp();
    clickButton('6');
    clickButton('/');
    clickButton('2');
    clickButton('=');
    expect(getDisplay()).toHaveTextContent('3');
  });

  it('returns 0 on division by zero', () => {
    renderApp();
    clickButton('5');
    clickButton('/');
    clickButton('0');
    clickButton('=');
    expect(getDisplay()).toHaveTextContent('0');
  });

  it('clears the display', () => {
    renderApp();
    clickButton('9');
    clickButton('C');
    expect(getDisplay()).toHaveTextContent('0');
  });

  it('allows decimal input', () => {
    renderApp();
    clickButton('3');
    clickButton('.');
    clickButton('1');
    clickButton('4');
    expect(getDisplay()).toHaveTextContent('3.14');
  });
});
