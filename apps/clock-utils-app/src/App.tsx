import { useState, useEffect } from 'react';

function App(): JSX.Element {
  const [time, setTime] = useState<Date>(new Date());

  useEffect(() => {
    const timer = setInterval(() => {
      setTime(new Date());
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  const hours = time.getHours().toString().padStart(2, '0');
  const minutes = time.getMinutes().toString().padStart(2, '0');
  const seconds = time.getSeconds().toString().padStart(2, '0');

  const dateOptions: Intl.DateTimeFormatOptions = {
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  };
  const dateStr = time.toLocaleDateString(undefined, dateOptions);

  return (
    <div style={styles.container}>
      <div style={styles.clockContainer}>
        <div style={styles.timeDisplay}>
          <span style={styles.digit}>{hours}</span>
          <span style={styles.separator}>:</span>
          <span style={styles.digit}>{minutes}</span>
          <span style={styles.separator}>:</span>
          <span style={styles.digit}>{seconds}</span>
        </div>
        <div style={styles.dateDisplay}>
          {dateStr}
        </div>
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    height: '100vh',
    backgroundColor: '#1a1a2e',
    color: '#e0e0e0',
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
    margin: 0,
  },
  clockContainer: {
    textAlign: 'center',
  },
  timeDisplay: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    gap: '4px',
    marginBottom: '24px',
  },
  digit: {
    fontSize: '72px',
    fontWeight: 'bold',
    fontFamily: "'Courier New', Courier, monospace",
    color: '#e0e0e0',
    backgroundColor: '#0f0f23',
    padding: '8px 16px',
    borderRadius: '8px',
    border: '1px solid #2a2a4a',
    minWidth: '90px',
    textAlign: 'center',
  },
  separator: {
    fontSize: '64px',
    fontWeight: 'bold',
    color: '#bb86fc',
    animation: 'blink 1s step-end infinite',
  },
  dateDisplay: {
    fontSize: '24px',
    color: '#a0a0c0',
    letterSpacing: '2px',
  },
};

export default App;
