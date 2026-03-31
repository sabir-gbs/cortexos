import { useState, useEffect, useCallback, useRef } from 'react';

const API_BASE: string = import.meta.env.VITE_API_URL || '';

function App(): JSX.Element {
  const [content, setContent] = useState<string>('');
  const [filename, setFilename] = useState<string>('untitled.txt');
  const [saveStatus, setSaveStatus] = useState<'saved' | 'unsaved' | 'saving' | 'loading'>('loading');
  const [showFind, setShowFind] = useState(false);
  const [findQuery, setFindQuery] = useState('');
  const [replaceQuery, setReplaceQuery] = useState('');
  const [findIndex, setFindIndex] = useState(-1);
  const [matchCount, setMatchCount] = useState(0);
  const [showLineNumbers, setShowLineNumbers] = useState(true);
  const [wordWrap, setWordWrap] = useState(true);
  const [activeTab, setActiveTab] = useState<'edit' | 'preview'>('edit');

  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // ── API helpers ─────────────────────────────────────────────────────────────

  async function loadFileFromAPI(name: string): Promise<string> {
    const encodedPath = encodeURIComponent(name.startsWith('/') ? name : `/${name}`);
    const res = await fetch(`${API_BASE}/api/v1/files/${encodedPath}`, {
      credentials: 'include',
      headers: { 'Content-Type': 'application/json' },
    });
    if (!res.ok) {
      throw new Error(`Failed to load file: ${res.status}`);
    }
    const json = await res.json();
    const data = json.data ?? json;
    return String(data.content ?? data.body ?? data.text ?? '');
  }

  async function saveFileToAPI(name: string, text: string): Promise<void> {
    const encodedPath = encodeURIComponent(name.startsWith('/') ? name : `/${name}`);
    const res = await fetch(`${API_BASE}/api/v1/files/${encodedPath}`, {
      method: 'PUT',
      credentials: 'include',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ content: text }),
    });
    if (!res.ok) {
      throw new Error(`Failed to save file: ${res.status}`);
    }
  }

  // ── Load initial content ────────────────────────────────────────────────────

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const text = await loadFileFromAPI(filename);
        if (!cancelled) {
          setContent(text);
          setSaveStatus('saved');
        }
      } catch (err) {
        console.error("Failed to load file:", err);
        if (!cancelled) {
          setContent('');
          setSaveStatus('saved');
        }
      }
    }
    load();
    return () => { cancelled = true; };
  }, []); // Only load once on mount

  // ── Auto-save ───────────────────────────────────────────────────────────────

  const saveToAPI = useCallback(async (text: string, name: string): Promise<void> => {
    setSaveStatus('saving');
    try {
      await saveFileToAPI(name, text);
      setSaveStatus('saved');
    } catch (err) {
      console.error("Failed to save file:", err);
      setSaveStatus('unsaved');
    }
  }, []);

  useEffect(() => {
    if (saveStatus !== 'unsaved') return;
    const timer = setTimeout(() => {
      saveToAPI(content, filename);
    }, 800);
    return () => clearTimeout(timer);
  }, [content, filename, saveStatus, saveToAPI]);

  // Find matches
  useEffect(() => {
    if (!findQuery) {
      setMatchCount(0);
      setFindIndex(-1);
      return;
    }
    const matches: number[] = [];
    let pos = 0;
    const lower = content.toLowerCase();
    const query = findQuery.toLowerCase();
    while ((pos = lower.indexOf(query, pos)) !== -1) {
      matches.push(pos);
      pos += 1;
    }
    setMatchCount(matches.length);
    if (matches.length > 0) {
      setFindIndex(prev => prev >= 0 && prev < matches.length ? prev : 0);
    } else {
      setFindIndex(-1);
    }
  }, [findQuery, content]);

  function handleChange(e: React.ChangeEvent<HTMLTextAreaElement>): void {
    setContent(e.target.value);
    setSaveStatus('unsaved');
  }

  function handleNew(): void {
    setContent('');
    setFilename('untitled.txt');
    setSaveStatus('saved');
    setShowFind(false);
    setFindQuery('');
    setReplaceQuery('');
  }

  function handleSave(): void {
    saveToAPI(content, filename);
  }

  function handleKeyDown(e: React.KeyboardEvent): void {
    if (e.ctrlKey && e.key === 'f') {
      e.preventDefault();
      setShowFind(true);
    }
    if (e.ctrlKey && e.shiftKey && e.key === 'H') {
      e.preventDefault();
      handleReplaceAll();
    }
    if (e.ctrlKey && e.key === 's') {
      e.preventDefault();
      handleSave();
    }
  }

  function findNext(): void {
    if (matchCount === 0) return;
    const next = (findIndex + 1) % matchCount;
    setFindIndex(next);
    highlightMatch(next);
  }

  function findPrev(): void {
    if (matchCount === 0) return;
    const prev = findIndex <= 0 ? matchCount - 1 : findIndex - 1;
    setFindIndex(prev);
    highlightMatch(prev);
  }

  function highlightMatch(idx: number): void {
    if (!textareaRef.current || !findQuery) return;
    const lower = content.toLowerCase();
    const query = findQuery.toLowerCase();
    let pos = 0;
    let matchPos = -1;
    for (let i = 0; i <= idx; i++) {
      matchPos = lower.indexOf(query, pos);
      if (matchPos === -1) return;
      pos = matchPos + 1;
    }
    if (matchPos >= 0) {
      textareaRef.current.focus();
      textareaRef.current.setSelectionRange(matchPos, matchPos + findQuery.length);
    }
  }

  function handleReplace(): void {
    if (findIndex < 0 || !findQuery) return;
    const lower = content.toLowerCase();
    const query = findQuery.toLowerCase();
    let pos = 0;
    let matchPos = -1;
    for (let i = 0; i <= findIndex; i++) {
      matchPos = lower.indexOf(query, pos);
      if (matchPos === -1) return;
      pos = matchPos + 1;
    }
    if (matchPos >= 0) {
      const newContent = content.slice(0, matchPos) + replaceQuery + content.slice(matchPos + findQuery.length);
      setContent(newContent);
      setSaveStatus('unsaved');
    }
  }

  function handleReplaceAll(): void {
    if (!findQuery) return;
    const lower = content.toLowerCase();
    const query = findQuery.toLowerCase();
    if (!lower.includes(query)) return;
    // Replace from end to preserve indices
    let newContent = content;
    let pos = 0;
    const lowerNC = newContent.toLowerCase();
    while ((pos = lowerNC.indexOf(query, pos)) !== -1) {
      newContent = newContent.slice(0, pos) + replaceQuery + newContent.slice(pos + findQuery.length);
      pos += replaceQuery.length;
    }
    setContent(newContent);
    setSaveStatus('unsaved');
  }

  // Stats
  const lines = content.split('\n');
  const lineCount = lines.length;
  const charCount = content.length;
  const wordCount = content.trim() ? content.trim().split(/\s+/).length : 0;
  const isDirty = saveStatus === 'unsaved';

  // Line numbers
  const lineNumbers = showLineNumbers
    ? lines.map((_, i) => i + 1).join('\n')
    : '';

  const statusColor = saveStatus === 'saved' ? '#6bffa4' : saveStatus === 'saving' ? '#ffd666' : saveStatus === 'loading' ? '#888' : '#ff6b6b';
  const statusText = saveStatus === 'saved' ? 'Saved' : saveStatus === 'saving' ? 'Saving...' : saveStatus === 'loading' ? 'Loading...' : 'Unsaved';

  return (
    <div style={styles.container}>
      {/* Title Bar */}
      <div style={styles.titleBar}>
        <span style={styles.filename}>
          {filename}{isDirty ? ' *' : ''}
        </span>
        <div style={styles.stats}>
          <span style={styles.stat}>Ln {lineCount}</span>
          <span style={styles.stat}>Ch {charCount}</span>
          <span style={styles.stat}>Words {wordCount}</span>
        </div>
      </div>

      {/* Toolbar */}
      <div style={styles.toolbar}>
        <button onClick={handleNew} style={styles.toolBtn} title="New file">New</button>
        <button onClick={handleSave} style={styles.toolBtn} title="Save (Ctrl+S)">Save</button>
        <span style={styles.separator} />
        <button onClick={() => setShowLineNumbers(!showLineNumbers)} style={showLineNumbers ? styles.toolBtnActive : styles.toolBtn}>
          Lines
        </button>
        <button onClick={() => setWordWrap(!wordWrap)} style={wordWrap ? styles.toolBtnActive : styles.toolBtn}>
          Wrap
        </button>
        <span style={styles.separator} />
        <button onClick={() => setShowFind(true)} style={styles.toolBtn} title="Find (Ctrl+F)">Find</button>
        <button onClick={handleReplaceAll} style={styles.toolBtn} title="Replace All (Ctrl+Shift+H)">Replace All</button>
        <div style={{ flex: 1 }} />
        <span style={{ ...styles.saveStatus, color: statusColor }}>{statusText}</span>
      </div>

      {/* Find/Replace Bar */}
      {showFind && (
        <div style={styles.findBar}>
          <input
            value={findQuery}
            onChange={e => setFindQuery(e.target.value)}
            placeholder="Find..."
            style={styles.findInput}
            autoFocus
          />
          <button onClick={findPrev} style={styles.findBtn} disabled={matchCount === 0}>Prev</button>
          <button onClick={findNext} style={styles.findBtn} disabled={matchCount === 0}>Next</button>
          <span style={styles.matchInfo}>
            {matchCount > 0 ? `${findIndex + 1}/${matchCount}` : 'No matches'}
          </span>
          <input
            value={replaceQuery}
            onChange={e => setReplaceQuery(e.target.value)}
            placeholder="Replace..."
            style={styles.findInput}
          />
          <button onClick={handleReplace} style={styles.findBtn} disabled={findIndex < 0}>Replace</button>
          <button onClick={handleReplaceAll} style={styles.findBtn}>All</button>
          <button onClick={() => { setShowFind(false); setFindQuery(''); }} style={styles.closeBtn}>x</button>
        </div>
      )}

      {/* Editor Area */}
      <div style={styles.editorArea}>
        {showLineNumbers && (
          <div style={styles.lineNumbers}>
            <pre style={styles.lineNumbersPre}>{lineNumbers}</pre>
          </div>
        )}
        <textarea
          ref={textareaRef}
          value={content}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          placeholder="Start typing..."
          style={{
            ...styles.textarea,
            whiteSpace: wordWrap ? 'pre-wrap' : 'pre',
            overflowX: wordWrap ? 'hidden' : 'auto',
          }}
          spellCheck
        />
      </div>

      {/* Status Bar */}
      <div style={styles.statusBar}>
        <div style={styles.tabs}>
          <button
            onClick={() => setActiveTab('edit')}
            style={activeTab === 'edit' ? styles.tabActive : styles.tab}
          >Edit</button>
          <button
            onClick={() => setActiveTab('preview')}
            style={activeTab === 'preview' ? styles.tabActive : styles.tab}
          >Preview</button>
        </div>
        <span style={styles.statusText}>UTF-8 | {wordWrap ? 'Wrap' : 'No Wrap'}</span>
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
  },
  titleBar: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '6px 16px',
    backgroundColor: '#0f0f23',
    borderBottom: '1px solid #2a2a4a',
  },
  filename: {
    fontSize: '13px',
    fontWeight: 600,
    color: '#a0a0c0',
  },
  stats: {
    display: 'flex',
    gap: '12px',
  },
  stat: {
    fontSize: '11px',
    color: '#666688',
  },
  toolbar: {
    display: 'flex',
    alignItems: 'center',
    gap: '4px',
    padding: '4px 8px',
    backgroundColor: '#12122a',
    borderBottom: '1px solid #2a2a4a',
  },
  toolBtn: {
    backgroundColor: '#2a2a4a',
    color: '#a0a0c0',
    border: '1px solid #3a3a5a',
    borderRadius: '3px',
    padding: '3px 10px',
    cursor: 'pointer',
    fontSize: '12px',
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
  },
  toolBtnActive: {
    backgroundColor: '#4a3f6b',
    color: '#bb86fc',
    border: '1px solid #6a5f8b',
    borderRadius: '3px',
    padding: '3px 10px',
    cursor: 'pointer',
    fontSize: '12px',
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
  },
  separator: {
    width: '1px',
    height: '16px',
    backgroundColor: '#3a3a5a',
    margin: '0 4px',
  },
  saveStatus: {
    fontSize: '11px',
    fontWeight: 500,
  },
  findBar: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    padding: '6px 12px',
    backgroundColor: '#15152d',
    borderBottom: '1px solid #2a2a4a',
  },
  findInput: {
    backgroundColor: '#0f0f23',
    border: '1px solid #3a3a5a',
    borderRadius: '3px',
    padding: '4px 8px',
    color: '#e0e0e0',
    fontSize: '12px',
    width: '140px',
    outline: 'none',
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
  },
  findBtn: {
    backgroundColor: '#2a2a4a',
    color: '#a0a0c0',
    border: '1px solid #3a3a5a',
    borderRadius: '3px',
    padding: '3px 8px',
    cursor: 'pointer',
    fontSize: '11px',
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
  },
  matchInfo: {
    fontSize: '11px',
    color: '#888',
    minWidth: '60px',
  },
  closeBtn: {
    backgroundColor: 'transparent',
    color: '#ff6b6b',
    border: 'none',
    cursor: 'pointer',
    fontSize: '14px',
    fontWeight: 'bold',
    padding: '2px 6px',
  },
  editorArea: {
    display: 'flex',
    flex: 1,
    overflow: 'hidden',
  },
  lineNumbers: {
    backgroundColor: '#0f0f23',
    padding: '16px 8px 16px 12px',
    textAlign: 'right',
    userSelect: 'none',
    borderRight: '1px solid #2a2a4a',
    overflow: 'hidden',
    minWidth: '40px',
  },
  lineNumbersPre: {
    margin: 0,
    fontSize: '13px',
    lineHeight: 1.5,
    color: '#555577',
    fontFamily: "'Courier New', Courier, monospace",
  },
  textarea: {
    flex: 1,
    backgroundColor: '#1a1a2e',
    color: '#e0e0e0',
    border: 'none',
    outline: 'none',
    resize: 'none',
    padding: '16px',
    fontSize: '13px',
    lineHeight: 1.5,
    fontFamily: "'Courier New', Courier, monospace",
    boxSizing: 'border-box',
    tabSize: 2,
  },
  statusBar: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '4px 16px',
    backgroundColor: '#0f0f23',
    borderTop: '1px solid #2a2a4a',
  },
  tabs: {
    display: 'flex',
    gap: '2px',
  },
  tab: {
    backgroundColor: 'transparent',
    color: '#666688',
    border: 'none',
    padding: '2px 10px',
    cursor: 'pointer',
    fontSize: '11px',
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
  },
  tabActive: {
    backgroundColor: '#2a2a4a',
    color: '#e0e0e0',
    border: 'none',
    borderRadius: '3px',
    padding: '2px 10px',
    cursor: 'pointer',
    fontSize: '11px',
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
  },
  statusText: {
    fontSize: '11px',
    color: '#666688',
  },
};

export default App;
