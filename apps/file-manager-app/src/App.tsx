import { useState, useEffect, useCallback } from 'react';

const API_BASE: string = import.meta.env.VITE_API_URL || '';

interface FileItem {
  name: string;
  type: 'folder' | 'file';
  size: string;
  modified: string;
}

interface Directory {
  path: string;
  items: FileItem[];
}

interface UseFileSystemResult {
  directory: Directory | null;
  loading: boolean;
  error: string | null;
  navigate: (path: string) => void;
  refresh: () => void;
}

function useFileSystem(initialPath: string): UseFileSystemResult {
  const [currentPath, setCurrentPath] = useState(initialPath);
  const [directory, setDirectory] = useState<Directory | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchDirectory = useCallback(async (path: string) => {
    setLoading(true);
    setError(null);
    try {
      const params = new URLSearchParams({ path });
      const res = await fetch(`${API_BASE}/api/v1/files/list?${params}`, {
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
      });
      if (!res.ok) {
        throw new Error(`Failed to load directory: ${res.status} ${res.statusText}`);
      }
      const json = await res.json();
      const data = json.data ?? json;
      // Normalize API response (cortex-files FileEntry shape) into our Directory shape
      const rawItems: Record<string, unknown>[] = Array.isArray(data) ? data : (data.items ?? data.children ?? data.files ?? []);
      const items: FileItem[] = rawItems.map(
        (item: Record<string, unknown>) => ({
          name: String(item.name ?? ''),
          type: (item.is_directory as boolean) ? 'folder' as const : 'file' as const,
          size: String(item.size_bytes ?? item.size ?? '--'),
          modified: String(item.updated_at ?? item.modified ?? item.mtime ?? item.last_modified ?? '--'),
        }),
      );
      setDirectory({ path, items });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      setDirectory(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchDirectory(currentPath);
  }, [currentPath, fetchDirectory]);

  const navigate = useCallback((path: string) => {
    setCurrentPath(path);
  }, []);

  const refresh = useCallback(() => {
    fetchDirectory(currentPath);
  }, [currentPath, fetchDirectory]);

  return { directory, loading, error, navigate, refresh };
}

function App(): JSX.Element {
  const [currentPath, setCurrentPath] = useState('/');
  const { directory, loading, error, navigate, refresh } = useFileSystem(currentPath);

  const items = directory ? directory.items : [];

  const pathParts = currentPath === '/'
    ? ['/']
    : ['/', ...currentPath.split('/').filter(Boolean)];

  function navigateToFolder(name: string): void {
    const newPath = currentPath === '/' ? `/${name}` : `${currentPath}/${name}`;
    setCurrentPath(newPath);
    navigate(newPath);
  }

  function navigateToPath(path: string): void {
    const target = path === '/' ? '/' : path;
    setCurrentPath(target);
    navigate(target);
  }

  function buildBreadcrumbPath(index: number): string {
    if (index === 0) return '/';
    return '/' + pathParts.slice(1, index + 1).join('/');
  }

  function formatTimestamp(raw: string): string {
    if (raw === '--' || !raw) return '--';
    try {
      const d = new Date(raw);
      if (isNaN(d.getTime())) return raw;
      return d.toISOString().slice(0, 10);
    } catch (err) {
      console.error("Failed to parse timestamp:", err);
      return raw;
    }
  }

  function formatSize(raw: string): string {
    if (raw === '--' || !raw) return '--';
    const num = Number(raw);
    if (isNaN(num)) return raw;
    if (num === 0) return '0 B';
    if (num < 1024) return `${num} B`;
    if (num < 1024 * 1024) return `${(num / 1024).toFixed(1)} KB`;
    if (num < 1024 * 1024 * 1024) return `${(num / (1024 * 1024)).toFixed(1)} MB`;
    return `${(num / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }

  return (
    <div style={styles.container}>
      <div style={styles.breadcrumb}>
        {pathParts.map((part, index) => {
          const isLast = index === pathParts.length - 1;
          const display = part === '/' ? 'Root' : part;
          return (
            <span key={index} style={styles.breadcrumbPart}>
              {index > 0 && <span style={styles.breadcrumbSep}>/</span>}
              <span
                onClick={() => navigateToPath(buildBreadcrumbPath(index))}
                style={{
                  ...styles.breadcrumbLink,
                  ...(isLast ? styles.breadcrumbActive : {}),
                }}
              >
                {display}
              </span>
            </span>
          );
        })}
        <button onClick={refresh} style={styles.refreshBtn} title="Refresh">Refresh</button>
      </div>

      <div style={styles.listHeader}>
        <span style={{ ...styles.colName, ...styles.headerText }}>Name</span>
        <span style={{ ...styles.colSize, ...styles.headerText }}>Size</span>
        <span style={{ ...styles.colDate, ...styles.headerText }}>Modified</span>
      </div>

      <div style={styles.fileList}>
        {loading && (
          <div style={styles.emptyState}>Loading...</div>
        )}
        {error && (
          <div style={styles.errorState}>Error: {error}</div>
        )}
        {!loading && !error && items.length === 0 && (
          <div style={styles.emptyState}>This folder is empty</div>
        )}
        {!loading && !error && items.map((item) => (
          <div
            key={item.name}
            style={styles.fileRow}
            onClick={() => item.type === 'folder' && navigateToFolder(item.name)}
          >
            <span style={styles.colName}>
              <span style={styles.icon}>{item.type === 'folder' ? '\uD83D\uDCC1' : '\uD83D\uDCC4'}</span>
              <span style={{
                ...styles.itemName,
                ...(item.type === 'folder' ? styles.folderName : {}),
              }}>
                {item.name}
              </span>
            </span>
            <span style={{ ...styles.colSize, ...styles.itemMeta }}>{formatSize(item.size)}</span>
            <span style={{ ...styles.colDate, ...styles.itemMeta }}>{formatTimestamp(item.modified)}</span>
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
  },
  breadcrumb: {
    display: 'flex',
    alignItems: 'center',
    padding: '10px 16px',
    backgroundColor: '#0f0f23',
    borderBottom: '1px solid #2a2a4a',
    fontSize: '13px',
    flexWrap: 'wrap',
  },
  breadcrumbPart: {
    display: 'inline-flex',
    alignItems: 'center',
  },
  breadcrumbSep: {
    margin: '0 6px',
    color: '#444466',
  },
  breadcrumbLink: {
    color: '#a0a0c0',
    cursor: 'pointer',
    padding: '2px 4px',
    borderRadius: '3px',
  },
  breadcrumbActive: {
    color: '#e0e0e0',
    fontWeight: 600,
  },
  refreshBtn: {
    marginLeft: 'auto',
    padding: '3px 10px',
    backgroundColor: '#2a2a4a',
    color: '#a0a0c0',
    border: '1px solid #3a3a5a',
    borderRadius: '3px',
    cursor: 'pointer',
    fontSize: '12px',
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
  },
  listHeader: {
    display: 'flex',
    padding: '8px 16px',
    backgroundColor: '#12122a',
    borderBottom: '1px solid #2a2a4a',
  },
  headerText: {
    fontSize: '11px',
    fontWeight: 600,
    textTransform: 'uppercase' as const,
    letterSpacing: '0.5px',
    color: '#666688',
  },
  fileList: {
    flex: 1,
    overflowY: 'auto',
  },
  emptyState: {
    color: '#666688',
    textAlign: 'center',
    padding: '40px',
    fontSize: '14px',
  },
  errorState: {
    color: '#ff6b6b',
    textAlign: 'center',
    padding: '40px',
    fontSize: '14px',
  },
  fileRow: {
    display: 'flex',
    alignItems: 'center',
    padding: '8px 16px',
    borderBottom: '1px solid #1f1f3a',
    cursor: 'default',
  },
  colName: {
    flex: 3,
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  colSize: {
    flex: 1,
  },
  colDate: {
    flex: 1,
  },
  icon: {
    fontSize: '16px',
    flexShrink: 0,
  },
  itemName: {
    fontSize: '13px',
  },
  folderName: {
    fontWeight: 500,
    color: '#bb86fc',
  },
  itemMeta: {
    fontSize: '12px',
    color: '#666688',
  },
};

export default App;
