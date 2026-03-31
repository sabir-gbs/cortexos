# 17c. Notes App

## 1. Purpose

The Notes app provides a personal note-taking application for CortexOS with rich text editing, folder organization, search, pinning, and auto-save. It enables users to quickly capture and organize thoughts with a list-and-editor layout pattern, persisting all content through cortex-files.

## 2. Scope

- Create, edit, and delete notes.
- Rich text formatting: bold, italic, underline, ordered lists, unordered lists, hyperlinks, headings (H1, H2, H3).
- Folder organization: create, rename, delete folders. Move notes between folders.
- Full-text search across all notes. Search matches highlighted in results.
- Pin notes: pinned notes appear at the top of the list, marked with a pin icon.
- Auto-save with 2-second debounce after the last edit.
- Layout: sidebar list of notes on the left, editor pane on the right.
- Notes sorted by last-modified date (most recent first). Pinned notes always at top regardless of date.
- App location: `apps/notes-app`.

## 3. Out of Scope

- Collaborative note editing or sharing.
- Image or file attachment embedding in notes.
- Markdown rendering (rich text is stored as structured data, not Markdown).
- Tag system or labels.
- Note version history.
- Export to PDF or other formats.
- Drawing or handwriting support.
- Tables in notes.
- Code blocks with syntax highlighting.
- Note templates.

## 4. Objectives

1. Provide a fast, lightweight note-taking experience with essential rich text capabilities.
2. Validate structured content storage and retrieval through cortex-files.
3. Demonstrate a master-detail (list-editor) UI pattern within a first-party app.
4. Implement reliable auto-save that never loses user content.

## 5. User-Visible Behavior

### 5.1 Layout

- Two-pane layout: a sidebar (280px default width, resizable between 200px and 400px) showing the note list, and the main editor pane occupying the remaining space.
- The sidebar contains: a search bar at the top, a folder selector (dropdown or collapsible tree), and a scrollable list of notes.
- A "New Note" button is prominently placed at the top of the sidebar.
- The editor pane shows the selected note's title (editable heading field) and body (rich text editor area).
- When no note is selected, the editor pane shows an empty state: "Select a note or create a new one."

### 5.2 Note List

- Each note in the list displays: title (first line or user-set title), preview snippet (first 80 characters of body, stripped of formatting), last-modified date, and pin icon if pinned.
- Notes are sorted: pinned notes first (sorted by pin date, most recent pin first), then unpinned notes sorted by last-modified date (most recent first).
- Clicking a note selects it and loads it in the editor.
- Right-click context menu on a note: Pin/Unpin, Move to Folder, Delete.
- Double-clicking a note title in the list allows inline rename.

### 5.3 Rich Text Editor

- Toolbar at the top of the editor provides formatting buttons: Bold (Ctrl+B), Italic (Ctrl+I), Underline (Ctrl+U), Heading dropdown (H1/H2/H3/Normal), Ordered List, Unordered List, Link (Ctrl+K).
- Formatting is applied to the selection or toggled at the cursor position.
- Link insertion: clicking the Link button or pressing Ctrl+K opens an inline prompt for URL. Selected text becomes the link text. If no text is selected, the URL is inserted as link text.
- Headings apply block-level formatting to the current paragraph.
- Lists: pressing Enter in a list item creates a new list item. Pressing Enter on an empty list item exits the list.
- The editor uses a contenteditable div with structured document model, not a raw textarea.

### 5.4 Folders

- Default folders: "All Notes" (virtual, shows all notes), "Unfiled" (notes not in any folder).
- Users can create custom folders via a "New Folder" button.
- Folder list is shown above the note list in the sidebar as a collapsible section.
- Clicking a folder filters the note list to that folder.
- Notes can be moved between folders via right-click "Move to Folder" or drag-and-drop in the note list.
- Deleting a folder moves its notes to "Unfiled". The folder itself is deleted. A confirmation prompt is shown.

### 5.5 Search

- The search bar at the top of the sidebar filters notes in real time as the user types.
- Search matches note titles and body content (plain text, stripping formatting).
- Matching notes remain in the list with the search term highlighted in the title and snippet preview.
- Clearing the search bar restores the full note list.
- Search is scoped to the currently selected folder, or "All Notes" if no folder is selected.

### 5.6 Auto-Save

- Edits are auto-saved 2 seconds after the last keystroke or formatting change.
- A save indicator in the editor toolbar shows: "Saving..." during save, "Saved" after completion, or "Save failed" on error.
- The user does not need to manually save. There is no Save button.
- Switching to a different note triggers an immediate save of the current note before switching.

### 5.7 Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New note |
| `Ctrl+B` | Bold |
| `Ctrl+I` | Italic |
| `Ctrl+U` | Underline |
| `Ctrl+K` | Insert/edit link |
| `Ctrl+F` | Focus search bar |
| `Delete` | Delete selected note (with confirmation) |
| `Ctrl+Shift+P` | Pin/Unpin selected note |
| `Escape` | Clear search / deselect note |

## 6. System Behavior

### 6.1 Storage

- Notes are stored as individual files in cortex-files under the path `/notes/`.
- Each note is stored as a JSON file: `/notes/{noteId}.json`.
- Folder metadata is stored in `/notes/.folders.json` as an array of folder definitions.
- The rich text body is stored as a serialized document model (array of block nodes with inline formatting), not as HTML.
- The auto-save debounce timer resets on every edit. The save operation is asynchronous and does not block the UI.

### 6.2 Note Document Model

- The note body is modeled as an array of blocks. Each block has a type (paragraph, heading, list-item) and inline content (text spans with formatting marks).
- Bold, italic, and underline are boolean marks on text spans. Links are mark objects with a URL property.
- Headings have a level (1, 2, or 3).
- Lists are represented as consecutive list-item blocks. Ordered and unordered list items are distinguished by a listType property on the block.

### 6.3 Search Implementation

- Search is performed client-side by iterating over all loaded notes.
- The search is case-insensitive.
- The note index (id, title, snippet, folder, pinned, lastModified) is kept in memory for fast filtering.
- Full note content is loaded on demand when a note is selected. Search queries check the in-memory index first for title matches, then load full content for body matches if needed.
- Search results update with each keystroke (debounced at 150 ms).

### 6.4 App Lifecycle

- Single-instance app. Launching a second instance brings the existing window to focus.
- On launch, the note index is loaded from cortex-files. The most recently modified note is auto-selected.
- On close, any pending auto-save is flushed immediately before the window closes.
- Session state (selected note, selected folder, sidebar width, scroll positions) is persisted via cortex-runtime.

### 6.5 Trash and Deletion

- Deleting a note moves it to `/trash/notes/{noteId}.json` via cortex-files.
- A confirmation dialog is shown: "Delete '<title>'?" with Delete / Cancel.
- Deleted notes are permanently removed from the index. Recovery is via the system trash if cortex-files supports it.

## 7. Architecture

```
apps/notes-app/
  manifest.json
  package.json
  src/
    main.ts                    # Registers app with runtime
    App.tsx                    # Root component, two-pane layout
    components/
      Sidebar.tsx              # Note list, search, folders
      NoteList.tsx             # Scrollable note list with sort and pin
      NoteCard.tsx             # Individual note entry in the list
      SearchBar.tsx            # Search input with clear button
      FolderTree.tsx           # Folder selector (collapsible)
      EditorPane.tsx           # Right-side editor area
      NoteEditor.tsx           # Rich text editor (contenteditable)
      EditorToolbar.tsx        # Formatting toolbar
      SaveIndicator.tsx        # Saving/Saved/Failed indicator
      EmptyState.tsx           # No-note-selected placeholder
      LinkPrompt.tsx           # Inline URL input for link insertion
      DeleteConfirmDialog.tsx  # Confirmation dialog for note deletion
    services/
      note-storage.ts          # cortex-files read/write for notes
      folder-storage.ts        # cortex-files read/write for folders
      search.ts                # Full-text search across note index
      document-model.ts        # Rich text document model, serialization
      auto-save.ts             # Debounced save logic (2s)
    hooks/
      useNotes.ts              # Note CRUD, index management
      useFolders.ts            # Folder CRUD
      useSearch.ts             # Search state and filtering
      useAutoSave.ts           # Auto-save timer and flush
      useEditorState.ts        # Editor formatting state
    types.ts
  tests/
    unit/
      document-model.test.ts   # Block model, serialization, formatting
      search.test.ts           # Search filtering, case-insensitivity, snippets
      auto-save.test.ts        # Debounce timing, flush on switch
      note-storage.test.ts     # Read/write serialization
    integration/
      note-crud.test.ts        # Create, edit, delete, move flow
      folder-crud.test.ts      # Create folder, move note, delete folder
      search-flow.test.ts      # Search across notes, folder-scoped search
      auto-save-flow.test.ts   # Edit, wait, verify persisted content
      lifecycle.test.ts        # App launch, single instance, close with unsaved
```

No Rust backend crate needed. All logic is client-side TypeScript.

## 8. Data Model

### 8.1 Note

```typescript
interface Note {
  id: string;                       // UUID
  title: string;                    // First line or user-set title
  body: NoteBlock[];                // Rich text content as block array
  folderId: string | null;          // null = Unfiled
  pinned: boolean;
  createdAt: string;                // ISO 8601
  updatedAt: string;                // ISO 8601
}
```

### 8.2 Note Block

```typescript
interface NoteBlock {
  id: string;
  type: "paragraph" | "heading" | "list-item";
  level?: 1 | 2 | 3;               // Only for heading type
  listType?: "ordered" | "unordered"; // Only for list-item type
  content: InlineSpan[];
}

interface InlineSpan {
  text: string;
  marks: Mark[];
}

type Mark =
  | { type: "bold" }
  | { type: "italic" }
  | { type: "underline" }
  | { type: "link"; url: string };
```

### 8.3 Folder

```typescript
interface Folder {
  id: string;                       // UUID
  name: string;
  createdAt: string;                // ISO 8601
}
```

### 8.4 Note Index Entry

```typescript
interface NoteIndexEntry {
  id: string;
  title: string;
  snippet: string;                  // First 80 chars of plain text
  folderId: string | null;
  pinned: boolean;
  updatedAt: string;
}
```

### 8.5 App State

```typescript
interface NotesAppState {
  notes: NoteIndexEntry[];
  folders: Folder[];
  selectedNoteId: string | null;
  selectedFolderId: string | null;  // null = "All Notes"
  searchQuery: string;
  sidebarWidth: number;             // px
  saveStatus: "idle" | "saving" | "saved" | "error";
}
```

### 8.6 Manifest

```typescript
{
  id: "com.cortexos.notes",
  name: "Notes",
  version: "1.0.0",
  description: "Personal note-taking with rich text and folders",
  firstParty: true,
  bundled: true,
  entry: { frontend: "src/main.ts" },
  window: {
    defaultWidth: 900,
    defaultHeight: 600,
    minWidth: 600,
    minHeight: 400,
    resizable: true,
    singleInstance: true
  },
  permissions: {
    required: ["runtime.state", "runtime.lifecycle", "files.read", "files.write"],
    optional: ["ai.context", "ai.invoke"]
  },
  ai: {
    surfaces: { assistantPanel: true, contextMenu: false, inlineSuggestions: false },
    contextProviders: ["notes-context"],
    actions: [
      {
        id: "summarize-note",
        label: "Summarize this note",
        description: "Provides a summary of the currently open note",
        confirmationRequired: false,
        destructive: false
      }
    ]
  },
  accessibility: { highContrastSupport: true, screenReaderSupport: true, keyboardNavigation: true },
  category: "productivity"
}
```

## 9. Public Interfaces

### 9.1 AI Context

```typescript
interface NotesAIContext {
  noteTitle: string;
  noteSnippet: string;              // First 500 chars plain text
  folderName: string | null;
  noteCount: number;
}
```

### 9.2 Commands Exposed

None. Notes does not expose commands to other apps.

## 10. Internal Interfaces

### 10.1 Note Storage

```typescript
interface NoteStorage {
  loadIndex(): Promise<NoteIndexEntry[]>;
  loadNote(id: string): Promise<Note>;
  saveNote(note: Note): Promise<void>;
  deleteNote(id: string): Promise<void>;
  moveNote(id: string, folderId: string | null): Promise<void>;
}
```

### 10.2 Folder Storage

```typescript
interface FolderStorage {
  loadFolders(): Promise<Folder[]>;
  createFolder(name: string): Promise<Folder>;
  renameFolder(id: string, name: string): Promise<void>;
  deleteFolder(id: string): Promise<void>;  // Moves notes to Unfiled
}
```

### 10.3 Document Model

```typescript
interface DocumentModel {
  toPlainText(blocks: NoteBlock[]): string;
  fromPlainText(text: string): NoteBlock[];
  serialize(blocks: NoteBlock[]): string;    // JSON string
  deserialize(json: string): NoteBlock[];
  getSnippet(blocks: NoteBlock[], maxLength: number): string;
}
```

### 10.4 Search

```typescript
interface NoteSearch {
  search(query: string, notes: NoteIndexEntry[], scope: string | null): NoteIndexEntry[];
}
```

## 11. State Management

- **Ephemeral**: Editor toolbar button active states, search bar focus, context menu position, link prompt open/close.
- **Session**: Selected note ID, selected folder ID, sidebar width, scroll positions, save status. Persisted via cortex-runtime session state.
- **Persistent**: Notes and folders stored on cortex-files. The note index is rebuilt from cortex-files on each launch.
- State key: `com.cortexos.notes.session`.

## 12. Failure Modes and Error Handling

| Failure | Behavior |
|---------|----------|
| cortex-files unavailable on launch | Show error: "Cannot load notes. File system unavailable." with Retry button. App remains open but non-functional until retry succeeds. |
| Note load failure | Show error toast: "Could not open note." The note appears grayed out in the list. Other notes remain accessible. |
| Auto-save failure | Show "Save failed" indicator. Retry auto-save on next edit. After 3 consecutive failures, show persistent warning: "Changes may not be saved." |
| Folder metadata corruption | Rebuild folders from defaults (All Notes, Unfiled). Log as warning. Custom folders are lost; notes remain intact. |
| Note content corruption | Show error: "Could not parse note content." Offer to view raw content or delete the note. Other notes unaffected. |
| Search error | Fall back to title-only search. Log as warning. |
| Disk space full on save | Show error: "Could not save. Storage may be full." Auto-save retries on next edit. |

All errors are non-blocking where possible. The app remains partially usable during failures.

## 13. Security and Permissions

- `files.read` and `files.write` are required. Notes cannot function without file system access.
- Note content is private to the user. No note content is transmitted externally unless the user invokes an AI action.
- Notes storage path (`/notes/`) is within the user's cortex-files namespace. Other apps cannot access it without the user's files permission.
- Folder metadata file (`.folders.json`) uses a dot-prefix convention to distinguish it from note files.
- No executable content in notes. Rich text is stored as structured data, not HTML, preventing XSS.

## 14. Performance Requirements

- Note list render: under 16 ms for up to 500 notes in the sidebar.
- Note open: under 200 ms to load and render a note with up to 10,000 characters.
- Auto-save: under 500 ms for a note up to 50 KB.
- Search: under 100 ms to filter across 500 notes (index-based).
- Typing latency: under 16 ms (one frame) in the rich text editor.
- Startup first meaningful paint: under 400 ms (with note index loaded).
- Memory: note index cached in memory (approx 1 KB per note). Full note content loaded for active note only.
- Bundle size: under 200 KB gzipped.

## 15. Accessibility Requirements

- Sidebar note list items have `role="listbox"` with `role="option"` per note.
- Rich text editor has `role="textbox"` with `aria-multiline="true"` and `aria-label="Note editor"`.
- Formatting toolbar buttons have `aria-pressed` state (bold, italic, underline) and `aria-label` descriptions.
- Folder tree items have `role="treeitem"` within `role="tree"`.
- Search bar has `aria-label="Search notes"`.
- Pin icon has `aria-label="Pinned"` visible to screen readers.
- Keyboard navigation: Tab navigates between sidebar, search, editor. Arrow keys navigate note list and folders.
- Focus is visible on all interactive elements.
- Color is not the sole indicator of formatting state (bold button uses pressed state, not color change).

## 16. Observability and Logging

Logged events:
- `notes.launched` (info) -- App opened. Payload: `{ noteCount: number, folderCount: number }`.
- `notes.created` (info) -- Note created.
- `notes.deleted` (info) -- Note deleted.
- `notes.folder.created` (info) -- Folder created.
- `notes.folder.deleted` (info) -- Folder deleted.
- `notes.search.performed` (debug) -- Search executed. Payload: `{ resultCount: number }`. No search query logged.
- `notes.autosave.completed` (debug) -- Auto-save completed.
- `notes.autosave.failed` (warn) -- Auto-save failed.
- `notes.error` (warn) -- Storage or parsing error. Payload: `{ errorType: string }`. No note content.
- `notes.ai.summarize_invoked` (info) -- AI summarize action triggered.

No PII is logged. Note titles, content, and search queries are never included in log payloads.

## 17. Testing Requirements

### 17.1 Unit Tests

- Document model: block creation, inline formatting, serialization/deserialization round-trip, plain text extraction, snippet generation.
- Search: case-insensitive matching, folder scoping, empty results, special characters, empty query.
- Auto-save: debounce timing verification (2s), flush on note switch, flush on app close, failure retry.
- Note storage: load/save serialization, delete, move between folders.
- Folder storage: CRUD operations, delete folder reassigns notes to Unfiled.

### 17.2 Integration Tests

- Full note lifecycle: create note, edit with formatting, auto-save, reload, verify content.
- Folder workflow: create folder, create note in folder, move note between folders, delete folder (notes reassigned).
- Search workflow: create multiple notes, search, verify filtered results, clear search.
- Pin workflow: create notes, pin one, verify pinned note is first, unpin, verify sort order.
- Multi-edit: edit note A, switch to note B, verify note A was auto-saved, switch back, verify content.

### 17.3 Accessibility Tests

- AX tree validation for sidebar, editor, toolbar, search bar.
- Keyboard-only workflow: create note, format text, create folder, move note, search, pin.
- Screen reader announcement of save status changes.

## 18. Acceptance Criteria

- [ ] Create a new note with auto-generated title (first line of content).
- [ ] Edit note with bold, italic, underline formatting, persisted correctly.
- [ ] Insert headings (H1, H2, H3) that render at correct sizes.
- [ ] Create ordered and unordered lists with proper Enter-to-add behavior.
- [ ] Insert a hyperlink via toolbar and Ctrl+K.
- [ ] Notes auto-save within 2 seconds of last edit.
- [ ] Switching notes triggers immediate save of the current note.
- [ ] Save indicator shows correct state: Saving / Saved / Failed.
- [ ] Note list sorted by last-modified, pinned notes at top.
- [ ] Search filters notes in real-time, highlighting matches.
- [ ] Create, rename, and delete folders.
- [ ] Move notes between folders via right-click menu.
- [ ] Delete note shows confirmation and moves to trash.
- [ ] Delete folder reassigns notes to Unfiled with confirmation.
- [ ] Sidebar resizable between 200px and 400px.
- [ ] App launches in under 400 ms.
- [ ] All three themes render correctly.
- [ ] Screen reader announces save status and formatting changes.
- [ ] Unit test coverage >= 80%.

## 19. Build Order and Dependencies
**Layer 11**. Depends on: 09 (app runtime), 16 (theme tokens), 17 (first-party app parent)

### Prerequisites

- Spec 17 parent (first-party app framework).
- `@cortexos/ui-components` (shared UI library).
- `@cortexos/runtime-client` (for state persistence and lifecycle).
- `@cortexos/files-client` (for note and folder storage).
- `@cortexos/ai-client` (for AI surface).
- `@cortexos/theme` (design token consumer).

### Build Position

Notes is the **fifth** first-party app to build (after Clock Utilities, Calculator, Terminal Lite, and Text Editor). It validates structured content storage and the master-detail UI pattern.

No Rust crate needed. Pure frontend app.

## 20. Non-Goals and Anti-Patterns

### Non-Goals

- Collaborative editing or note sharing.
- Image or file attachments in notes.
- Tagging system.
- Version history for notes.
- Export to external formats.
- Markdown input/rendering.
- Drawing or handwriting.

### Anti-Patterns

- Storing rich text as raw HTML (use structured document model to prevent XSS and ensure portability).
- Saving on every keystroke without debounce (causes excessive cortex-files writes).
- Loading all note bodies into memory at once (load index only, fetch body on demand).
- Using `eval()` or `innerHTML` for rendering rich text (use React components from document model).
- Blocking the UI thread during save operations.

## 21. Implementation Instructions for Claude Code / Codex

### Subsystem Ownership

- Notes owns: document model, rich text editor, note/folder CRUD, auto-save logic, search filtering, sidebar layout.
- Notes does not own: file system access (delegates to cortex-files), window management, theme system.

### Recommended Implementation Order

1. Create `manifest.json` and validate against FirstPartyAppManifest schema.
2. Implement `services/document-model.ts` -- block model, inline spans, serialization, plain text extraction. Write comprehensive unit tests.
3. Implement `services/note-storage.ts` -- load index, load/save/delete note via cortex-files. Write unit tests with mocked cortex-files.
4. Implement `services/folder-storage.ts` -- folder CRUD via cortex-files. Write unit tests.
5. Implement `services/auto-save.ts` -- debounce timer, flush, retry. Write unit tests.
6. Implement `services/search.ts` -- index-based filtering. Write unit tests.
7. Implement `hooks/useNotes.ts` and `hooks/useFolders.ts` -- state management for notes and folders.
8. Implement `components/Sidebar.tsx`, `NoteList.tsx`, `NoteCard.tsx`, `SearchBar.tsx`.
9. Implement `components/NoteEditor.tsx` with contenteditable rich text editing.
10. Implement `components/EditorToolbar.tsx` with formatting buttons.
11. Wire up `App.tsx` connecting sidebar, editor, and data layer.
12. Implement auto-save integration with the editor.
13. Implement folder UI: `components/FolderTree.tsx`, create/rename/delete.
14. Implement pin, move-to-folder, and delete context menu actions.
15. Integrate `@cortexos/ai-client` for AI surface.
16. Accessibility audit and fixes.
17. Theme verification (light, dark, high-contrast).

### What Can Be Stubbed Initially

- AI "Summarize note" action can return a placeholder summary initially.
- Link editing can start with URL-only insertion (no edit-existing-link flow) initially.
- Folder tree can be a flat list initially, refined to collapsible tree later.

### What Must Be Real in V1

- Full rich text editing with bold, italic, underline, headings, lists, and links.
- Document model with correct serialization/deserialization.
- Auto-save with 2-second debounce and flush-on-switch.
- Note CRUD with folder organization.
- Search across all notes with highlighting.
- Pin/unpin with correct sort order.
- Delete confirmation dialogs.
- Theme support.
- Accessibility (keyboard navigation, screen reader).

### What Cannot Be Inferred

- Sidebar default width (280px per layout spec).
- Sidebar resize range (200px to 400px).
- Snippet preview length (80 characters).
- Auto-save debounce duration (2 seconds).
- Search debounce duration (150 ms).
- Maximum notes before performance concern (500).
- Default window size (900x600 per manifest).

### Stop Conditions

1. All unit tests pass with >= 80% coverage.
2. Document model serialization round-trips correctly for all supported block and mark types.
3. Auto-save fires at 2-second debounce and flushes on note switch.
4. Integration tests for full note lifecycle (create, edit, save, reload, delete) pass.
5. Search returns correct results with folder scoping.
6. Pin sort order is correct (pinned first, then by modified date).
7. No raw HTML storage confirmed (document model used for serialization).
8. All three themes render correctly.
9. Performance: note list renders 500 notes without jank.

### Testing Gates

- Document model unit tests must pass before editor UI work begins.
- Auto-save unit tests must pass before editor integration.
- Note CRUD integration tests must pass before merge.
- Search integration tests must pass before merge.
- Accessibility tests must pass before merge.
