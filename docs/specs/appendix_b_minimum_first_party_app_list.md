# Appendix B: Minimum First-Party App List

## Purpose

This appendix defines the complete list of first-party applications that must ship with CortexOS v1. Each app entry specifies its spec reference, directory location, description, priority, and dependencies on other subsystems.

## Priority Definitions

| Priority | Meaning |
|----------|---------|
| P0 | Must have for launch. The release is blocked if this app is not functional. |
| P1 | Should have for launch. The release is not blocked, but the app should be included if at all possible. |

## App List

---

### B.1. Settings

| Property | Value |
|----------|-------|
| **App Name** | Settings |
| **Spec Reference** | Spec 05 (Settings Service and Settings App) |
| **App ID** | `settings` |
| **App Directory** | `apps/settings-app/` |
| **Priority** | P0 (must have for launch) |
| **Single Instance** | Yes (only one Settings window at a time) |

**Description**: The central configuration application for CortexOS. Provides the user interface for all system settings including: user profile, AI provider configuration (preferred LLM, model selection, fallback chain, privacy mode, budget policy), permissions management, theme selection, accessibility options, notification preferences, file manager preferences, and system information (About). The Settings app is the primary way users interact with the settings service (cortex-settings) and is critical for the AI provider selection flow.

**Mandatory Sections**:
- AI Settings: preferred provider, preferred model, fallback configuration, privacy mode, file/clipboard access, budget policy, model disclosure
- User Profile: display name, avatar
- Appearance: theme selection, font size
- Permissions: app permissions overview, AI permissions overview
- Accessibility: keyboard shortcuts, screen reader options
- Notifications: notification preferences per app
- About: version info, build info, license info

**Subsystem Dependencies**:
- `cortex-settings` (reads and writes all settings)
- `cortex-auth` (reads user profile)
- `cortex-policy` (reads and manages permissions)
- `cortex-ai` (reads provider registry for dropdown population)
- `cortex-observability` (reads budget usage data)
- `cortex-notify` (reads notification preferences)
- `cortex-config` (reads system configuration)
- Desktop Shell (launched from dock or command palette)

---

### B.2. Calculator

| Property | Value |
|----------|-------|
| **App Name** | Calculator |
| **Spec Reference** | Spec 17a |
| **App ID** | `calculator` |
| **App Directory** | `apps/calculator-app/` |
| **Priority** | P1 (should have for launch) |
| **Single Instance** | Yes (only one Calculator window at a time) |

**Description**: A standard calculator application supporting basic arithmetic operations (addition, subtraction, multiplication, division), percentage, sign toggle, and clear functions. Provides a familiar calculator interface with a display area showing the current expression and result. Supports keyboard input for all operations. History of calculations is preserved within the session but not persisted across sessions (v1). No scientific mode in v1.

**Mandatory Features**:
- Standard four-function calculator layout
- Decimal point support
- Percentage calculation
- Sign toggle (+/-)
- Clear (C) and Clear Entry (CE)
- Keyboard shortcuts for all operations
- Display shows current expression and result
- Error handling for division by zero and overflow

**Subsystem Dependencies**:
- `cortex-runtime` (app lifecycle, window management)
- Desktop Shell (launched from dock or command palette)
- Theme system (respects design tokens)
- Accessibility system (keyboard navigation, screen reader support)

---

### B.3. Text Editor

| Property | Value |
|----------|-------|
| **App Name** | Text Editor |
| **Spec Reference** | Spec 17b |
| **App ID** | `text-editor` |
| **App Directory** | `apps/text-editor-app/` |
| **Priority** | P1 (should have for launch) |
| **Single Instance** | No (multiple instances allowed, each editing a different file) |

**Description**: A plain text editor for viewing and editing text files. Supports opening files from the virtual filesystem, editing content, and saving changes back. Provides basic editing operations: cursor movement, selection, copy/cut/paste, undo/redo, find and replace. Supports line numbers display, word wrap toggle, and basic syntax highlighting for common formats (JSON, Markdown, plain text). Does not support rich text formatting in v1.

**Mandatory Features**:
- Open file from virtual filesystem
- Save file to virtual filesystem
- Save As (save to a new location)
- Cursor movement and text selection
- Copy, cut, paste (via keyboard and context menu)
- Undo and redo
- Find and replace
- Line numbers (toggleable)
- Word wrap (toggleable)
- Syntax highlighting for JSON, Markdown
- Tab character support
- New file creation
- Unsaved changes indicator in title bar
- Modified file warning on close

**Subsystem Dependencies**:
- `cortex-files` (open, save, save-as file operations)
- `cortex-runtime` (app lifecycle, multi-instance management)
- Desktop Shell (file association: .txt, .md, .json, .log files)
- Theme system (syntax highlighting colors from design tokens)
- Accessibility system (keyboard navigation)
- `cortex-ai` (optional: AI-assisted text completion if AI is configured)

---

### B.4. Notes

| Property | Value |
|----------|-------|
| **App Name** | Notes |
| **Spec Reference** | Spec 17c |
| **App ID** | `notes` |
| **App Directory** | `apps/notes-app/` |
| **Priority** | P1 (should have for launch) |
| **Single Instance** | Yes (only one Notes window, but with a sidebar listing all notes) |

**Description**: A note-taking application for creating, editing, and organizing plain text notes. Notes are stored in the virtual filesystem and managed by the notes app. Each note has a title (first line of content) and a body. The interface shows a sidebar with a list of notes sorted by modification date and a main editing area. Notes support Markdown preview mode. Notes are persisted automatically on edit (auto-save with debounce of 500ms). No categories, tags, or folders in v1.

**Mandatory Features**:
- Create new note
- Delete note (with confirmation)
- Edit note content (plain text with Markdown support)
- Markdown preview toggle
- Sidebar list of all notes sorted by last modified
- Auto-save on edit (500ms debounce)
- Search notes by content (basic substring search)
- Note title derived from first line of content
- Empty note list shows welcome message

**Subsystem Dependencies**:
- `cortex-files` (note storage in virtual filesystem)
- `cortex-search` (notes are indexed for global search)
- `cortex-runtime` (app lifecycle)
- Desktop Shell (launched from dock or command palette)
- Theme system (respects design tokens)
- Accessibility system (keyboard navigation, screen reader)
- `cortex-ai` (optional: AI-assisted note summarization if AI is configured)

---

### B.5. File Manager

| Property | Value |
|----------|-------|
| **App Name** | File Manager |
| **Spec Reference** | Spec 17d |
| **App ID** | `file-manager` |
| **App Directory** | `apps/file-manager-app/` |
| **Priority** | P0 (must have for launch) |
| **Single Instance** | No (multiple instances allowed) |

**Description**: The primary file management application for navigating, organizing, and managing files and directories in the CortexOS virtual filesystem. Provides a dual-pane or single-pane view with a directory tree sidebar, file/folder listing, breadcrumb navigation, and context menus for file operations. Supports all standard file operations: create, rename, move, copy, delete, and view properties. Includes a trash/recycle mechanism with restore capability. File Manager is critical infrastructure; users must be able to browse and manage their files.

**Mandatory Features**:
- Directory tree sidebar (collapsible)
- File and folder listing (list view and grid view toggle)
- Breadcrumb navigation
- Navigate into directories
- Navigate up to parent directory
- Create new folder
- Create new file (empty, with extension)
- Rename file or folder
- Delete file or folder (moves to trash)
- Copy file or folder
- Move file or folder (cut and paste)
- Trash view (show deleted items, restore, permanent delete)
- File properties view (name, size, type, created date, modified date)
- Sort by name, date, size, type
- Select single or multiple items
- Context menu (right-click)
- Open file with associated app
- Drag and drop for move/copy operations

**Subsystem Dependencies**:
- `cortex-files` (all file operations: create, read, update, delete, move, copy, trash)
- `cortex-runtime` (app lifecycle, multi-instance, file association launching)
- `cortex-search` (files are indexed for global search)
- Desktop Shell (file type associations, default app for folders)
- Theme system (file type icons from design tokens)
- Accessibility system (keyboard navigation for all operations)
- `cortex-policy` (permission checks for file operations)

---

### B.6. Media Viewer

| Property | Value |
|----------|-------|
| **App Name** | Media Viewer |
| **Spec Reference** | Spec 17e |
| **App ID** | `media-viewer` |
| **App Directory** | `apps/media-viewer-app/` |
| **Priority** | P1 (should have for launch) |
| **Single Instance** | No (multiple instances allowed, each viewing a different file) |

**Description**: A media viewing application for displaying image files. Supports common image formats: PNG, JPEG, GIF (including animated), SVG, WebP, and BMP. Provides zoom (in, out, fit-to-window, actual size), pan, and rotate operations. No audio or video playback in v1. When opened without a file argument, shows an empty state prompting the user to open a file.

**Mandatory Features**:
- Open image file from virtual filesystem
- Display image in viewer area
- Zoom in, zoom out, fit to window, actual size (100%)
- Pan (scroll or drag when zoomed in)
- Rotate 90 degrees clockwise/counterclockwise
- Navigate between images in the same folder (previous/next)
- Image information overlay (dimensions, file size, format)
- Support formats: PNG, JPEG, GIF, SVG, WebP, BMP
- Empty state when no file is open
- Error state for unsupported or corrupted files

**Supported File Extensions**:
- `.png`, `.jpg`, `.jpeg`, `.gif`, `.svg`, `.webp`, `.bmp`

**Subsystem Dependencies**:
- `cortex-files` (open file, read file content)
- `cortex-runtime` (app lifecycle, multi-instance)
- Desktop Shell (file association for image file types)
- Theme system (viewer chrome respects design tokens)
- Accessibility system (keyboard shortcuts for zoom, pan, rotate)

---

### B.7. Terminal-lite

| Property | Value |
|----------|-------|
| **App Name** | Terminal-lite |
| **Spec Reference** | Spec 17f |
| **App ID** | `terminal-lite` |
| **App Directory** | `apps/terminal-lite-app/` |
| **Priority** | P1 (should have for launch) |
| **Single Instance** | No (multiple terminal instances allowed) |

**Description**: A lightweight terminal emulator providing a command-line interface to the CortexOS virtual filesystem and system commands. Supports a defined set of built-in commands for file system navigation, file manipulation, and system information. Not a full Unix shell; implements a curated command set. Commands execute against the CortexOS virtual filesystem, not the host filesystem. Command history is preserved within the session (up to 100 entries) but not persisted across sessions in v1.

**Built-in Commands**:

| Command | Description |
|---------|-------------|
| `ls [path]` | List directory contents |
| `cd <path>` | Change current directory |
| `pwd` | Print current working directory |
| `cat <file>` | Display file contents |
| `mkdir <dir>` | Create directory |
| `touch <file>` | Create empty file |
| `rm <file>` | Remove file (permanent, no trash) |
| `rmdir <dir>` | Remove empty directory |
| `mv <src> <dest>` | Move/rename file or directory |
| `cp <src> <dest>` | Copy file |
| `echo <text>` | Print text to terminal |
| `clear` | Clear terminal output |
| `help` | Show available commands |
| `version` | Show CortexOS version |
| `whoami` | Show current user |
| `date` | Show current date and time |
| `history` | Show command history |

**Mandatory Features**:
- Command input line with prompt showing current directory
- Command output area (scrollable)
- Command history navigation (up/down arrows)
- Tab completion for file and directory names
- Clear output area
- Error messages for invalid commands and file not found
- Welcome message on open
- ANSI color support (basic: red for errors, green for success, default for output)

**Subsystem Dependencies**:
- `cortex-files` (all file system operations)
- `cortex-auth` (user identity for whoami)
- `cortex-runtime` (app lifecycle, multi-instance)
- Desktop Shell (launched from dock or command palette)
- Theme system (terminal colors from design tokens)
- Accessibility system (keyboard navigation is inherent to terminal)

---

### B.8. Clock Utilities

| Property | Value |
|----------|-------|
| **App Name** | Clock Utilities |
| **Spec Reference** | Spec 17g |
| **App ID** | `clock-utils` |
| **App Directory** | `apps/clock-utils-app/` |
| **Priority** | P1 (should have for launch) |
| **Single Instance** | Yes (only one Clock window at a time) |

**Description**: A clock utility application providing three functions: a digital clock display, a stopwatch, and a countdown timer. Each function is accessible via a tab interface. The clock tab shows the current time and date. The stopwatch tab provides start, stop, lap, and reset controls. The timer tab allows setting a duration and provides start, pause, and reset controls, with a notification when the timer completes.

**Tabs and Features**:

**Clock Tab**:
- Digital clock display (hours:minutes:seconds, 12-hour or 24-hour format)
- Current date display
- Time zone display
- Format toggle (12h / 24h)

**Stopwatch Tab**:
- Elapsed time display (minutes:seconds.centiseconds)
- Start button
- Stop button
- Lap button (records lap time, displays in list)
- Reset button
- Lap list (scrollable, shows lap number, lap time, total time)

**Timer Tab**:
- Duration input (hours, minutes, seconds via number inputs)
- Start button
- Pause button
- Reset button
- Countdown display
- Notification when timer reaches zero (triggers notification via cortex-notify)
- Audible alert option (single beep sound)

**Subsystem Dependencies**:
- `cortex-runtime` (app lifecycle)
- `cortex-notify` (timer completion notification)
- Desktop Shell (launched from dock or command palette)
- Theme system (respects design tokens)
- Accessibility system (keyboard navigation for all controls)

---

## Summary Table

| # | App Name | App ID | Directory | Priority | Spec | Single Instance |
|---|----------|--------|-----------|----------|------|----------------|
| 1 | Settings | `settings` | `apps/settings-app/` | P0 | 05 | Yes |
| 2 | Calculator | `calculator` | `apps/calculator-app/` | P1 | 17a | Yes |
| 3 | Text Editor | `text-editor` | `apps/text-editor-app/` | P1 | 17b | No |
| 4 | Notes | `notes` | `apps/notes-app/` | P1 | 17c | Yes |
| 5 | File Manager | `file-manager` | `apps/file-manager-app/` | P0 | 17d | No |
| 6 | Media Viewer | `media-viewer` | `apps/media-viewer-app/` | P1 | 17e | No |
| 7 | Terminal-lite | `terminal-lite` | `apps/terminal-lite-app/` | P1 | 17f | No |
| 8 | Clock Utilities | `clock-utils` | `apps/clock-utils-app/` | P1 | 17g | Yes |

## Subsystem Dependency Matrix

| Subsystem | Settings | Calculator | Text Editor | Notes | File Manager | Media Viewer | Terminal-lite | Clock Utils |
|-----------|----------|-----------|-------------|-------|-------------|-------------|---------------|-------------|
| cortex-settings | W | - | - | - | - | - | - | - |
| cortex-auth | R | - | - | - | - | - | R | - |
| cortex-policy | RW | - | - | - | R | - | - | - |
| cortex-ai | R | - | optional | optional | - | - | - | - |
| cortex-files | - | - | RW | RW | RW | R | RW | - |
| cortex-search | - | - | - | R | R | - | - | - |
| cortex-runtime | R | R | R | R | R | R | R | R |
| cortex-notify | - | - | - | - | - | - | - | W |
| cortex-observability | R | - | - | - | - | - | - | - |
| cortex-config | R | - | - | - | - | - | - | - |
| Desktop Shell | R | R | R | R | R | R | R | R |
| Theme System | R | R | R | R | R | R | R | R |
| Accessibility | R | R | R | R | R | R | R | R |

Legend: R = reads from, W = writes to, RW = reads and writes, optional = feature available if AI configured, - = no dependency

## Build Order

The recommended build order for first-party apps, based on subsystem dependency complexity:

1. **Calculator** - simplest app, fewest dependencies, useful for validating app runtime and theme system
2. **Clock Utilities** - simple app, validates notification integration
3. **Text Editor** - validates file system integration
4. **Media Viewer** - validates file system and file associations
5. **Notes** - validates file system, search indexing, and auto-save
6. **Terminal-lite** - validates file system commands and auth integration
7. **File Manager** - most complex file system consumer, validates policy integration
8. **Settings** - most complex app overall, depends on all subsystems being functional

## Manifest Requirements

Each first-party app must have a valid app manifest (per spec 09 / spec 21) in its directory at `manifest.json`. The manifest must declare:

- `app_id`: as listed in the table above
- `version`: "1.0.0"
- `name`: human-readable app name
- `description`: brief description
- `single_instance`: as listed in the table above
- `permissions`: array of required permissions (varies per app)
- `file_associations`: array of file extensions the app handles (varies per app)
- `min_cortexos_version`: "1.0.0"

First-party apps are bundled with the OS and are not subject to third-party app review processes. They are subject to all other quality gates and acceptance criteria defined in spec 23 and appendix C.
