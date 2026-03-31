import { useState } from 'react';

type Operation = '+' | '-' | '*' | '/' | null;

function App(): JSX.Element {
  const [display, setDisplay] = useState<string>('0');
  const [previousValue, setPreviousValue] = useState<number | null>(null);
  const [operation, setOperation] = useState<Operation>(null);
  const [waitingForOperand, setWaitingForOperand] = useState(false);

  function inputDigit(digit: string): void {
    if (waitingForOperand) {
      setDisplay(digit);
      setWaitingForOperand(false);
    } else {
      setDisplay(display === '0' ? digit : display + digit);
    }
  }

  function inputDecimal(): void {
    if (waitingForOperand) {
      setDisplay('0.');
      setWaitingForOperand(false);
      return;
    }
    if (!display.includes('.')) {
      setDisplay(display + '.');
    }
  }

  function clear(): void {
    setDisplay('0');
    setPreviousValue(null);
    setOperation(null);
    setWaitingForOperand(false);
  }

  function performOperation(nextOperation: Operation): void {
    const inputValue = parseFloat(display);

    if (previousValue === null) {
      setPreviousValue(inputValue);
    } else if (operation) {
      const result = calculate(previousValue, inputValue, operation);
      setPreviousValue(result);
      setDisplay(String(result));
    }

    setWaitingForOperand(true);
    setOperation(nextOperation);
  }

  function calculate(a: number, b: number, op: Operation): number {
    switch (op) {
      case '+': return a + b;
      case '-': return a - b;
      case '*': return a * b;
      case '/': return b !== 0 ? a / b : 0;
      default: return b;
    }
  }

  function handleEquals(): void {
    if (operation && previousValue !== null) {
      const inputValue = parseFloat(display);
      const result = calculate(previousValue, inputValue, operation);
      setDisplay(String(result));
      setPreviousValue(null);
      setOperation(null);
      setWaitingForOperand(true);
    }
  }

  const buttons: Array<{ label: string; onClick: () => void; className?: string }> = [
    { label: 'C', onClick: clear, className: 'btn-clear' },
    { label: '/', onClick: () => performOperation('/'), className: 'btn-op' },
    { label: '*', onClick: () => performOperation('*'), className: 'btn-op' },
    { label: '-', onClick: () => performOperation('-'), className: 'btn-op' },
    { label: '7', onClick: () => inputDigit('7') },
    { label: '8', onClick: () => inputDigit('8') },
    { label: '9', onClick: () => inputDigit('9') },
    { label: '+', onClick: () => performOperation('+'), className: 'btn-op' },
    { label: '4', onClick: () => inputDigit('4') },
    { label: '5', onClick: () => inputDigit('5') },
    { label: '6', onClick: () => inputDigit('6') },
    { label: '=', onClick: handleEquals, className: 'btn-equals' },
    { label: '1', onClick: () => inputDigit('1') },
    { label: '2', onClick: () => inputDigit('2') },
    { label: '3', onClick: () => inputDigit('3') },
    { label: '.', onClick: inputDecimal },
    { label: '0', onClick: () => inputDigit('0'), className: 'btn-zero' },
  ];

  return (
    <div style={styles.container}>
      <div style={styles.display}>
        <span style={styles.displayText}>
          {display}
        </span>
      </div>
      <div style={styles.buttonGrid}>
        {buttons.map((btn) => (
          <button
            key={btn.label}
            onClick={btn.onClick}
            className={btn.className}
            style={{
              ...styles.button,
              ...(btn.className === 'btn-op' ? styles.opButton : {}),
              ...(btn.className === 'btn-clear' ? styles.clearButton : {}),
              ...(btn.className === 'btn-equals' ? styles.equalsButton : {}),
              ...(btn.className === 'btn-zero' ? styles.zeroButton : {}),
            }}
          >
            {btn.label}
          </button>
        ))}
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100vh',
    backgroundColor: '#1a1a2e',
    color: '#e0e0e0',
    fontFamily: "'Courier New', Courier, monospace",
    padding: '16px',
    boxSizing: 'border-box',
  },
  display: {
    backgroundColor: '#0f0f23',
    border: '1px solid #2a2a4a',
    borderRadius: '8px',
    padding: '20px',
    marginBottom: '16px',
    textAlign: 'right' as const,
    minHeight: '60px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'flex-end',
  },
  displayText: {
    fontSize: '36px',
    fontWeight: 'bold',
    color: '#e0e0e0',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
  },
  buttonGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(4, 1fr)',
    gap: '8px',
    flex: 1,
  },
  button: {
    backgroundColor: '#2a2a4a',
    color: '#e0e0e0',
    border: 'none',
    borderRadius: '8px',
    fontSize: '20px',
    cursor: 'pointer',
    fontFamily: "'Courier New', Courier, monospace",
    transition: 'background-color 0.15s',
    minHeight: '48px',
  },
  opButton: {
    backgroundColor: '#4a3f6b',
    color: '#bb86fc',
  },
  clearButton: {
    backgroundColor: '#6b3a3a',
    color: '#ff6b6b',
  },
  equalsButton: {
    backgroundColor: '#3a6b4a',
    color: '#6bffa4',
  },
  zeroButton: {
    gridColumn: 'span 2',
  },
};

export default App;
