# 17b. Text Editor App

## 1. Purpose

The Text Editor app provides a lightweight plain-text editor for CortexOS, supporting syntax-highlighted editing of common file types with find/replace, multi-tab editing, and integration with cortex-files for save/load operations. It is explicitly not a full IDE -- it provides fast, reliable text editing without project management, build tooling, or debugging features.

## 2. Scope

- Plain text editing with syntax highlighting for: JavaScript, TypeScript, Python, Rust, HTML, CSS, JSON, and Markdown.
- Line numbers displayed alongside the editor gutter.
- Find and replace: `Ctrl+F` opens find bar, `Ctrl+H` opens find-and-replace bar with replace/replace-all actions.
- Multi-tab editing: open multiple files in tabs, switch between them.
- File open/save via cortex-files integration. New file creation supported.
- Encoding: UTF-8 by default. No encoding conversion in V1.
- Word count, line count, and character count displayed in the status bar.
- Monospace font throughout the editor area.
- Zoom: `Ctrl+`/`Ctrl-` to increase/decrease font size. `Ctrl+0` resets to default.
- Auto-indentation for supported languages (basic heuristic-based).
- Undo/redo: `Ctrl+Z`/`Ctrl+Shift+Z`.
- App location: `apps/text-editor-app`.

## 3. Out of Scope

- Integrated terminal, debugger, or build system.
- Git integration or version control UI.
- Code completion, IntelliSense, or language server protocol.
- Project/workspace management.
- Plugin or extension system.
- Split editor panes or side-by-side diff view.
- Encoding conversion beyond UTF-8.
- Binary file editing.
- Collaborative real-time editing.
- Minimap or code folding.
- Snippet expansion or macro recording.
- Image or rich content embedding.

## 4. Objectives

1. Provide a fast, reliable plain-text editor that handles common programming and markup languages.
2. Validate cortex-files integration for read and write of text documents.
3. Demonstrate multi-tab UI pattern within a first-party app.
4. Serve as the default file handler for `.txt`, `.js`, `.ts`, `.py`, `.rs`, `.html`, `.css`, `.json`, and `.md` files opened from the File Manager.

## 5. User-Visible Behavior

### 5.1 Editor Area

- Central editing area with a monospace font, left-aligned text, and a line-number gutter.
- Cursor is a blinking vertical line. Selection is highlighted with the theme's selection color.
- Line numbers are displayed in the gutter, right-aligned, in a muted color. Current line number is emphasized.
- Text wraps at the editor boundary when word wrap is enabled (toggle via View menu, default off). When word wrap is off, horizontal scrollbar appears.
- Tab key inserts spaces (configurable: 2 or 4 spaces, default 2). `Shift+Tab` outdents.

### 5.2 Tabs

- A tab bar at the top shows open files. Each tab displays the filename.
- Modified (unsaved) files show a dot indicator on the tab.
- Clicking a tab switches to that file. Middle-click closes the tab.
- Closing a modified file prompts: "Save changes to `<filename>`?" with Save / Don't Save / Cancel.
- Tabs can be reordered by drag.
- Maximum open tabs: 20. Attempting to open more shows a toast: "Maximum 20 tabs open."

### 5.3 Find and Replace

- `Ctrl+F` opens a find bar below the tab bar. Input field with previous/next buttons, match count display, and close button.
- `Ctrl+H` extends the find bar with a replace field, Replace and Replace All buttons.
- Matches are highlighted in the editor. Current match has a distinct highlight.
- Options: case-sensitive toggle, whole-word toggle, regex toggle. All default off.
- `Enter` moves to next match. `Shift+Enter` moves to previous match.
- Escape closes the find/replace bar.

### 5.4 Status Bar

- Bottom status bar displays: line number, column number, selection count, language/mode, encoding (UTF-8), word count, line count, character count.
- When text is selected, the status bar shows the number of selected characters and lines.

### 5.5 Zoom

- `Ctrl+` increases font size by 1px (min 10px, max 32px).
- `Ctrl-` decreases font size by 1px (min 10px, max 32px).
- `Ctrl+0` resets to default font size (14px).
- Zoom level is per-session, not persisted.

### 5.6 Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New untitled file |
| `Ctrl+O` | Open file (via cortex-files picker) |
| `Ctrl+S` | Save current file |
| `Ctrl+Shift+S` | Save as (via cortex-files picker) |
| `Ctrl+W` | Close current tab |
| `Ctrl+Tab` | Switch to next tab |
| `Ctrl+Shift+Tab` | Switch to previous tab |
| `Ctrl+F` | Open find bar |
| `Ctrl+H` | Open find and replace bar |
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |
| `Ctrl++` | Zoom in |
| `Ctrl+-` | Zoom out |
| `Ctrl+0` | Reset zoom |
| `Ctrl+G` | Go to line (prompt for line number) |

## 6. System Behavior

### 6.1 File Operations

- Open: reads file content from cortex-files. File handle is stored per tab for subsequent saves.
- Save: writes current content back to cortex-files using the stored handle. Clears the modified indicator on success.
- Save As: prompts for a new path via cortex-files picker, writes content, updates the tab's file handle and filename.
- New: creates an untitled tab with empty content and no file handle. First save triggers Save As behavior.
- Auto-save is not implemented. User must save explicitly.

### 6.2 Syntax Highlighting

- Syntax highlighting is performed client-side using a tokenizer for each supported language.
- Highlighting applies semantic token classes: keyword, string, comment, number, type, function, operator, punctuation, tag, attribute, property. Colors are derived from the active theme's design tokens.
- Highlighting is incremental: on edit, only the changed lines and affected surrounding lines are re-tokenized.
- Large files (over 100 KB) disable syntax highlighting and show a notification: "Syntax highlighting disabled for large file."
- File type is detected by extension. Unknown extensions render as plain text.

### 6.3 Undo/Redo

- Undo/redo stack is maintained per tab. Maximum 500 undo steps per tab.
- Closing a tab clears its undo/redo stack.
- Save does not clear the undo stack.

### 6.4 App Lifecycle

- Multi-instance app. Each window maintains its own set of tabs.
- On close, if any tab has unsaved changes, a dialog lists all modified files with Save / Don't Save / Cancel.
- Session state (open tabs, cursor positions, scroll positions) is persisted via cortex-runtime session state for hot-reload recovery.
- File contents are not stored in session state -- they are re-read from cortex-files on reload.

### 6.5 File Associations

Registered file handlers:
- `.txt` -> Text Editor
- `.js` -> Text Editor
- `.ts` -> Text Editor
- `.py` -> Text Editor
- `.rs` -> Text Editor
- `.html` -> Text Editor
- `.css` -> Text Editor
- `.json` -> Text Editor
- `.md` -> Text Editor

### 6.6 Go to Line

- `Ctrl+G` opens a small inline prompt: "Go to line:" with a number input.
- Entering a valid line number scrolls the editor to that line and places the cursor at the beginning.
- Entering an invalid number (out of range) shows a brief error toast.

## 7. Architecture

```
apps/text-editor-app/
  manifest.json
  package.json
  src/
    main.ts                    # Registers app with runtime, file associations
    App.tsx                    # Root component, tab management
    components/
      TabBar.tsx               # Tab strip with reorder, close, modified indicators
      Editor.tsx               # Main editor component (wraps editor engine)
      FindReplaceBar.tsx       # Find/replace UI
      StatusBar.tsx            # Bottom status bar
      GoToLinePrompt.tsx       # Inline go-to-line input
    services/
      editor-engine.ts         # Core text editing: cursor, selection, insert, delete, undo/redo
      tokenizer.ts             # Syntax tokenizer dispatcher
      tokenizers/
        javascript.ts          # JS/TS tokenizer
        python.ts              # Python tokenizer
        rust.ts                # Rust tokenizer
        html.ts                # HTML tokenizer
        css.ts                 # CSS tokenizer
        json.ts                # JSON tokenizer
        markdown.ts            # Markdown tokenizer
      file-ops.ts              # cortex-files read/write wrappers
      text-utils.ts            # Word count, line count, character count
    hooks/
      useTabs.ts               # Tab state management
      useEditor.ts             # Editor state: content, cursor, scroll, undo/redo
      useFindReplace.ts        # Find/replace state and search logic
      useZoom.ts               # Font size state
    types.ts
  tests/
    unit/
      editor-engine.test.ts    # Insert, delete, undo, redo, selection
      tokenizer.test.ts        # Token output for each language
      text-utils.test.ts       # Word/line/char counting
      find-replace.test.ts     # Search, replace, replace-all
    integration/
      file-ops.test.ts         # Open/save round-trip via cortex-files
      tabs.test.ts             # Tab lifecycle: open, switch, close, modified
      lifecycle.test.ts        # App launch, multi-instance
```

No Rust backend crate needed. All logic is client-side TypeScript.

## 8. Data Model

### 8.1 Tab

```typescript
interface EditorTab {
  id: string;                       // UUID
  fileHandle: FileHandle | null;    // null for unsaved new files
  filename: string;                 // Display name (e.g., "index.ts")
  content: string;                  // Current editor content
  originalContent: string;          // Content as last loaded/saved (for dirty detection)
  language: SupportedLanguage | "plaintext";
  cursor: CursorPosition;
  scrollPosition: { x: number; y: number };
  undoStack: UndoEntry[];
  redoStack: UndoEntry[];
  zoomLevel: number;                // Font size in px
}
```

### 8.2 Cursor and Selection

```typescript
interface CursorPosition {
  line: number;      // 0-indexed
  column: number;    // 0-indexed
}

interface Selection {
  start: CursorPosition;
  end: CursorPosition;   // end is exclusive
}
```

### 8.3 Find/Replace State

```typescript
interface FindReplaceState {
  query: string;
  replaceText: string;
  caseSensitive: boolean;
  wholeWord: boolean;
  useRegex: boolean;
  matches: Selection[];
  currentMatchIndex: number;
}
```

### 8.4 App State

```typescript
interface TextEditorState {
  tabs: EditorTab[];
  activeTabId: string | null;
  findReplace: FindReplaceState | null;
  findReplaceOpen: boolean;
}

type SupportedLanguage = "javascript" | "typescript" | "python" | "rust" | "html" | "css" | "json" | "markdown";
```

### 8.5 Manifest

```typescript
{
  id: "com.cortexos.text-editor",
  name: "Text Editor",
  version: "1.0.0",
  description: "Plain text editor with syntax highlighting",
  firstParty: true,
  bundled: true,
  entry: { frontend: "src/main.ts" },
  window: {
    defaultWidth: 800,
    defaultHeight: 600,
    minWidth: 400,
    minHeight: 300,
    resizable: true,
    singleInstance: false
  },
  permissions: {
    required: ["runtime.state", "runtime.lifecycle", "files.read", "files.write"],
    optional: ["ai.context", "ai.invoke"]
  },
  ai: {
    surfaces: { assistantPanel: true, contextMenu: false, inlineSuggestions: false },
    contextProviders: ["text-editor-context"],
    actions: [
      {
        id: "explain-selection",
        label: "Explain selected code",
        description: "Provides an explanation of the currently selected text",
        confirmationRequired: false,
        destructive: false
      }
    ]
  },
  fileAssociations: [".txt", ".js", ".ts", ".py", ".rs", ".html", ".css", ".json", ".md"],
  accessibility: { highContrastSupport: true, screenReaderSupport: true, keyboardNavigation: true },
  category: "productivity"
}
```

## 9. Public Interfaces

### 9.1 AI Context

```typescript
interface TextEditorAIContext {
  filename: string;
  language: string;
  selectedText: string | null;     // Truncated to 2000 characters
  cursorLine: number;
  totalLines: number;
}
```

### 9.2 Commands Exposed

None. Text Editor does not expose commands to other apps.

## 10. Internal Interfaces

### 10.1 Editor Engine

```typescript
interface EditorEngine {
  insertText(state: EditorTab, text: string): EditorTab;
  deleteSelection(state: EditorTab): EditorTab;
  undo(state: EditorTab): EditorTab;
  redo(state: EditorTab): EditorTab;
  getSelectionText(state: EditorTab): string;
  getLine(state: EditorTab, lineNumber: number): string;
  getWordCount(content: string): number;
  getLineCount(content: string): number;
  getCharCount(content: string): number;
}
```

### 10.2 Tokenizer

```typescript
interface Tokenizer {
  tokenize(line: string, state: TokenizerState): TokenizeResult;
}

interface TokenizeResult {
  tokens: Token[];
  nextState: TokenizerState;
}

interface Token {
  type: TokenType;
  text: string;
  start: number;
  end: number;
}

type TokenType = "keyword" | "string" | "comment" | "number" | "type" | "function" | "operator" | "punctuation" | "tag" | "attribute" | "property" | "plain";
```

### 10.3 File Operations

```typescript
interface FileOps {
  openFile(handle: FileHandle): Promise<{ content: string; language: SupportedLanguage | "plaintext" }>;
  saveFile(handle: FileHandle, content: string): Promise<void>;
  saveFileAs(path: string, content: string): Promise<FileHandle>;
}
```

## 11. State Management

- **Ephemeral**: Find bar open/close, find options (case-sensitive, whole-word, regex), zoom level, scroll position, cursor position.
- **Session**: Open tabs (file handles, filenames, languages, cursor positions, scroll positions, zoom levels). Persisted via cortex-runtime session state. Content is re-read from cortex-files on restore.
- **Persistent**: None. File content is the source of truth on cortex-files.
- State key per instance: `com.cortexos.text-editor.session.{instanceId}`.

## 12. Failure Modes and Error Handling

| Failure | Behavior |
|---------|----------|
| File not found on open | Display error toast: "File not found: `<filename>`". Tab is not created. |
| File read error (cortex-files) | Display error toast: "Could not read file: `<filename>`". Tab is not created. |
| File write error (save) | Display error toast: "Could not save file: `<filename>`". Modified indicator remains. Retry is available. |
| File changed externally | On tab focus, compare modification timestamp. If changed, show dialog: "File has been modified externally. Reload?" with Reload / Keep Mine / Cancel. |
| File too large (over 5 MB) | Show warning: "Large file may be slow to edit. Continue?" User can proceed or cancel. |
| Syntax highlighting failure | Fall back to plain text rendering. Log as warning. |
| Undo stack exhausted | Undo/redo buttons gray out. No error displayed. |
| Regex error in find | Display inline error: "Invalid regular expression". Find continues to show matches from the last valid pattern. |
| cortex-files unavailable | Show error toast: "File system unavailable. You can still edit but cannot save." Editor remains functional for editing. |
| Too many tabs (over 20) | Show toast: "Maximum 20 tabs open." |

All errors are non-blocking. The editor remains usable after any error.

## 13. Security and Permissions

- `files.read` and `files.write` are required for open/save operations.
- If permissions are not granted, the editor can still create new untitled documents but cannot open or save files.
- Content is never sent to external services unless the user explicitly invokes an AI action.
- No `eval()` or dynamic code execution on file content.
- File paths are sanitized through cortex-files; the editor does not construct raw filesystem paths.

## 14. Performance Requirements

- File open and render: under 500 ms for files up to 1 MB.
- Typing latency: keystroke to character display under 16 ms (one frame) for files up to 500 KB.
- Syntax highlighting: incremental re-tokenization of changed lines under 5 ms.
- Tab switch: under 100 ms (content is kept in memory per tab).
- Find in file: under 200 ms for files up to 1 MB.
- Scroll: 60 fps smooth scrolling for files up to 500 KB (virtualized line rendering for larger files).
- Memory: approximately 2x file size for content + undo stack. Maximum tab content: 5 MB per tab.
- Startup first meaningful paint: under 400 ms.
- Bundle size: under 200 KB gzipped.

## 15. Accessibility Requirements

- Editor area has `role="textbox"` with `aria-multiline="true"` and `aria-label="Text editor"`.
- Line numbers have `aria-hidden="true"` (decorative, not announced).
- Tab bar tabs have `role="tab"` with `aria-selected` state.
- Find/replace inputs have `aria-label` for search field, replace field, and each option toggle.
- Status bar content has `role="status"` with `aria-live="polite"`.
- Keyboard navigation reaches all controls: tabs, find bar, editor, status bar.
- Focus is visible on all interactive elements.
- Text sizing: monospace font scales with zoom. Layout remains functional at max zoom (32px).

## 16. Observability and Logging

Logged events:
- `text-editor.launched` (info) -- App opened.
- `text-editor.file.opened` (info) -- File opened. Payload: `{ language: string, sizeBucket: "<100KB" | "100KB-1MB" | ">1MB" }`. No filename or content.
- `text-editor.file.saved` (info) -- File saved. Payload: `{ language: string }`. No filename or content.
- `text-editor.find.used` (info) -- Find bar opened. Payload: `{ options: string[] }` (active options like caseSensitive, regex).
- `text-editor.tab.opened` (info) -- New tab created.
- `text-editor.ai.explain_invoked` (info) -- AI explain action triggered.
- `text-editor.error` (warn) -- File operation error type. Payload: `{ operation: string, errorType: string }`. No file content.

No PII is logged. File names, paths, and content are never included in log payloads.

## 17. Testing Requirements

### 17.1 Unit Tests

- Editor engine: insert text, delete selection, cursor movement, undo/redo stack, boundary cases (empty document, single character, very long line).
- Tokenizer: each language's keyword, string, comment, number, and operator recognition. Edge cases: unterminated strings, nested comments, mixed content.
- Text utilities: word count (handles multiple spaces, empty strings, Unicode), line count, character count.
- Find/replace: literal search, case-sensitive, whole-word, regex, replace, replace-all, no matches, special characters in query.

### 17.2 Integration Tests

- File open/save round-trip: create file via cortex-files, open in editor, modify, save, verify content.
- Tab lifecycle: open multiple files, switch tabs, verify content preserved per tab, close tab with unsaved changes (verify prompt).
- Multi-step edit: open file, make edits, undo, redo, save, verify final content.
- External file change: modify file externally, switch to tab, verify reload prompt.
- Find/replace flow: open find, type query, navigate matches, replace one, replace all, verify content.

### 17.3 Accessibility Tests

- AX tree validation for editor, tabs, find bar, status bar.
- Keyboard-only full workflow: open file, edit, find, replace, save, close tab.
- Screen reader announcement of status bar changes.

## 18. Acceptance Criteria

- [ ] Syntax highlighting renders correctly for all 8 supported languages.
- [ ] Line numbers display and update correctly on edit and scroll.
- [ ] Find (`Ctrl+F`) highlights all matches, navigates with Enter/Shift+Enter.
- [ ] Replace (`Ctrl+H`) replaces single and all occurrences correctly.
- [ ] Regex find works for valid patterns; shows error for invalid patterns.
- [ ] Multi-tab: up to 20 files open simultaneously, switching preserves content and cursor.
- [ ] Modified indicator appears on edit and clears on save.
- [ ] Close modified tab triggers save prompt with Save / Don't Save / Cancel.
- [ ] Open/save round-trip preserves file content exactly via cortex-files.
- [ ] Word count, line count, and character count display correctly in status bar.
- [ ] Zoom (`Ctrl+`/`Ctrl-`/`Ctrl+0`) adjusts font size within 10-32px range.
- [ ] Go to line (`Ctrl+G`) scrolls to correct line.
- [ ] Undo/redo work correctly for up to 500 steps.
- [ ] Large files (over 100 KB) disable syntax highlighting with notification.
- [ ] App launches in under 400 ms.
- [ ] All three themes render correctly.
- [ ] Screen reader announces status bar content.
- [ ] File associations open Text Editor from File Manager.
- [ ] Unit test coverage >= 80%.

## 19. Build Order and Dependencies
**Layer 11**. Depends on: 09 (app runtime), 16 (theme tokens), 17 (first-party app parent)

### Prerequisites

- Spec 17 parent (first-party app framework).
- `@cortexos/ui-components` (shared UI library).
- `@cortexos/runtime-client` (for state persistence and lifecycle).
- `@cortexos/files-client` (for file open/save operations).
- `@cortexos/ai-client` (for AI surface).
- `@cortexos/theme` (design token consumer).

### Build Position

Text Editor is the **fourth** first-party app to build (after Clock Utilities, Calculator, and Terminal Lite). It validates cortex-files read/write integration and multi-tab UI patterns.

No Rust crate needed. Pure frontend app.

## 20. Non-Goals and Anti-Patterns

### Non-Goals

- IDE features: debugger, terminal, project management, build tools.
- Language server integration or code completion.
- Git or version control UI.
- Plugin/extension system.
- Encoding conversion beyond UTF-8.
- Collaborative editing.

### Anti-Patterns

- Using `eval()` or `new Function()` for syntax highlighting or text processing.
- Loading the entire file content into the DOM at once for large files (use virtualized rendering).
- Storing file content in session state (content lives on cortex-files).
- Blocking the UI thread on file reads or writes (all cortex-files calls must be async).
- Implementing a custom text layout engine instead of using browser contenteditable or a proven textarea approach.

## 21. Implementation Instructions for Claude Code / Codex

### Subsystem Ownership

- Text Editor owns: editor engine (insert, delete, cursor, selection, undo/redo), syntax tokenizers, find/replace logic, tab management, text statistics.
- Text Editor does not own: file system access (delegates to cortex-files), window management, theme system.

### Recommended Implementation Order

1. Create `manifest.json` and validate against FirstPartyAppManifest schema.
2. Implement `services/editor-engine.ts` -- core text editing with cursor, selection, insert, delete, undo/redo. Write comprehensive unit tests first.
3. Implement `services/text-utils.ts` -- word/line/char counting. Write unit tests.
4. Implement `services/tokenizers/` -- one tokenizer per language. Start with JSON (simplest), then Markdown, HTML, CSS, JavaScript/TypeScript, Python, Rust. Write unit tests per tokenizer.
5. Implement `components/Editor.tsx` with line numbers and basic editing.
6. Implement `components/TabBar.tsx` and `hooks/useTabs.ts` for multi-tab support.
7. Implement `services/file-ops.ts` wrapping cortex-files for open/save/save-as.
8. Wire up `App.tsx` connecting editor, tabs, and file operations.
9. Implement `components/FindReplaceBar.tsx` and `hooks/useFindReplace.ts`.
10. Implement `components/StatusBar.tsx` with counts and cursor position.
11. Implement `components/GoToLinePrompt.tsx`.
12. Implement `hooks/useZoom.ts` and wire zoom shortcuts.
13. Integrate `@cortexos/ai-client` for AI surface.
14. Accessibility audit and fixes.
15. Theme verification (light, dark, high-contrast).
16. Performance optimization: virtualized line rendering for large files.

### What Can Be Stubbed Initially

- AI "Explain selection" action can return a placeholder explanation initially.
- Virtualized rendering can use simple rendering first, then optimize for large files.
- Tokenizers for less common languages (Rust) can be deferred after core languages (JS, HTML, CSS, JSON).

### What Must Be Real in V1

- Full editor engine with undo/redo (500 steps).
- Syntax highlighting for all 8 languages.
- Find/replace with case-sensitive, whole-word, and regex options.
- Multi-tab editing (up to 20 tabs).
- Open/save/save-as via cortex-files.
- Line numbers, word/line/char count, go-to-line.
- Zoom controls.
- File associations.
- Unsaved changes prompt on tab/window close.
- Theme support.
- Accessibility (keyboard navigation, screen reader).

### What Cannot Be Inferred

- Default font size (14px monospace per design tokens).
- Tab size (2 spaces default, configurable).
- Syntax highlighting color mappings (consume from theme design tokens).
- Default window size (800x600 per manifest).
- Maximum undo steps (500 per tab).
- Large file threshold (100 KB for syntax highlighting, 5 MB for open warning).

### Stop Conditions

1. All unit tests pass with >= 80% coverage.
2. Editor engine tests cover insert, delete, undo, redo, and edge cases.
3. All 8 tokenizers produce correct output for their respective languages.
4. Integration tests for file open/save round-trip pass.
5. Find/replace works with all options (literal, case-sensitive, whole-word, regex).
6. Multi-tab lifecycle tests pass (open, switch, close, modified prompt).
7. No `eval()` usage confirmed by code review and linter rule.
8. All three themes render correctly.
9. File associations work from File Manager.
10. Performance: typing latency under 16 ms for files up to 500 KB.

### Testing Gates

- Editor engine unit tests must pass before any UI work begins.
- Tokenizer unit tests must pass before syntax highlighting is wired to the UI.
- File operation integration tests must pass before merge.
- Performance benchmark: open a 500 KB file and verify typing latency.
