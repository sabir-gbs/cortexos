import { useState } from 'react';

interface Note {
  id: number;
  text: string;
  createdAt: Date;
}

function App(): JSX.Element {
  const [notes, setNotes] = useState<Note[]>([]);
  const [inputValue, setInputValue] = useState('');
  const [nextId, setNextId] = useState(1);

  function addNote(): void {
    const trimmed = inputValue.trim();
    if (!trimmed) return;

    const newNote: Note = {
      id: nextId,
      text: trimmed,
      createdAt: new Date(),
    };
    setNotes([newNote, ...notes]);
    setInputValue('');
    setNextId(nextId + 1);
  }

  function deleteNote(id: number): void {
    setNotes(notes.filter((note) => note.id !== id));
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>): void {
    if (e.key === 'Enter') {
      addNote();
    }
  }

  function formatTime(date: Date): string {
    return date.toLocaleTimeString(undefined, {
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  return (
    <div style={styles.container}>
      <h1 style={styles.header}>Notes</h1>
      <div style={styles.inputRow}>
        <input
          type="text"
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Write a note..."
          style={styles.input}
        />
        <button onClick={addNote} style={styles.addButton}>
          Add
        </button>
      </div>
      <div style={styles.noteList}>
        {notes.length === 0 && (
          <div style={styles.emptyState}>No notes yet. Add one above.</div>
        )}
        {notes.map((note) => (
          <div key={note.id} style={styles.noteItem}>
            <div style={styles.noteContent}>
              <span style={styles.noteText}>{note.text}</span>
              <span style={styles.noteTime}>{formatTime(note.createdAt)}</span>
            </div>
            <button
              onClick={() => deleteNote(note.id)}
              style={styles.deleteButton}
              title="Delete note"
            >
              x
            </button>
          </div>
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
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
    padding: '20px',
    boxSizing: 'border-box',
  },
  header: {
    margin: '0 0 20px 0',
    fontSize: '24px',
    fontWeight: 600,
    color: '#e0e0e0',
  },
  inputRow: {
    display: 'flex',
    gap: '8px',
    marginBottom: '20px',
  },
  input: {
    flex: 1,
    backgroundColor: '#0f0f23',
    border: '1px solid #2a2a4a',
    borderRadius: '6px',
    padding: '10px 14px',
    color: '#e0e0e0',
    fontSize: '14px',
    outline: 'none',
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
  },
  addButton: {
    backgroundColor: '#4a3f6b',
    color: '#bb86fc',
    border: 'none',
    borderRadius: '6px',
    padding: '10px 20px',
    cursor: 'pointer',
    fontSize: '14px',
    fontWeight: 600,
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
  },
  noteList: {
    flex: 1,
    overflowY: 'auto',
  },
  emptyState: {
    color: '#666688',
    textAlign: 'center',
    marginTop: '40px',
    fontSize: '14px',
  },
  noteItem: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#0f0f23',
    border: '1px solid #2a2a4a',
    borderRadius: '6px',
    padding: '12px 16px',
    marginBottom: '8px',
  },
  noteContent: {
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
    flex: 1,
  },
  noteText: {
    fontSize: '14px',
    color: '#e0e0e0',
  },
  noteTime: {
    fontSize: '11px',
    color: '#666688',
  },
  deleteButton: {
    backgroundColor: 'transparent',
    color: '#ff6b6b',
    border: '1px solid #6b3a3a',
    borderRadius: '4px',
    padding: '4px 10px',
    cursor: 'pointer',
    fontSize: '14px',
    fontWeight: 'bold',
    marginLeft: '12px',
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
  },
};

export default App;
