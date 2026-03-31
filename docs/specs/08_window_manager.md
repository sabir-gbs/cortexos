# 08 — Window Manager

## 1. Purpose
Define the window management system for CortexOS's browser-rendered desktop, handling window creation, positioning, resizing, state management, and virtual workspaces.

## 2. Scope
- Window lifecycle (create, position, resize, minimize, maximize, restore, close)
- Z-ordering and focus management
- Window chrome (title bar, controls)
- Snap/tiling (halves, quarters)
- Virtual workspaces (up to 4)
- Keyboard shortcuts for window management
- State persistence across sessions

## 3. Out of Scope
- App rendering within windows (owned by spec 09)
- Desktop shell layout (owned by spec 07)
- Drag-and-drop content between windows (owned by apps via command bus)

## 4. Objectives
1. Every window operation is reversible and persisted.
2. Keyboard-only users can manage all windows without a mouse.
3. Crashed apps don't affect other windows or the window manager.
4. Window state survives OS restart.

## 5. User-Visible Behavior

| Action | Outcome |
|---|---|
| User opens an app | New window appears on current workspace, focused |
| User minimizes | Window shrinks to taskbar icon, no longer visible |
| User maximizes | Window fills screen, title bar remains |
| User closes | Window removed, app receives stop signal |
| User drags title bar | Window moves to new position |
| User drags edge/corner | Window resizes within min-size constraints |
| User snaps to edge | Window fills that half/quarter of screen |
| Win+Left/Right | Snap to left/right half |
| Ctrl+Win+Left/Right | Switch workspace |
| Alt+F4 | Close focused window |
| Alt+Tab | Cycle focus through windows on current workspace |

## 6. System Behavior

### 6.1 Window State Machine
```
Creating → Active ↔ Minimized
         → Active ↔ Maximized
         → Active → Closing → Closed
```

### 6.2 Focus Rules
- Exactly one window is focused at a time (or none if all minimized)
- Clicking a window focuses it and brings to top of z-order
- Newly created windows get focus automatically
- Minimized windows lose focus; focus moves to next topmost window

### 6.3 Z-Ordering
- Each window has a z_order integer (higher = on top)
- Focusing a window sets its z_order to max(z_orders) + 1
- Z-orders are compacted periodically (garbage collection)

### 6.4 Snap/Tiling Positions
- Left half: x=0, y=0, w=50%, h=100%
- Right half: x=50%, y=0, w=50%, h=100%
- Top-left quarter: x=0, y=0, w=50%, h=50%
- Top-right quarter: x=50%, y=0, w=50%, h=50%
- Bottom-left quarter: x=0, y=50%, w=50%, h=50%
- Bottom-right quarter: x=50%, y=50%, w=50%, h=50%

## 7. Architecture

TypeScript frontend module. State managed in browser, synced to server via WebSocket for persistence.

```
┌─────────────────────────────────┐
│       Window Manager (TS)       │
│  ┌────────────────────────────┐ │
│  │ Layout Engine              │ │
│  │ (position, size, snap)     │ │
│  └────────────┬───────────────┘ │
│  ┌────────────┴───────────────┐ │
│  │ Focus & Z-Order Manager    │ │
│  └────────────┬───────────────┘ │
│  ┌────────────┴───────────────┐ │
│  │ Workspace Manager          │ │
│  └────────────┬───────────────┘ │
│  ┌────────────┴───────────────┐ │
│  │ State Persistence          │ │
│  │ (sync to server)           │ │
│  └────────────────────────────┘ │
└─────────────────────────────────┘
```

## 8. Data Model

```typescript
interface WindowState {
  id: string;                    // UUID
  app_id: string;                // App manifest ID
  app_instance_id: string;       // Unique per instance
  title: string;                 // Displayed in title bar
  position: { x: number; y: number };
  size: { width: number; height: number };
  min_size: { width: number; height: number };  // From app manifest
  z_order: number;
  state: "active" | "minimized" | "maximized";
  workspace_id: number;          // 0-3
  snap_zone: SnapZone | null;
  icon_url: string | null;
  created_at: string;
  updated_at: string;
}

type SnapZone =
  | "left-half" | "right-half"
  | "top-left" | "top-right"
  | "bottom-left" | "bottom-right";

interface WorkspaceState {
  id: number;                    // 0-3
  name: string;                  // User-configurable
  is_active: boolean;
  window_ids: string[];
}
```

## 9. Public Interfaces

### Command Bus Commands
```
wm.window.move         → { window_id, x, y }
wm.window.resize       → { window_id, width, height }
wm.window.set_state    → { window_id, state: "active" | "minimized" | "maximized" }
wm.window.close        → { window_id }
wm.window.focus        → { window_id }
wm.window.move_workspace → { window_id, workspace_id }
wm.workspace.activate  → { workspace_id }
```

### HTTP Snapshot Endpoints
```
GET /api/v1/windows     → List all windows for current session bootstrap / recovery
GET /api/v1/workspaces  → List workspaces for current session bootstrap / recovery
```

### WebSocket Events
```
window.created       -> WindowState
window.updated       -> WindowState (partial: changed fields)
window.closed        -> { window_id }
window.focused       -> { window_id }
wm.workspace.changed  → { workspace_id }
```

## 10. Internal Interfaces
- Reads app manifest from cortex-runtime (app name, icon, min_size, single_instance)
- Emits events via command bus
- Browser clients issue realtime window mutations through command bus commands; HTTP is limited to bootstrap/snapshot reads
- Desktop shell subscribes to window events for taskbar rendering

## 11. State Management
- Window state maintained in browser memory for real-time rendering
- Synced to server after every state change (debounced 500ms for position/size during drag)
- Full state snapshot on workspace switch, minimize, maximize, close
- State restored on OS restart from server

## 12. Failure Modes and Error Handling

| Failure | Handling |
|---|---|
| Server unreachable during state sync | Queue locally, sync on reconnect |
| Invalid position (off-screen) | Clamp to visible area |
| Resize below min_size | Enforce min_size constraint |
| Workspace full (max windows) | No hard limit; performance degrades gracefully |

## 13. Security and Permissions
- Apps cannot programmatically move/resize other apps' windows
- Apps can only request their own window state changes via API
- Window focus changes require user interaction (click or keyboard)

## 14. Performance Requirements

| Metric | Target |
|---|---|
| Window creation to visible | < 100ms |
| Drag/resize frame rate | 60fps |
| State sync to server | < 200ms (debounced) |
| Workspace switch | < 150ms |
| Full state restore | < 500ms |

## 15. Accessibility Requirements
- All window controls accessible via keyboard (Alt+F4, Alt+Space, Win+arrows)
- Title bar buttons have ARIA labels ("Minimize", "Maximize", "Close")
- Focus indicator visible on window chrome
- Snap zones announced to screen readers
- Workspace switching announced via ARIA live region

## 16. Observability and Logging
- Window creation/close logged at INFO
- State sync failures logged at WARN
- Performance metrics: window creation time, drag frame rate

## 17. Testing Requirements
- Unit: position clamping, snap calculations, z-order management
- Integration: state sync round-trip, workspace switching
- E2E: open app → resize → minimize → restore → close → verify persistence

## 18. Acceptance Criteria
- [ ] Windows can be created, moved, resized, minimized, maximized, closed
- [ ] Keyboard shortcuts work for all operations
- [ ] Snap to halves/quarters works with Win+Left/Right
- [ ] 4 workspaces switchable with Ctrl+Win+Left/Right
- [ ] State persists across OS restart
- [ ] Min-size constraints enforced
- [ ] Focus management: exactly one focused or none
- [ ] All controls accessible via keyboard and screen reader

## 19. Build Order and Dependencies
**Layer 7**. Depends on: 01, 02, 09 (app runtime for manifests)

## 20. Non-Goals and Anti-Patterns
- No automatic tiling (manual snap only)
- No window transparency effects (v1)
- No multi-monitor support (browser-rendered, single viewport)
- NEVER allow apps to control other apps' windows

## 21. Implementation Instructions for Claude Code / Codex
1. Implement WindowState and WorkspaceState TypeScript interfaces.
2. Build layout engine: position/size calculations, snap zone detection.
3. Build focus/z-order manager: track focused window, z-order compaction.
4. Build workspace manager: 4 workspaces, active switching, window membership.
5. Implement command-bus commands for realtime window mutations and HTTP snapshot endpoints for bootstrap/recovery reads.
6. Implement keyboard shortcut handler.
7. Write tests: snap calculations, focus rules, state persistence round-trip.
