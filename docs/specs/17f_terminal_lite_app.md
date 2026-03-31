# 17f. Terminal Lite App

## 1. Purpose

The Terminal Lite app provides a safe, sandboxed command-line interface within CortexOS. It offers a familiar shell-like experience for file system operations using a curated set of commands, all routed through cortex-files. It does not expose arbitrary shell execution, ensuring the OS security boundary is maintained.

## 2. Scope

- Terminal prompt with command input and output display.
- Scrollback buffer of 1000 lines.
- Command history navigation with up/down arrow keys (max 500 entries).
- Tab auto-complete for file paths and command names.
- Supported commands: `ls`, `cat`, `pwd`, `echo`, `cd`, `mkdir`, `touch`, `rm`, `cp`, `mv`, `clear`, `help`, `history`, `whoami`, `date`, `exit`.
- All file operations routed through cortex-files API.
- Current working directory tracking (virtual, within cortex-files namespace).
- App location: `apps/terminal-lite-app`.

## 3. Out of Scope

- Arbitrary shell execution (no `sh`, `bash`, `zsh`, or subprocess spawning).
- Pipes (`|`), redirections (`>`, `>>`, `<`), or command chaining (`&&`, `||`, `;`).
- Environment variables (`$VAR`, `export`).
- Glob/wildcard expansion (`*`, `?`, `[...]`).
- Background processes (`&`), job control (`fg`, `bg`, `jobs`).
- Alias definitions or shell scripting.
- `grep`, `find`, `sed`, `awk`, `curl`, `wget`, or other Unix utilities.
- `chmod`, `chown`, or any permission management commands.
- `sudo` or privilege escalation.
- Terminal multiplexing or split panes.
- SSH or remote shell access.
- ANSI escape sequence rendering beyond basic formatting (bold, color for errors/prompts).

## 4. Objectives

1. Provide a safe, discoverable command-line interface for basic file operations.
2. Validate command bus integration with cortex-files for read and write operations.
3. Demonstrate a text-heavy UI pattern with scrollback, input history, and auto-complete.
4. Serve as a power-user tool complementing the graphical File Manager.

## 5. User-Visible Behavior

### 5.1 Terminal Display

- The terminal fills the app window with a dark-background text area (light text on dark background regardless of OS theme, similar to traditional terminals).
- A prompt line at the bottom shows: `user@hostname:~/path$ ` where `~/path` is the current working directory relative to the user home.
- User types commands after the prompt. Output appears above.
- Scrollbar on the right for navigating scrollback. Mouse wheel scrolls output.
- `Ctrl+L` or `clear` command clears the visible output.

### 5.2 Command Input

- Single-line text input at the bottom. `Enter` submits the command.
- `Up`/`Down` arrows navigate command history. History is per-session, persisted across hot-reload.
- `Tab` triggers auto-complete:
  - If the cursor is at a command position (first word), completes against the supported command list.
  - If the cursor is at an argument position, completes against file/directory names in the current working directory or the partial path typed so far.
  - If multiple matches exist, `Tab` once shows no change; `Tab` twice lists all matches below the input line.
- `Ctrl+C` cancels the current input and returns a new prompt.
- `Ctrl+U` clears the current input line.
- `Backspace` and `Delete` work as expected. No multiline input.

### 5.3 Command Behavior

| Command | Syntax | Behavior |
|---------|--------|----------|
| `ls` | `ls [path]` | Lists directory contents. Default: current directory. Shows file names, type indicator (`/` for directories). |
| `cat` | `cat <path>` | Outputs file content as text. Binary files show a warning: "Binary file, cannot display." |
| `pwd` | `pwd` | Prints the current working directory path. |
| `echo` | `echo <text>` | Prints the provided text to output. No variable expansion. |
| `cd` | `cd <path>` | Changes current working directory. Supports relative and absolute paths. `cd ..` navigates up. `cd` with no argument goes to user home. |
| `mkdir` | `mkdir <path>` | Creates a directory at the specified path. Fails if parent does not exist. |
| `touch` | `touch <path>` | Creates an empty file at the specified path. If file exists, no-op (does not update timestamp). |
| `rm` | `rm <path>` | Removes a file. `rm -r <path>` removes a directory recursively. Prompts for confirmation on directories. |
| `cp` | `cp <src> <dest>` | Copies a file from src to dest. Does not support directory copy in V1. |
| `mv` | `mv <src> <dest>` | Moves/renames a file from src to dest. Does not support directory move in V1. |
| `clear` | `clear` | Clears the terminal output. |
| `help` | `help [command]` | Lists all available commands, or shows help for a specific command. |
| `history` | `history` | Displays command history with line numbers. |
| `whoami` | `whoami` | Displays the current user name from cortex-runtime identity. |
| `date` | `date` | Displays the current date and time in ISO 8601 format. |
| `exit` | `exit` | Closes the terminal app. |

### 5.4 Error Output

- Errors are displayed in red text (or high-contrast error color) on the line after the command.
- Format: `error: <message>` (lowercase, similar to common CLI conventions).
- Common errors: "No such file or directory", "Not a directory", "Permission denied", "Command not found: <cmd>", "Usage: <command> <syntax>".

## 6. System Behavior

### 6.1 Command Parsing

- Commands are parsed as space-separated tokens. The first token is the command name.
- Quoted strings (single or double quotes) are treated as a single token: `echo "hello world"` outputs `hello world`.
- No variable expansion, no command substitution, no glob expansion.
- Empty input is ignored (produces a new prompt).
- Leading and trailing whitespace is trimmed.

### 6.2 File Operations

- All file operations are routed through `cortex-files` client API:
  - `ls` -> `cortex-files.files.list(path)`
  - `cat` -> `cortex-files.files.read(handle)`
  - `cd` -> updates local CWD state, validates via `cortex-files.files.stat(path)`
  - `mkdir` -> `cortex-files.files.createDirectory(path)`
  - `touch` -> `cortex-files.files.createFile(path, "")`
  - `rm` -> `cortex-files.files.delete(handle)`
  - `cp` -> `cortex-files.files.read(handle)` then `cortex-files.files.createFile(destPath, content)`
  - `mv` -> copy then delete (no atomic rename in V1)
- The current working directory is tracked as a string path in app state, resolved relative to the user home directory.
- Paths starting with `/` are treated as absolute within the cortex-files namespace. Paths without `/` are relative to CWD.
- `..` is resolved by removing the last path component. `.` is a no-op.

### 6.3 Auto-Complete Implementation

- On `Tab` press, the input is split at the cursor position.
- If the text before the cursor contains no spaces, auto-complete attempts command name completion against the 16 supported commands.
- If the text contains a space (cursor is in argument position), auto-complete extracts the partial path token and queries `cortex-files.files.list` of the relevant directory.
- Matches are filtered by prefix. If exactly one match, the input is completed. If multiple matches, the first Tab shows no change; a second Tab within 500 ms lists all matches.
- Auto-complete queries are debounced and cached per directory for the session.

### 6.4 App Lifecycle

- Multi-instance app. Each window is an independent terminal session.
- Each instance has its own CWD, command history, and scrollback.
- On unmount, session state is saved. On remount, scrollback is cleared but CWD and history are restored.
- The `exit` command closes the current window.

### 6.5 Scrollback Management

- Maximum 1000 lines of output retained in memory.
- When the limit is reached, the oldest lines are removed (FIFO).
- Lines are stored as an array of strings with metadata (command output vs. error vs. prompt).
- Virtualized rendering: only visible lines plus a buffer are rendered to the DOM to maintain performance.

## 7. Architecture

```
apps/terminal-lite-app/
  manifest.json
  package.json
  src/
    main.ts                    # Registers app with runtime
    App.tsx                    # Root component
    components/
      TerminalDisplay.tsx      # Scrollback output area (virtualized)
      PromptLine.tsx           # Input line with prompt prefix
      AutoCompleteOverlay.tsx  # Completion suggestions dropdown
    services/
      command-parser.ts        # Tokenization, quote handling
      command-registry.ts      # Command definitions, validation, help text
      path-resolver.ts         # Relative/absolute path resolution, .. and . handling
      commands/
        ls.ts                  # ls implementation
        cat.ts                 # cat implementation
        pwd.ts                 # pwd implementation
        echo.ts                # echo implementation
        cd.ts                  # cd implementation
        mkdir.ts               # mkdir implementation
        touch.ts               # touch implementation
        rm.ts                  # rm implementation
        cp.ts                  # cp implementation
        mv.ts                  # mv implementation
        clear.ts               # clear implementation
        help.ts                # help implementation
        history.ts             # history implementation
        whoami.ts              # whoami implementation
        date.ts                # date implementation
        exit.ts                # exit implementation
      auto-completer.ts        # Tab completion logic
    hooks/
      useTerminal.ts           # Main terminal state: scrollback, CWD, history
      useCommandHistory.ts     # Up/down history navigation
      useAutoComplete.ts       # Tab completion state
    ai/
      context.ts               # Provides CWD and last command to AI
      actions.ts               # "Explain this command" action
    types.ts
  tests/
    unit/
      command-parser.test.ts   # Tokenization, quoting, edge cases
      path-resolver.test.ts    # Path resolution, .., absolute/relative
      auto-completer.test.ts   # Command and path completion
      commands/
        ls.test.ts
        cat.test.ts
        cd.test.ts
        rm.test.ts
        cp.test.ts
        mv.test.ts
    integration/
      terminal-session.test.ts # Full command execution flows
      file-ops.test.ts         # File operations through cortex-files
```

No Rust backend crate needed. All command logic is TypeScript that calls cortex-files client APIs.

## 8. Data Model

### 8.1 Terminal State

```typescript
interface TerminalState {
  cwd: string;                    // Current working directory path
  scrollback: ScrollbackEntry[];  // Max 1000 entries
  commandHistory: string[];       // Max 500 entries
  historyIndex: number;           // Current position in history (-1 = new input)
  currentInput: string;
  cursorPosition: number;
  autoCompleting: boolean;
  autoCompleteMatches: string[];
  pendingConfirmation: PendingConfirmation | null;
}

interface ScrollbackEntry {
  id: string;
  type: "prompt" | "output" | "error" | "system";
  text: string;
  timestamp: string;              // ISO 8601
}

interface PendingConfirmation {
  command: string;                // The command awaiting confirmation
  message: string;                // e.g., "Remove directory 'foo' and all contents?"
}
```

### 8.2 Command Definition

```typescript
interface CommandDefinition {
  name: string;
  syntax: string;
  description: string;
  minArgs: number;
  maxArgs: number | null;         // null = unlimited
  validate(args: string[]): string | null;  // Returns error message or null
}
```

### 8.3 Manifest

```typescript
{
  id: "com.cortexos.terminal-lite",
  name: "Terminal",
  version: "1.0.0",
  description: "Command-line interface for file operations",
  firstParty: true,
  bundled: true,
  entry: { frontend: "src/main.ts" },
  window: {
    defaultWidth: 720,
    defaultHeight: 480,
    minWidth: 400,
    minHeight: 250,
    resizable: true,
    singleInstance: false
  },
  permissions: {
    required: ["runtime.state", "runtime.lifecycle", "files.read", "files.write"],
    optional: ["ai.context", "ai.invoke"]
  },
  ai: {
    surfaces: { assistantPanel: true, contextMenu: false, inlineSuggestions: false },
    contextProviders: ["terminal-context"],
    actions: [
      {
        id: "explain-command",
        label: "Explain this command",
        description: "Provides an explanation of the last executed command and its output",
        confirmationRequired: false,
        destructive: false
      }
    ]
  },
  accessibility: { highContrastSupport: true, screenReaderSupport: true, keyboardNavigation: true },
  category: "development"
}
```

## 9. Public Interfaces

### 9.1 AI Context

```typescript
interface TerminalAIContext {
  cwd: string;
  lastCommand: string | null;
  lastOutput: string | null;     // Truncated to 500 characters
  lastError: string | null;
}
```

### 9.2 Commands Exposed

None. Terminal Lite does not expose commands to other apps.

## 10. Internal Interfaces

### 10.1 Command Executor

```typescript
interface CommandExecutor {
  execute(commandName: string, args: string[], state: TerminalState): Promise<CommandResult>;
}

interface CommandResult {
  output: string[];
  errors: string[];
  stateChanges: Partial<TerminalState>;  // e.g., cwd change from cd
  confirmRequired?: PendingConfirmation;
}
```

### 10.2 Path Resolver

```typescript
interface PathResolver {
  resolve(path: string, cwd: string): string;          // Resolves relative to absolute
  parentDir(path: string): string;                      // Get parent directory
  join(base: string, relative: string): string;         // Join path segments
  isAbsolute(path: string): boolean;
}
```

### 10.3 Auto-Completer

```typescript
interface AutoCompleter {
  completeCommands(partial: string): string[];
  completePaths(partial: string, cwd: string): Promise<string[]>;
}
```

## 11. State Management

- **Ephemeral**: Current input text, cursor position, auto-complete dropdown visibility, scroll position.
- **Session**: CWD, command history (max 500), scrollback (max 1000 lines). Persisted via cortex-runtime session state.
- **Persistent**: None. Each terminal session starts fresh (or restores session state on hot-reload).
- State key per instance: `com.cortexos.terminal-lite.session.{instanceId}`.

## 12. Failure Modes and Error Handling

| Failure | Behavior |
|---------|----------|
| Command not found | Display: `error: command not found: <name>`. Suggest similar commands if a close match exists (Levenshtein distance <= 2). |
| Wrong argument count | Display: `error: usage: <command> <syntax>`. |
| File not found | Display: `error: no such file or directory: <path>`. |
| Not a directory | Display: `error: not a directory: <path>`. |
| Is a directory (cat on dir) | Display: `error: <path> is a directory`. |
| Permission denied | Display: `error: permission denied: <path>`. |
| Directory not empty (rm without -r) | Display: `error: directory not empty: <path>. Use rm -r to remove recursively.` |
| cortex-files unavailable | Display: `error: file system unavailable. Please try again.` All file commands fail gracefully. Non-file commands (echo, help, date, etc.) continue to work. |
| Binary file cat | Display: `warning: binary file, cannot display: <path>`. |
| Path traversal above root | Display: `error: cannot navigate above root directory`. CWD remains unchanged. |
| `rm` on directory without `-r` | Display confirmation prompt: `Remove directory '<name>' and all contents? (y/n)`. |
| Copy/move dest exists | Display: `error: file already exists: <dest>`. |

All errors are non-blocking. The terminal always returns to a usable prompt.

## 13. Security and Permissions

- `files.read` and `files.write` are required. Terminal Lite cannot function without file system access.
- No arbitrary command execution. Only the 16 whitelisted commands are recognized. Unknown commands are rejected.
- No shell metacharacter processing: `|`, `>`, `<`, `&&`, `||`, `$()`, backticks are treated as literal characters.
- Path traversal is bounded: `cd ../../..` cannot navigate above the user's home directory root within cortex-files.
- `rm` requires confirmation for directories. No `rm -rf /` equivalent is possible.
- No `sudo`, no privilege escalation, no access to system files outside the user's cortex-files namespace.
- The `whoami` command returns the user's display name from cortex-runtime, not a system username.
- Input sanitization: commands are tokenized, not evaluated. No injection vector through command arguments.

## 14. Performance Requirements

- Command output rendering: under 16 ms for up to 100 lines of output.
- Auto-complete response: under 100 ms (file listing query + filter).
- Scrollback scroll: 60 fps with 1000 lines rendered (virtualized).
- Startup first meaningful paint: under 300 ms.
- Memory: scrollback capped at 1000 lines. Each line max 10,000 characters. Approximate max memory: 10 MB for scrollback.
- Bundle size: under 150 KB gzipped.

## 15. Accessibility Requirements

- Terminal output area has `role="log"` with `aria-live="polite"` and `aria-label="Terminal output"`.
- New output is announced to screen readers (last line only, not full scrollback).
- Prompt input has `aria-label="Terminal command input"`.
- Auto-complete suggestions list has `role="listbox"` with `role="option"` for each suggestion.
- Error lines are marked with `role="alert"` for immediate screen reader announcement.
- Keyboard shortcuts are the primary interaction method (already keyboard-first by design).
- Focus never leaves the input field during normal operation (auto-complete suggestions are announced, not focused).
- Text sizing: monospace font scales with user settings. Layout remains functional at 200% text size.

## 16. Observability and Logging

Logged events:
- `terminal.launched` (info) -- App opened.
- `terminal.command.executed` (info) -- Command name only (no arguments, no output). Example payload: `{ command: "ls" }`.
- `terminal.command.error` (info) -- Command error type. Example payload: `{ command: "cat", error: "file_not_found" }`.
- `terminal.autocomplete.used` (debug) -- Auto-complete triggered.
- `terminal.files.unavailable` (warn) -- cortex-files call failed.
- `terminal.ai.explain_invoked` (info) -- AI explain action triggered.

No PII is logged. File paths, command arguments, and output content are never included in log payloads.

## 17. Testing Requirements

### 17.1 Unit Tests

- Command parser: tokenization, quoted strings, empty input, whitespace handling, special characters as literals.
- Path resolver: absolute paths, relative paths, `..`, `.`, trailing slashes, root boundary.
- Auto-completer: command completion with partial prefix, path completion with partial path, no matches, single match, multiple matches.
- Each command: correct output for valid input, correct error for invalid input, edge cases (empty directory for `ls`, non-existent path for `cd`, binary file for `cat`).

### 17.2 Integration Tests

- Full session flow: launch, execute `pwd`, `ls`, `cd`, `touch`, `cat`, `rm`, verify file system state through cortex-files.
- Multi-step workflow: `mkdir test`, `cd test`, `touch file.txt`, `echo hello`, `ls`, `cd ..`, `rm -r test`.
- Error recovery: invalid command, file not found, permission denied -- verify terminal remains usable after each error.
- Scrollback: generate 1000+ lines of output, verify oldest lines are pruned, verify scroll works.
- Command history: execute commands, verify up/down navigation, verify history persists across hot-reload.

### 17.3 Accessibility Tests

- AX tree validation for terminal display and prompt.
- Screen reader announcement of new output lines.
- Keyboard-only full workflow test.

## 18. Acceptance Criteria

- [ ] All 16 commands execute correctly with valid input.
- [ ] All commands produce correct error messages for invalid input.
- [ ] `cd` updates the prompt correctly. `pwd` reflects the new CWD.
- [ ] `ls` displays files and directories with `/` indicator.
- [ ] `cat` displays text file content and warns on binary files.
- [ ] `rm` requires confirmation for directories.
- [ ] `cp` and `mv` correctly copy/move files through cortex-files.
- [ ] Tab auto-complete works for command names and file paths.
- [ ] Up/down arrows navigate command history.
- [ ] Scrollback retains max 1000 lines, oldest pruned first.
- [ ] `clear` clears the terminal output.
- [ ] `help` lists all commands; `help <cmd>` shows specific usage.
- [ ] Unknown commands are rejected with an error message.
- [ ] No shell metacharacters are processed (`|`, `>`, `$()` are literal).
- [ ] Path traversal is bounded to user home directory.
- [ ] App launches in under 300 ms.
- [ ] All three themes render correctly (terminal uses dark background in all themes).
- [ ] Screen reader announces new output.
- [ ] AI panel opens and provides command context.
- [ ] Multi-instance: multiple terminal windows operate independently.
- [ ] Unit test coverage >= 80%.

## 19. Build Order and Dependencies
**Layer 11**. Depends on: 09 (app runtime), 11 (filesystem), 16 (theme tokens), 17 (first-party app parent)

### Prerequisites

- Spec 17 parent (first-party app framework).
- `@cortexos/ui-components` (shared UI library).
- `@cortexos/runtime-client` (for state persistence and user identity).
- `@cortexos/files-client` (for all file operations).
- `@cortexos/ai-client` (for AI surface).
- `@cortexos/theme` (design token consumer).

### Build Position

Terminal Lite is the **third** first-party app to build (after Clock Utilities and Calculator). It validates command bus integration with cortex-files for both read and write operations.

No Rust crate needed. Pure frontend app. All file operations go through the `@cortexos/files-client` TypeScript API.

## 20. Non-Goals and Anti-Patterns

### Non-Goals

- Full Unix shell compatibility or POSIX compliance.
- Arbitrary command execution or subprocess spawning.
- Scripting, pipes, redirections, or process management.
- Terminal emulation (no ANSI escape sequences beyond basic formatting).
- SSH or remote access.

### Anti-Patterns

- Using `eval()`, `new Function()`, or any dynamic code execution on user input.
- Accessing the filesystem directly instead of through cortex-files.
- Implementing `sudo` or any privilege escalation mechanism.
- Processing shell metacharacters (`|`, `>`, `<`, `$()`, backticks).
- Allowing path traversal above the user's cortex-files namespace.
- Blocking the UI thread on file operations (all cortex-files calls must be async).
- Executing multiple commands from a single input line.

## 21. Implementation Instructions for Claude Code / Codex

### Subsystem Ownership

- Terminal Lite owns: command parsing, command execution logic (the 16 commands), auto-complete, scrollback management, path resolution, prompt rendering.
- Terminal Lite does not own: file system access (delegates to cortex-files), user identity (delegates to cortex-runtime), window management.

### Recommended Implementation Order

1. Create `manifest.json` and validate against FirstPartyAppManifest schema.
2. Implement `services/command-parser.ts` -- tokenization with quote handling. Write comprehensive unit tests.
3. Implement `services/path-resolver.ts` -- path resolution logic. Write comprehensive unit tests.
4. Implement `services/command-registry.ts` -- command definitions and validation.
5. Implement commands in dependency order: `pwd`, `echo`, `date`, `whoami`, `help`, `clear`, `history`, `exit` (no cortex-files dependency), then `ls`, `cat`, `cd`, `mkdir`, `touch`, `rm`, `cp`, `mv` (require cortex-files). Write unit tests for each.
6. Implement `services/auto-completer.ts` -- command and path completion. Write unit tests.
7. Implement `components/TerminalDisplay.tsx` with virtualized scrollback rendering.
8. Implement `components/PromptLine.tsx` with prompt and input.
9. Wire up `App.tsx` connecting display, prompt, command executor, and auto-complete.
10. Integrate `@cortexos/runtime-client` for session state persistence.
11. Integrate `@cortexos/ai-client` for AI surface.
12. Add confirmation dialog for `rm` on directories.
13. Accessibility audit and fixes.
14. Theme verification (dark background in all themes).

### What Can Be Stubbed Initially

- AI "Explain command" action can return a placeholder explanation initially.
- Auto-complete can start with command-only completion (no path completion) until cortex-files integration is ready.
- Virtualized rendering can use simple rendering initially, then be optimized once correctness is verified.

### What Must Be Real in V1

- All 16 commands with correct behavior.
- Command parser with quote handling.
- Path resolution with `..` and root boundary enforcement.
- Tab auto-complete for command names and file paths.
- Scrollback with 1000-line limit.
- Command history with up/down navigation (500 entries).
- Confirmation prompt for `rm` on directories.
- No shell metacharacter processing (security-critical).
- Theme support (dark background in all themes).
- Accessibility (keyboard navigation, screen reader).

### What Cannot Be Inferred

- Exact prompt format (`user@hostname:~/path$ ` -- hostname from cortex-runtime).
- Terminal color scheme (dark background, light text, red errors -- hardcoded, not theme-derived).
- Monospace font choice (consume from design tokens, fallback to `monospace`).
- Default window size (720x480 per manifest).
- Auto-complete double-Tab timing threshold (500 ms).

### Stop Conditions

1. All unit tests pass with >= 80% coverage.
2. Command parser tests cover quoting, empty input, and special characters.
3. Path resolver tests cover absolute/relative paths, `..`, root boundary.
4. Integration tests for a full file operation workflow pass.
5. No `eval()` or dynamic code execution confirmed by code review and linter rule.
6. Security review confirms no shell metacharacter processing.
7. Multi-instance test: two windows operate independently.
8. All three themes render correctly (dark background in all).

### Testing Gates

- Command parser and path resolver unit tests must pass before any command implementations are written.
- File operation commands must pass integration tests before merge.
- Security tests (no eval, no shell metacharacters, path traversal bounded) must pass before merge.
- Keyboard navigation test must pass before merge.
