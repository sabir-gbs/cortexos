import { useState, useRef, DragEvent } from 'react';

interface MediaState {
  fileName: string;
  fileSize: number;
  fileType: string;
}

function App(): JSX.Element {
  const [media, setMedia] = useState<MediaState | null>(null);
  const [isDragOver, setIsDragOver] = useState(false);
  const dropRef = useRef<HTMLDivElement>(null);

  function handleDragOver(e: DragEvent<HTMLDivElement>): void {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(true);
  }

  function handleDragLeave(e: DragEvent<HTMLDivElement>): void {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
  }

  function handleDrop(e: DragEvent<HTMLDivElement>): void {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);

    const files = e.dataTransfer.files;
    if (files.length > 0) {
      const file = files[0];
      setMedia({
        fileName: file.name,
        fileSize: file.size,
        fileType: file.type,
      });
    }
  }

  function clearMedia(): void {
    setMedia(null);
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  const dropZoneBorder = isDragOver ? '2px dashed #bb86fc' : '2px dashed #3a3a5a';
  const dropZoneBg = isDragOver ? '#1f1f4a' : '#0f0f23';

  return (
    <div style={styles.container}>
      <div
        ref={dropRef}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        style={{
          ...styles.dropZone,
          border: dropZoneBorder,
          backgroundColor: dropZoneBg,
        }}
      >
        {media ? (
          <div style={styles.mediaInfo}>
            <div style={styles.mediaIcon}>&#127916;</div>
            <div style={styles.mediaName}>{media.fileName}</div>
            <div style={styles.mediaMeta}>
              {media.fileType || 'Unknown type'} &middot; {formatSize(media.fileSize)}
            </div>
            <button onClick={clearMedia} style={styles.clearButton}>
              Clear
            </button>
          </div>
        ) : (
          <div style={styles.placeholder}>
            <div style={styles.placeholderIcon}>&#128247;</div>
            <div style={styles.placeholderText}>No media selected</div>
            <div style={styles.placeholderHint}>Drag and drop a file here</div>
          </div>
        )}
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    height: '100vh',
    backgroundColor: '#1a1a2e',
    color: '#e0e0e0',
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
    padding: '20px',
    boxSizing: 'border-box',
  },
  dropZone: {
    flex: 1,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    borderRadius: '12px',
    transition: 'all 0.2s ease',
  },
  placeholder: {
    textAlign: 'center',
  },
  placeholderIcon: {
    fontSize: '48px',
    marginBottom: '16px',
  },
  placeholderText: {
    fontSize: '18px',
    color: '#a0a0c0',
    marginBottom: '8px',
  },
  placeholderHint: {
    fontSize: '13px',
    color: '#666688',
  },
  mediaInfo: {
    textAlign: 'center',
  },
  mediaIcon: {
    fontSize: '48px',
    marginBottom: '16px',
  },
  mediaName: {
    fontSize: '16px',
    fontWeight: 600,
    color: '#e0e0e0',
    marginBottom: '8px',
  },
  mediaMeta: {
    fontSize: '13px',
    color: '#666688',
    marginBottom: '16px',
  },
  clearButton: {
    backgroundColor: '#6b3a3a',
    color: '#ff6b6b',
    border: 'none',
    borderRadius: '6px',
    padding: '8px 24px',
    cursor: 'pointer',
    fontSize: '13px',
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
  },
};

export default App;
