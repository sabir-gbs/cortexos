import { useState } from 'react';

type Theme = 'light' | 'dark';
type AIProvider = 'OpenAI' | 'Anthropic' | 'Google' | 'Ollama' | 'Zhipu';

interface SettingsState {
  theme: Theme;
  aiProvider: AIProvider;
  aiModel: string;
}

const AI_PROVIDERS: AIProvider[] = ['OpenAI', 'Anthropic', 'Google', 'Ollama', 'Zhipu'];
const VERSION = '0.1.0';

function App(): JSX.Element {
  const [settings, setSettings] = useState<SettingsState>({
    theme: 'dark',
    aiProvider: 'OpenAI',
    aiModel: '',
  });

  function updateSetting<K extends keyof SettingsState>(key: K, value: SettingsState[K]): void {
    setSettings((prev) => ({ ...prev, [key]: value }));
  }

  const isDark = settings.theme === 'dark';
  const bgColor = isDark ? '#1a1a2e' : '#f0f0f5';
  const textColor = isDark ? '#e0e0e0' : '#1a1a2e';
  const cardBg = isDark ? '#0f0f23' : '#ffffff';
  const borderColor = isDark ? '#2a2a4a' : '#d0d0dd';
  const labelColor = isDark ? '#a0a0c0' : '#555566';

  return (
    <div style={{ ...styles.container, backgroundColor: bgColor, color: textColor }}>
      <h1 style={{ ...styles.header, color: textColor }}>Settings</h1>

      <section style={{ ...styles.section, backgroundColor: cardBg, borderColor }}>
        <h2 style={{ ...styles.sectionTitle, color: labelColor }}>Appearance</h2>
        <div style={styles.row}>
          <label style={{ ...styles.label, color: textColor }}>Theme</label>
          <div style={styles.toggleGroup}>
            <button
              onClick={() => updateSetting('theme', 'light')}
              style={{
                ...styles.toggleButton,
                backgroundColor: !isDark ? '#bb86fc' : 'transparent',
                color: !isDark ? '#1a1a2e' : textColor,
                borderColor,
              }}
            >
              Light
            </button>
            <button
              onClick={() => updateSetting('theme', 'dark')}
              style={{
                ...styles.toggleButton,
                backgroundColor: isDark ? '#bb86fc' : 'transparent',
                color: isDark ? '#1a1a2e' : textColor,
                borderColor,
              }}
            >
              Dark
            </button>
          </div>
        </div>
      </section>

      <section style={{ ...styles.section, backgroundColor: cardBg, borderColor }}>
        <h2 style={{ ...styles.sectionTitle, color: labelColor }}>AI</h2>
        <div style={styles.row}>
          <label style={{ ...styles.label, color: textColor }}>Provider</label>
          <select
            value={settings.aiProvider}
            onChange={(e) => updateSetting('aiProvider', e.target.value as AIProvider)}
            style={{ ...styles.select, backgroundColor: isDark ? '#1a1a2e' : '#f0f0f5', color: textColor, borderColor }}
          >
            {AI_PROVIDERS.map((p) => (
              <option key={p} value={p}>
                {p}
              </option>
            ))}
          </select>
        </div>
        <div style={styles.row}>
          <label style={{ ...styles.label, color: textColor }}>Model</label>
          <input
            type="text"
            value={settings.aiModel}
            onChange={(e) => updateSetting('aiModel', e.target.value)}
            placeholder="e.g., gpt-4o, claude-3.5-sonnet..."
            style={{ ...styles.textInput, backgroundColor: isDark ? '#1a1a2e' : '#f0f0f5', color: textColor, borderColor }}
          />
        </div>
      </section>

      <section style={{ ...styles.section, backgroundColor: cardBg, borderColor }}>
        <h2 style={{ ...styles.sectionTitle, color: labelColor }}>About</h2>
        <div style={styles.row}>
          <label style={{ ...styles.label, color: textColor }}>Version</label>
          <span style={{ ...styles.value, color: labelColor }}>{VERSION}</span>
        </div>
        <div style={styles.row}>
          <label style={{ ...styles.label, color: textColor }}>OS</label>
          <span style={{ ...styles.value, color: labelColor }}>CortexOS</span>
        </div>
      </section>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100vh',
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
    padding: '20px',
    boxSizing: 'border-box',
    overflowY: 'auto',
  },
  header: {
    margin: '0 0 24px 0',
    fontSize: '24px',
    fontWeight: 600,
  },
  section: {
    border: '1px solid',
    borderRadius: '8px',
    padding: '16px 20px',
    marginBottom: '16px',
  },
  sectionTitle: {
    margin: '0 0 16px 0',
    fontSize: '12px',
    fontWeight: 600,
    textTransform: 'uppercase' as const,
    letterSpacing: '1px',
  },
  row: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: '12px',
  },
  label: {
    fontSize: '14px',
    fontWeight: 500,
    flexShrink: 0,
    width: '100px',
  },
  value: {
    fontSize: '14px',
  },
  toggleGroup: {
    display: 'flex',
    gap: '0',
  },
  toggleButton: {
    padding: '6px 16px',
    border: '1px solid',
    cursor: 'pointer',
    fontSize: '13px',
    fontWeight: 500,
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
  },
  select: {
    padding: '6px 12px',
    border: '1px solid',
    borderRadius: '4px',
    fontSize: '13px',
    outline: 'none',
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
    minWidth: '160px',
  },
  textInput: {
    padding: '6px 12px',
    border: '1px solid',
    borderRadius: '4px',
    fontSize: '13px',
    outline: 'none',
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
    flex: 1,
    maxWidth: '300px',
  },
};

export default App;
