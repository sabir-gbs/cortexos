import { useState } from 'react';

interface GameEntry {
  id: string;
  name: string;
  description: string;
}

const GAMES: GameEntry[] = [
  {
    id: 'com.cortexos.games.solitaire',
    name: 'Solitaire',
    description: 'Classic Klondike solitaire card game. Stack cards by suit from Ace to King.',
  },
  {
    id: 'com.cortexos.games.minesweeper',
    name: 'Minesweeper',
    description: 'Clear the minefield without detonating any mines. Use logic to flag danger zones.',
  },
  {
    id: 'com.cortexos.games.snake',
    name: 'Snake',
    description: 'Guide the snake to eat food and grow longer. Avoid walls and your own tail.',
  },
  {
    id: 'com.cortexos.games.tetris',
    name: 'Tetris',
    description: 'Rotate and stack falling blocks to complete horizontal lines before they pile up.',
  },
  {
    id: 'com.cortexos.games.chess',
    name: 'Chess',
    description: 'The classic strategy board game. Checkmate your opponent to win.',
  },
];

function App(): JSX.Element {
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  function handlePlay(gameName: string): void {
    setToastMessage(`Launching ${gameName}...`);
    setTimeout(() => setToastMessage(null), 2000);
  }

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <h1 style={styles.headerTitle}>CortexOS Games</h1>
      </header>
      <div style={styles.grid}>
        {GAMES.map((game) => (
          <div key={game.id} style={styles.card}>
            <h2 style={styles.cardTitle}>{game.name}</h2>
            <p style={styles.cardDescription}>{game.description}</p>
            <button
              style={styles.playButton}
              onClick={() => handlePlay(game.name)}
              aria-label={`Play ${game.name}`}
            >
              Play
            </button>
          </div>
        ))}
      </div>
      {toastMessage && (
        <div style={styles.toast} role="status" aria-live="polite">
          {toastMessage}
        </div>
      )}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    minHeight: '100vh',
    backgroundColor: '#1a1a2e',
    color: '#e0e0e0',
    fontFamily: "'Courier New', Courier, monospace",
    padding: '16px',
    boxSizing: 'border-box',
    position: 'relative',
  },
  header: {
    textAlign: 'center' as const,
    marginBottom: '24px',
    paddingBottom: '12px',
    borderBottom: '1px solid #2a2a4a',
  },
  headerTitle: {
    fontSize: '28px',
    fontWeight: 'bold',
    color: '#e0e0e0',
    margin: 0,
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(260px, 1fr))',
    gap: '16px',
    flex: 1,
  },
  card: {
    backgroundColor: '#16213e',
    border: '1px solid #2a2a4a',
    borderRadius: '12px',
    padding: '20px',
    display: 'flex',
    flexDirection: 'column',
    justifyContent: 'space-between',
    transition: 'border-color 0.15s',
  },
  cardTitle: {
    fontSize: '20px',
    fontWeight: 'bold',
    color: '#e0e0e0',
    margin: '0 0 8px 0',
  },
  cardDescription: {
    fontSize: '14px',
    color: '#a0a0b8',
    lineHeight: 1.4,
    margin: '0 0 16px 0',
    flex: 1,
  },
  playButton: {
    backgroundColor: '#4a3f6b',
    color: '#bb86fc',
    border: 'none',
    borderRadius: '8px',
    padding: '10px 20px',
    fontSize: '16px',
    fontWeight: 'bold',
    cursor: 'pointer',
    fontFamily: "'Courier New', Courier, monospace",
    transition: 'background-color 0.15s',
    alignSelf: 'flex-start',
  },
  toast: {
    position: 'fixed' as const,
    bottom: '24px',
    left: '50%',
    transform: 'translateX(-50%)',
    backgroundColor: '#2a2a4a',
    color: '#e0e0e0',
    padding: '12px 24px',
    borderRadius: '8px',
    fontSize: '14px',
    fontFamily: "'Courier New', Courier, monospace",
    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.4)',
    zIndex: 1000,
  },
};

export default App;
