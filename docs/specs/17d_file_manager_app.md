# 17d. File Manager App

## 1. Purpose

The File Manager app provides the primary file system browser and management tool for CortexOS. It enables users to navigate, create, delete, rename, move, and organize files and directories stored in cortex-files through a graphical interface with both list and grid views, drag-and-drop operations, breadcrumbs navigation, and a trash system. It is classified as a P0 (priority-zero) app.

## 2. Scope

- Browse the cortex-files namespace with directory navigation.
- Create new files and directories.
- Delete files and directories (moved to trash at `/trash`).
- Rename files and directories.
- Move files and directories via drag-and-drop and cut/paste.
- File type icons: distinct icons for common file types (documents, images, code, audio, video, archives, directories).
- Breadcrumb navigation bar showing the current path with clickable segments.
- Dual view modes: list view (columns: name, size, modified date) and grid view (thumbnails + filename).
- Right-click context menu with file operations and Properties.
- Properties dialog showing: file name, type, size, created date, modified date, path.
- Trash: deleted items go to `/trash`. Trash can be emptied. Individual items can be restored.
- Double-click opens files with the default associated app.
- Sidebar with favorites (quick-access bookmarks to frequently used directories).
- App location: `apps/file-manager-app`.

## 3. Out of Scope

- File content preview (that is the role of individual apps).
- Archive creation or extraction (zip, tar, etc.).
- File sharing or permissions management UI (handled by system settings).
- Cloud storage sync or network file system browsing.
- Batch rename with patterns.
- File comparison or diff view.
- Disk usage analytics or storage visualization.
- File encryption or compression.
- Symbolic link creation.

## 4. Objectives

1. Provide a complete, intuitive file management interface for CortexOS.
2. Serve as the primary navigation surface for the user's file system.
3. Validate comprehensive cortex-files read/write/move/delete integration.
4. Demonstrate drag-and-drop, context menu, and multi-view UI patterns.
5. Establish the file-open behavior that other apps depend on (double-click to open in default app).

## 5. User-Visible Behavior

### 5.1 Layout

- Three-section layout: a sidebar on the left (200px, collapsible) with favorites and quick-access items, a toolbar at the top with navigation and view controls, and the main content area showing files and directories.
- The toolbar contains: back/forward buttons, breadcrumb path bar, view toggle (list/grid), and a "New" dropdown (New Folder, New File).
- The sidebar contains: a "Favorites" section with bookmarked directories, and "Quick Access" with entries for Home, Desktop, Documents, Downloads, and Trash.

### 5.2 Breadcrumb Navigation

- The breadcrumb bar shows the current path as clickable segments: `Home > Documents > Projects`.
- Clicking any segment navigates to that directory.
- The last segment (current directory) is displayed as bold text, not a link.
- Clicking the breadcrumb bar area (not a segment) allows typing a path directly.

### 5.3 List View

- Files displayed in a table with columns: Name (with icon), Size, Modified.
- Clicking a column header sorts by that column. Clicking again reverses sort order. Default sort: Name ascending.
- Directories are always listed before files regardless of sort column.
- Rows are full-width clickable. Single click selects. Double click opens (files) or navigates (directories).
- Selected row is highlighted. `Ctrl+Click` adds to selection. `Shift+Click` selects range.

### 5.4 Grid View

- Files displayed as a grid of cards. Each card shows: thumbnail/icon, filename below.
- Directories show a folder icon. Images show a thumbnail preview if under 2 MB.
- Grid card size: approximately 100x120px. Adjustable via zoom (`Ctrl+`/`Ctrl-` in grid view).
- Single click selects. Double click opens or navigates.
- Selection works the same as list view (Ctrl+Click, Shift+Click).

### 5.5 File Type Icons

| Category | Icon | Extensions |
|----------|------|------------|
| Directory | Folder icon | N/A |
| Document | Document icon | .txt, .pdf, .doc, .docx |
| Image | Image icon (or thumbnail) | .png, .jpg, .jpeg, .gif, .svg, .webp |
| Code | Code icon | .js, .ts, .py, .rs, .html, .css, .json |
| Audio | Audio icon | .mp3, .wav, .ogg, .flac |
| Video | Video icon | .mp4, .webm, .mkv |
| Archive | Archive icon | .zip, .tar, .gz, .rar |
| Generic | Generic file icon | All others |

### 5.6 Context Menu (Right-Click)

- **On file/directory**: Open, Rename, Move to Trash, Cut, Copy, Properties, Add to Favorites (directories only), Pin to Sidebar.
- **On empty space**: New Folder, New File, Paste, Sort by (submenu: Name, Size, Modified), Refresh.

### 5.7 Drag and Drop

- Dragging a file or directory onto a directory navigates the cursor into that directory on hover (after 500 ms), and drops the item into it on release.
- Dragging between files in the same directory initiates a reorder (visual only, does not affect file system sort).
- Dragging with Ctrl held copies instead of moves.
- Dragging to a sidebar favorite moves the item to that directory.

### 5.8 Trash

- Deleted items are moved to `/trash` via cortex-files.
- The Trash entry in the sidebar opens a special view showing trashed items with their original paths.
- Each trashed item has options: Restore (moves back to original location) and Delete Permanently.
- "Empty Trash" button in the trash view permanently deletes all trashed items with a confirmation dialog.
- Trash items show: original name, original path, deletion date, size.
- If the original location no longer exists on restore, the item is placed in the user's home directory.

### 5.9 Favorites

- Users can add directories to favorites via right-click "Add to Favorites" or by dragging a directory to the sidebar.
- Favorites are stored in cortex-files at `/favorites.json`.
- Favorite entries display the directory name. Clicking navigates to that directory.
- Favorites can be reordered by drag in the sidebar. Removed via right-click "Remove from Favorites".

### 5.10 Properties Dialog

- Triggered by right-click "Properties" on a file or directory.
- Modal dialog showing: Name, Type (file extension or "Directory"), Location (full path), Size (human-readable: KB, MB, GB), Created (date/time), Modified (date/time).
- For directories: shows item count (number of files and subdirectories).
- Read-only display. No inline editing (rename is done via the rename action).

### 5.11 Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New file |
| `Ctrl+Shift+N` | New folder |
| `Delete` | Move selected to trash |
| `F2` | Rename selected |
| `Ctrl+C` | Copy selected |
| `Ctrl+X` | Cut selected |
| `Ctrl+V` | Paste |
| `Ctrl+A` | Select all |
| `Backspace` | Navigate to parent directory |
| `Enter` | Open selected (file or directory) |
| `Ctrl+1` | Switch to list view |
| `Ctrl+2` | Switch to grid view |
| `F5` | Refresh current directory |
| `Ctrl+Z` | Undo last file operation |

## 6. System Behavior

### 6.1 File Operations

All file operations are routed through cortex-files client API:
- List directory: `cortex-files.files.list(path)`
- Create file: `cortex-files.files.createFile(path, "")`
- Create directory: `cortex-files.files.createDirectory(path)`
- Delete (to trash): `cortex-files.files.move(path, "/trash/{itemId}")` -- original path stored in trash metadata
- Rename: `cortex-files.files.rename(handle, newName)`
- Move: `cortex-files.files.move(sourcePath, destPath)`
- Copy: `cortex-files.files.read(handle)` then `cortex-files.files.createFile(destPath, content)`
- Get properties: `cortex-files.files.stat(path)`

### 6.2 Trash Metadata

- When an item is moved to trash, a metadata file is created at `/trash/.meta/{itemId}.json` containing: original path, original name, deletion timestamp.
- On restore, the metadata is read and the item is moved back to the original path.
- On empty trash, all items in `/trash/` and `/trash/.meta/` are permanently deleted.

### 6.3 Undo Support

- File Manager maintains an undo stack of the last 20 operations.
- Supported undo operations: move (rename, move to directory, drag-drop), delete (move to trash), create (new file/folder).
- Undo a delete restores from trash. Undo a move moves back to original location.
- Undo stack is session-scoped, not persisted.

### 6.4 Default App Opening

- When a file is double-clicked, the File Manager queries the runtime for the app registered to handle the file's extension.
- If a handler is registered, the file is opened in that app.
- If no handler is registered, a dialog shows: "No app is configured to open `.{ext}` files." with an "Open With..." option that lists available apps.
- Directories are always opened by navigating into them in the File Manager itself.

### 6.5 App Lifecycle

- Multi-instance app. Each window maintains independent navigation.
- Each instance tracks its own navigation history (back/forward stack), current path, and selection.
- On launch, the default view is the user's home directory.
- Session state (current path, view mode, sort preferences, sidebar width) is persisted via cortex-runtime.

### 6.6 Clipboard Operations

- Cut/Copy store file references (path + operation type) in an internal clipboard, not the system clipboard.
- Paste executes the operation (move for cut, copy for copy) at the current directory.
- Pasting in the same directory as the source appends " (copy)" to the filename.
- Clipboard is cleared on app close. It does not persist between sessions.

## 7. Architecture

```
apps/file-manager-app/
  manifest.json
  package.json
  src/
    main.ts                    # Registers app with runtime, file associations
    App.tsx                    # Root component, three-section layout
    components/
      Toolbar.tsx              # Navigation, breadcrumbs, view toggle, new menu
      Breadcrumbs.tsx          # Clickable path segments
      Sidebar.tsx              # Favorites and quick-access
      ListView.tsx             # Table view with sortable columns
      GridView.tsx             # Grid of file cards
      FileCard.tsx             # Individual file/directory card
      FileRow.tsx              # Individual file/directory table row
      ContextMenu.tsx          # Right-click context menu
      PropertiesDialog.tsx     # File/directory properties modal
      NewItemPrompt.tsx        # Inline new file/folder name input
      RenamePrompt.tsx         # Inline rename input
      TrashView.tsx            # Special view for trash contents
      EmptyTrashDialog.tsx     # Confirmation dialog for empty trash
    services/
      file-operations.ts       # cortex-files wrappers for all CRUD operations
      trash.ts                 # Trash move, restore, empty, metadata
      favorites.ts             # Favorites CRUD from /favorites.json
      navigation.ts            # Path resolution, navigation history stack
      file-icons.ts            # Extension-to-icon mapping
      clipboard.ts             # Internal cut/copy/paste logic
      undo.ts                  # Undo stack for file operations
    hooks/
      useFileManager.ts        # Main state: current path, items, selection, view mode
      useNavigation.ts         # Breadcrumb and back/forward history
      useSelection.ts          # Single, multi, range selection
      useDragDrop.ts           # Drag-and-drop state and handlers
      useContextMenu.ts        # Context menu position and visibility
      useClipboard.ts          # Cut/copy/paste clipboard state
    types.ts
  tests/
    unit/
      navigation.test.ts       # Path resolution, history stack, breadcrumbs
      trash.test.ts            # Move to trash, restore, empty, metadata
      favorites.test.ts        # CRUD operations
      clipboard.test.ts        # Cut/copy/paste logic
      undo.test.ts             # Undo stack, operation reversal
      file-icons.test.ts       # Extension mapping
    integration/
      file-ops.test.ts         # Create, rename, move, delete via cortex-files
      navigation-flow.test.ts  # Browse directories, back/forward, breadcrumbs
      drag-drop.test.ts        # Drag file to directory, copy with Ctrl
      trash-flow.test.ts       # Delete, view trash, restore, empty trash
      open-file.test.ts        # Double-click opens in default app
      lifecycle.test.ts        # App launch, multi-instance, session restore
```

No Rust backend crate needed. All logic is client-side TypeScript calling cortex-files APIs.

## 8. Data Model

### 8.1 File Entry

```typescript
interface FileEntry {
  name: string;
  path: string;                      // Full path
  isDirectory: boolean;
  size: number;                      // Bytes (0 for directories)
  modifiedAt: string;                // ISO 8601
  createdAt: string;                 // ISO 8601
  extension: string | null;          // null for directories
  mimeType: string | null;
}
```

### 8.2 Trash Item

```typescript
interface TrashItem {
  id: string;                        // Unique ID in trash
  name: string;                      // Original name
  originalPath: string;              // Original full path
  isDirectory: boolean;
  size: number;
  deletedAt: string;                 // ISO 8601
}
```

### 8.3 Favorite

```typescript
interface Favorite {
  id: string;                        // UUID
  name: string;                      // Display name
  path: string;                      // Full path
  order: number;                     // Sort order in sidebar
}
```

### 8.4 App State

```typescript
interface FileManagerState {
  currentPath: string;
  items: FileEntry[];
  selectedPaths: string[];           // Multiple selection
  viewMode: "list" | "grid";
  sortColumn: "name" | "size" | "modified";
  sortDirection: "asc" | "desc";
  navigationHistory: string[];       // Stack of visited paths
  navigationIndex: number;           // Current position in history
  clipboard: ClipboardState | null;
  sidebarWidth: number;              // px
  sidebarCollapsed: boolean;
  contextMenu: ContextMenuState | null;
  undoStack: UndoEntry[];
}
```

### 8.5 Manifest

```typescript
{
  id: "com.cortexos.file-manager",
  name: "Files",
  version: "1.0.0",
  description: "File system browser and manager",
  firstParty: true,
  bundled: true,
  entry: { frontend: "src/main.ts" },
  window: {
    defaultWidth: 900,
    defaultHeight: 600,
    minWidth: 500,
    minHeight: 350,
    resizable: true,
    singleInstance: false
  },
  permissions: {
    required: ["runtime.state", "runtime.lifecycle", "files.read", "files.write"],
    optional: ["ai.context", "ai.invoke"]
  },
  ai: {
    surfaces: { assistantPanel: true, contextMenu: false, inlineSuggestions: false },
    contextProviders: ["file-manager-context"],
    actions: []
  },
  accessibility: { highContrastSupport: true, screenReaderSupport: true, keyboardNavigation: true },
  category: "system"
}
```

## 9. Public Interfaces

### 9.1 AI Context

```typescript
interface FileManagerAIContext {
  currentPath: string;
  selectedCount: number;
  selectedTypes: string[];           // Extensions of selected files
  totalItems: number;
}
```

### 9.2 Commands Exposed

None. File Manager does not expose commands to other apps. Other apps open files through the runtime's file association system.

## 10. Internal Interfaces

### 10.1 File Operations Service

```typescript
interface FileOperations {
  listDirectory(path: string): Promise<FileEntry[]>;
  createFile(dirPath: string, name: string): Promise<FileEntry>;
  createDirectory(dirPath: string, name: string): Promise<FileEntry>;
  rename(path: string, newName: string): Promise<FileEntry>;
  move(sourcePath: string, destPath: string): Promise<void>;
  copy(sourcePath: string, destPath: string): Promise<void>;
  moveToTrash(path: string): Promise<TrashItem>;
  stat(path: string): Promise<FileEntry>;
}
```

### 10.2 Trash Service

```typescript
interface TrashService {
  listTrash(): Promise<TrashItem[]>;
  restoreFromTrash(itemId: string): Promise<void>;
  deletePermanently(itemId: string): Promise<void>;
  emptyTrash(): Promise<void>;
}
```

### 10.3 Navigation Service

```typescript
interface NavigationService {
  navigateTo(path: string, history: NavHistory): NavHistory;
  goBack(history: NavHistory): NavHistory;
  goForward(history: NavHistory): NavHistory;
  buildBreadcrumbs(path: string): BreadcrumbSegment[];
}

interface NavHistory {
  stack: string[];
  currentIndex: number;
}

interface BreadcrumbSegment {
  name: string;
  path: string;
  isLast: boolean;
}
```

## 11. State Management

- **Ephemeral**: Context menu position, drag state, inline rename input value, sort column hover state.
- **Session**: Current path, view mode, sort preferences, sidebar width, navigation history. Persisted via cortex-runtime session state.
- **Persistent**: Favorites stored at `/favorites.json` in cortex-files. Trash metadata stored at `/trash/.meta/`.
- State key per instance: `com.cortexos.file-manager.session.{instanceId}`.

## 12. Failure Modes and Error Handling

| Failure | Behavior |
|---------|----------|
| Directory not found | Show empty state: "This folder doesn't exist." with a "Go Home" button. |
| Permission denied on operation | Show error toast: "Permission denied." The operation is cancelled. Selection remains. |
| File already exists (rename/move) | Show error toast: "An item with this name already exists." Suggest appending "(1)". |
| cortex-files unavailable | Show error: "File system unavailable." with Retry button. App shows last cached state if available. |
| Trash restore fails (original path gone) | Restore to user's home directory. Show toast: "Restored to Home (original location not found)." |
| Empty trash partially fails | Show error toast with count: "Could not delete 3 items." Items that succeeded are removed. |
| File too large for thumbnail | Show file type icon instead of thumbnail. No error. |
| Clipboard paste fails | Show error toast: "Could not paste item." Undo stack is not affected. |
| Rename to empty string | Show inline validation: "Name cannot be empty." Rename input remains active. |
| Rename to invalid characters | Show inline validation: "Name cannot contain /, \\, :, *, ?, <, >, \|, \"". |

All errors are non-blocking. The file manager remains usable after any error.

## 13. Security and Permissions

- `files.read` and `files.write` are required. File Manager cannot function without file system access.
- Navigation is bounded to the user's cortex-files namespace. Path traversal above the user's root is not possible.
- File type icons are determined by extension mapping only. No content sniffing is performed.
- No executable files are launched directly. Double-click delegates to the runtime's file association system, which validates the target app.
- Delete operations move to trash first. Permanent deletion requires explicit action in the trash view.
- Copy/move operations validate destination paths to prevent overwriting without user consent.

## 14. Performance Requirements

- Directory listing render: under 200 ms for directories with up to 1,000 items.
- Sort operation: under 50 ms for 1,000 items (client-side sort of cached data).
- View switch (list to grid): under 100 ms (same data, different render).
- Thumbnail generation: under 500 ms per image. Thumbnails generated lazily on scroll into view.
- Drag-and-drop feedback: under 16 ms response time.
- Breadcrumb navigation: under 100 ms (no network call, just path string manipulation).
- Startup first meaningful paint: under 400 ms (with home directory loaded).
- Memory: directory listing cached for current and recently visited directories (max 10 directories). Approx 500 bytes per file entry.
- Bundle size: under 200 KB gzipped.

## 15. Accessibility Requirements

- File list items have `role="listbox"` with `role="option"` per item in both views.
- Grid view cards use `role="grid"` with `role="gridcell"`.
- Breadcrumb segments use `nav` with `aria-label="File path"`.
- Context menu has `role="menu"` with `role="menuitem"` per action.
- Toolbar buttons have `aria-label` descriptions (not relying on icons alone).
- Sidebar favorites have `role="navigation"` with `aria-label="Favorites"`.
- Selection state is announced to screen readers: "3 items selected."
- Keyboard navigation: Tab between sidebar, toolbar, content area. Arrow keys navigate items. Enter opens.
- Focus is visible on all interactive elements.
- Drag-and-drop has keyboard alternative (Ctrl+X/C/V).

## 16. Observability and Logging

Logged events:
- `file-manager.launched` (info) -- App opened. Payload: `{ startPath: "home" | "custom" }`.
- `file-manager.directory.opened` (info) -- Directory navigated to. Payload: `{ depth: number, itemCount: number }`. No path logged.
- `file-manager.file.opened` (info) -- File double-clicked. Payload: `{ extension: string }`. No filename or path.
- `file-manager.item.created` (info) -- File or folder created. Payload: `{ type: "file" | "directory" }`.
- `file-manager.item.deleted` (info) -- Item moved to trash.
- `file-manager.item.renamed` (info) -- Item renamed.
- `file-manager.item.moved` (info) -- Item moved.
- `file-manager.trash.emptied` (info) -- Trash emptied.
- `file-manager.error` (warn) -- Operation failed. Payload: `{ operation: string, errorType: string }`. No file paths.

No PII is logged. File paths, names, and content are never included in log payloads.

## 17. Testing Requirements

### 17.1 Unit Tests

- Navigation service: path resolution, breadcrumb generation, back/forward history stack.
- Trash service: move to trash metadata creation, restore path resolution, empty trash.
- Favorites: CRUD operations, ordering, persistence format.
- Clipboard: cut/copy/paste state, same-directory copy naming.
- Undo stack: operation recording, undo reversal, max 20 entries.
- File icons: extension-to-icon mapping for all supported categories.

### 17.2 Integration Tests

- Directory browsing: navigate into directory, back, forward, breadcrumb click.
- File CRUD: create file, rename, move to directory, delete (to trash).
- Trash workflow: delete item, view trash, restore item, verify original location, empty trash.
- Drag and drop: drag file to directory, drag with Ctrl to copy.
- View switch: verify items display correctly in list and grid views.
- Sort: verify sort by name, size, and modified date in both directions.
- Multi-selection: Ctrl+Click, Shift+Click, Ctrl+A.
- Open file: double-click delegates to runtime file association.
- Favorites: add directory, navigate via favorite, remove favorite.
- Multi-instance: two windows navigate independently.

### 17.3 Accessibility Tests

- AX tree validation for sidebar, toolbar, breadcrumbs, content area, context menu.
- Keyboard-only workflow: navigate directories, select files, create folder, rename, delete.
- Screen reader announcement of navigation and selection changes.

## 18. Acceptance Criteria

- [ ] Browse directories by double-click and breadcrumb navigation.
- [ ] Back/forward navigation maintains correct history.
- [ ] List view displays name, size, and modified date columns with sorting.
- [ ] Grid view displays icons/thumbnails with filenames.
- [ ] File type icons display correctly for all documented categories.
- [ ] Create new file and new folder in current directory.
- [ ] Rename file and directory via F2 and context menu.
- [ ] Move files via drag-and-drop to another directory.
- [ ] Copy files via Ctrl+drag to another directory.
- [ ] Delete moves item to trash. Trash view shows deleted items.
- [ ] Restore from trash returns item to original location.
- [ ] Empty trash permanently deletes all items with confirmation.
- [ ] Right-click context menu shows relevant options.
- [ ] Properties dialog shows correct metadata for files and directories.
- [ ] Double-click opens file in default associated app.
- [ ] Sidebar favorites: add, navigate, remove.
- [ ] Sidebar quick access: Home, Desktop, Documents, Downloads, Trash.
- [ ] Undo reverses the last file operation.
- [ ] App launches in under 400 ms.
- [ ] All three themes render correctly.
- [ ] Screen reader announces navigation and selection changes.
- [ ] Keyboard-only workflow completes all core operations.
- [ ] Unit test coverage >= 80%.

## 19. Build Order and Dependencies
**Layer 11**. Depends on: 09 (app runtime), 11 (filesystem), 16 (theme tokens), 17 (first-party app parent)

### Prerequisites

- Spec 17 parent (first-party app framework).
- `@cortexos/ui-components` (shared UI library).
- `@cortexos/runtime-client` (for state persistence, lifecycle, and file association lookup).
- `@cortexos/files-client` (for all file operations).
- `@cortexos/ai-client` (for AI surface).
- `@cortexos/theme` (design token consumer).

### Build Position

File Manager is the **first** first-party app to build (P0 priority). Other apps depend on its file-opening behavior and the file association system it exercises. It validates cortex-files integration most comprehensively.

No Rust crate needed. Pure frontend app.

## 20. Non-Goals and Anti-Patterns

### Non-Goals

- File content preview or editing.
- Archive management (zip, tar).
- Cloud storage integration.
- Batch rename with patterns.
- Disk usage visualization.
- File permissions UI.
- Symbolic link support.

### Anti-Patterns

- Accessing the filesystem directly instead of through cortex-files.
- Permanently deleting files without going through trash (except in trash view).
- Loading all directory contents at once for very large directories (use pagination or virtualization).
- Blocking the UI thread on file operations (all cortex-files calls must be async).
- Using raw file paths in log outputs (privacy).
- Implementing custom file type detection beyond extension mapping.

## 21. Implementation Instructions for Claude Code / Codex

### Subsystem Ownership

- File Manager owns: directory browsing UI, file operation orchestration, trash management, favorites, navigation history, context menu, drag-and-drop coordination, undo stack.
- File Manager does not own: actual file I/O (delegates to cortex-files), app launching (delegates to runtime), window management.

### Recommended Implementation Order

1. Create `manifest.json` and validate against FirstPartyAppManifest schema.
2. Implement `services/navigation.ts` -- path resolution, breadcrumb generation, history stack. Write comprehensive unit tests.
3. Implement `services/file-operations.ts` -- cortex-files wrappers. Write unit tests with mocked cortex-files.
4. Implement `services/file-icons.ts` -- extension-to-icon mapping. Write unit tests.
5. Implement `components/Breadcrumbs.tsx` and `components/Toolbar.tsx`.
6. Implement `components/ListView.tsx` with sortable columns and selection.
7. Implement `components/GridView.tsx` with file cards and selection.
8. Implement `components/Sidebar.tsx` with quick access.
9. Wire up `App.tsx` connecting navigation, views, and sidebar.
10. Implement `services/trash.ts` and `components/TrashView.tsx`.
11. Implement `services/favorites.ts` and integrate into sidebar.
12. Implement `components/ContextMenu.tsx` with all file operations.
13. Implement `services/clipboard.ts` and cut/copy/paste.
14. Implement `services/undo.ts` and undo support.
15. Implement `hooks/useDragDrop.ts` for drag-and-drop.
16. Implement `components/PropertiesDialog.tsx`.
17. Integrate `@cortexos/ai-client` for AI surface.
18. Accessibility audit and fixes.
19. Theme verification (light, dark, high-contrast).

### What Can Be Stubbed Initially

- AI context provider can return minimal data initially.
- Thumbnail generation for images can show file type icons initially, then add real thumbnails.
- Undo stack can be implemented as a simple array initially.

### What Must Be Real in V1

- Full directory browsing with list and grid views.
- File CRUD: create, rename, move, delete via cortex-files.
- Trash: delete-to-trash, restore, empty trash with confirmation.
- Breadcrumb navigation with clickable segments.
- Back/forward navigation history.
- Context menu with all documented options.
- Drag-and-drop file operations.
- Sidebar with favorites and quick access.
- Properties dialog.
- Double-click to open in default app.
- Sort by name, size, modified in both directions.
- Multi-selection (Ctrl+Click, Shift+Click, Ctrl+A).
- Theme support.
- Accessibility (keyboard navigation, screen reader).

### What Cannot Be Inferred

- Quick access paths (Home, Desktop, Documents, Downloads -- follow cortex-files user directory conventions).
- Default view mode (list view).
- Default sort (Name ascending, directories first).
- Grid card size (100x120px approx).
- Thumbnail size limit (2 MB).
- Maximum undo entries (20).
- Default window size (900x600 per manifest).
- Sidebar default width (200px).

### Stop Conditions

1. All unit tests pass with >= 80% coverage.
2. Navigation tests cover path resolution, back/forward, breadcrumbs.
3. File operation integration tests pass (create, rename, move, delete).
4. Trash workflow integration tests pass (delete, restore, empty).
5. Drag-and-drop works for move and copy operations.
6. Both view modes render correctly and support sorting.
7. Context menu shows correct options for files, directories, and empty space.
8. Double-click opens files in the registered default app.
9. All three themes render correctly.
10. Keyboard-only workflow test passes.

### Testing Gates

- Navigation service unit tests must pass before UI work begins.
- File operation integration tests must pass before merge.
- Trash workflow integration tests must pass before merge.
- Accessibility tests must pass before merge.
- Performance: directory with 1,000 items renders in under 200 ms.
