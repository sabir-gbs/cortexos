import { useState, useRef, useEffect, KeyboardEvent } from 'react';

interface TerminalLine {
  id: number;
  type: 'input' | 'output';
  content: string;
}

function App(): JSX.Element {
  const [lines, setLines] = useState<TerminalLine[]>([
    { id: 0, type: 'output', content: 'CortexOS Terminal v0.1.0' },
    { id: 1, type: 'output', content: 'Type "help" for available commands.\n' },
  ]);
  const [currentInput, setCurrentInput] = useState('');
  const [nextId, setNextId] = useState(2);
  const bottomRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [lines]);

  function processCommand(input: string): TerminalLine[] {
    const trimmed = input.trim();
    const parts = trimmed.split(/\s+/);
    const command = parts[0]?.toLowerCase();
    const args = parts.slice(1).join(' ');

    const newLines: TerminalLine[] = [];

    switch (command) {
      case 'help':
        newLines.push({
          id: nextId,
          type: 'output',
          content: 'Available commands:\n  help    - Show this help message\n  echo    - Echo text back\n  clear   - Clear the terminal\n  date    - Show current date and time\n  whoami  - Show current user\n  uname   - Show system information',
        });
        break;
      case 'echo':
        newLines.push({
          id: nextId,
          type: 'output',
          content: args || '',
        });
        break;
      case 'clear':
        return [];
      case 'date':
        newLines.push({
          id: nextId,
          type: 'output',
          content: new Date().toString(),
        });
        break;
      case 'whoami':
        newLines.push({
          id: nextId,
          type: 'output',
          content: 'cortexuser',
        });
        break;
      case 'uname':
        newLines.push({
          id: nextId,
          type: 'output',
          content: 'CortexOS 0.1.0 cortex-kernel x86_64',
        });
        break;
      case '':
        break;
      default:
        newLines.push({
          id: nextId,
          type: 'output',
          content: `command not found: ${command}`,
        });
        break;
    }

    return newLines;
  }

  function handleKeyDown(e: KeyboardEvent<HTMLInputElement>): void {
    if (e.key === 'Enter') {
      const inputLine: TerminalLine = {
        id: nextId,
        type: 'input',
        content: currentInput,
      };

      const outputLines = processCommand(currentInput);
      const maxId = outputLines.length > 0
        ? Math.max(...outputLines.map((l) => l.id))
        : inputLine.id;

      if (currentInput.trim().toLowerCase() === 'clear') {
        setLines([]);
      } else {
        setLines((prev) => [...prev, inputLine, ...outputLines]);
      }

      setNextId(maxId + 1);
      setCurrentInput('');
    }
  }

  function focusInput(): void {
    inputRef.current?.focus();
  }

  return (
    <div style={styles.container} onClick={focusInput}>
      <div style={styles.terminal}>
        {lines.map((line) => (
          <div key={line.id} style={line.type === 'input' ? styles.inputLine : styles.outputLine}>
            {line.type === 'input' && <span style={styles.prompt}>$ </span>}
            <span style={styles.lineContent}>{line.content}</span>
          </div>
        ))}
        <div style={styles.inputLine}>
          <span style={styles.prompt}>$ </span>
          <input
            ref={inputRef}
            type="text"
            value={currentInput}
            onChange={(e) => setCurrentInput(e.target.value)}
            onKeyDown={handleKeyDown}
            style={styles.input}
            autoFocus
          />
        </div>
        <div ref={bottomRef} />
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    height: '100vh',
    backgroundColor: '#1a1a2e',
    fontFamily: "'Courier New', Courier, monospace",
    cursor: 'text',
  },
  terminal: {
    flex: 1,
    padding: '12px 16px',
    overflowY: 'auto',
    fontSize: '14px',
    lineHeight: 1.5,
  },
  inputLine: {
    display: 'flex',
    alignItems: 'center',
  },
  outputLine: {
    color: '#e0e0e0',
    whiteSpace: 'pre-wrap',
  },
  prompt: {
    color: '#6bffa4',
    fontWeight: 'bold',
    flexShrink: 0,
    marginRight: '4px',
  },
  lineContent: {
    color: '#e0e0e0',
  },
  input: {
    flex: 1,
    backgroundColor: 'transparent',
    border: 'none',
    outline: 'none',
    color: '#e0e0e0',
    fontFamily: "'Courier New', Courier, monospace",
    fontSize: '14px',
    padding: 0,
    margin: 0,
  },
};

export default App;
