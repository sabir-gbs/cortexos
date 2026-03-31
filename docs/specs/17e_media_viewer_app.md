# 17e. Media Viewer App

## 1. Purpose

The Media Viewer app provides a unified, read-only viewer for images, audio, and video files in CortexOS. It handles common media formats with appropriate playback controls and metadata display, serving as the default viewer for file associations in its supported format set.

## 2. Scope

- **Image viewing**: PNG, JPG/JPEG, GIF (including animated), SVG, WebP, BMP.
  - Zoom in/out (keyboard and gesture), pan (drag), fit-to-window, actual-size (1:1 pixel).
  - Previous/next navigation within the source folder.
  - Image info panel: dimensions (width x height), file size, MIME type.
- **Audio playback**: MP3, WAV, OGG.
  - Play/pause, seek (slider), volume control, duration display (elapsed/total).
- **Video playback**: MP4, WebM.
  - Standard HTML5 video controls: play/pause, seek, volume, fullscreen toggle, duration display.
- File association registration for all listed MIME types with role `viewer`.
- App location: `apps/media-viewer-app`.

## 3. Out of Scope

- Media editing (crop, rotate, filter, trim, annotate).
- Playlist or queue management.
- Audio visualization (waveform, spectrum).
- Video subtitle rendering.
- RAW image format support.
- Screen recording or media capture.
- Media library management or tagging.
- HEIF/HEIC image support.
- Streaming from network URLs (local files only via cortex-files).

## 4. Objectives

1. Provide a fast, reliable viewer for the most common image, audio, and video formats.
2. Validate the cortex-files read path with binary media content (non-text file handling).
3. Demonstrate file association registration and folder-based navigation (previous/next).
4. Expose metadata (dimensions, size, type) through the info panel and AI context.

## 5. User-Visible Behavior

### 5.1 Image Mode

- Image is displayed centered in the viewport. Default view is fit-to-window (image scaled to fit without cropping).
- **Zoom**: `+`/`-` keys, scroll wheel, or toolbar buttons. Zoom range: 10% to 1000%. Zoom centers on the cursor position for scroll-wheel zoom.
- **Pan**: Click and drag when zoomed beyond fit-to-window. Cursor changes to grab/grabbing.
- **Fit-to-window**: Toolbar button or `Ctrl+0`. Resets zoom to fit the image in the viewport.
- **Actual size**: Toolbar button or `Ctrl+1`. Sets zoom to 100% (1:1 pixel ratio).
- **Previous/Next**: Left/Right arrow keys or toolbar chevrons navigate to the prev/next image file in the same folder. Non-image files in the folder are skipped. Wraps around at boundaries with a toast notification ("No more images in this folder").
- **Info panel**: Toggle via toolbar button or `Ctrl+I`. Slide-in panel showing: file name, dimensions (e.g., "1920 x 1080 px"), file size (e.g., "2.4 MB"), MIME type, and for GIFs: animation frame count if available.
- **Animated GIF**: Plays automatically. No playback controls. Click to pause/resume animation.

### 5.2 Audio Mode

- Displays album art placeholder (a generic waveform icon) centered in the viewport.
- Transport bar at the bottom: play/pause button, seek slider, elapsed time / total duration, volume slider with mute toggle.
- Keyboard: `Space` = play/pause, `Left`/`Right` = seek +/- 5 seconds, `Up`/`Down` = volume +/- 10%, `Home` = seek to start.
- Audio plays through the system audio output. No audio visualization in V1.

### 5.3 Video Mode

- Video is displayed centered, fit-to-window by default (letterboxed, aspect ratio preserved).
- Standard HTML5 video control bar: play/pause, seek slider, elapsed/total time, volume slider with mute, fullscreen toggle.
- Double-click toggles fullscreen. `Escape` exits fullscreen.
- Keyboard: `Space` = play/pause, `Left`/`Right` = seek +/- 5 seconds, `Up`/`Down` = volume +/- 10%, `F` = fullscreen toggle, `M` = mute toggle.
- Previous/next navigation not available in video mode (video files only).

### 5.4 General

- Window title shows the file name.
- Dragging a media file onto the window opens it.
- Unsupported format shows an error dialog: "This file format is not supported."

## 6. System Behavior

### 6.1 File Loading

- Files are loaded via `cortex-files.files.read(handle)` which returns an ArrayBuffer.
- Images: loaded into an `Image` object via `URL.createObjectURL`. Object URL is revoked on unmount or file change.
- Audio: loaded via `URL.createObjectURL` into an `HTMLAudioElement`.
- Video: loaded via `URL.createObjectURL` into an `HTMLVideoElement`.
- For SVG files, content is loaded as text and sanitized before rendering to prevent script injection. SVG scripts, event handlers, and external resource references are stripped.

### 6.2 Folder Navigation

- When a file is opened, the app queries `cortex-files.files.list(parentDirectory)` to enumerate sibling files.
- Files are filtered to supported media types and sorted lexicographically by name.
- The current file index is tracked. Previous/next moves within this sorted list.
- Folder enumeration is cached for the duration the file is open. Refreshing the folder listing requires reopening the file.

### 6.3 Metadata Extraction

- Image dimensions: extracted from the loaded `Image` object (`naturalWidth`, `naturalHeight`).
- File size: from the file metadata returned by cortex-files (`size` field).
- MIME type: from the file metadata returned by cortex-files (`mimeType` field).
- Duration (audio/video): from the `duration` property of the media element after `loadedmetadata` event.
- No EXIF parsing in V1.

### 6.4 App Lifecycle

- Single-instance app. Opening a second file when the viewer is already open replaces the current file (with a confirmation dialog if the media is currently playing audio/video).
- On unmount, object URLs are revoked and media elements are paused and cleaned up.
- State persisted across hot-reload: current file handle, zoom level, pan offset.
- State not persisted across sessions.

### 6.5 File Associations

```typescript
fileAssociations: [
  { extension: ".png",  mimeType: "image/png",        role: "viewer" },
  { extension: ".jpg",  mimeType: "image/jpeg",       role: "viewer" },
  { extension: ".jpeg", mimeType: "image/jpeg",       role: "viewer" },
  { extension: ".gif",  mimeType: "image/gif",        role: "viewer" },
  { extension: ".svg",  mimeType: "image/svg+xml",    role: "viewer" },
  { extension: ".webp", mimeType: "image/webp",       role: "viewer" },
  { extension: ".bmp",  mimeType: "image/bmp",        role: "viewer" },
  { extension: ".mp3",  mimeType: "audio/mpeg",       role: "viewer" },
  { extension: ".wav",  mimeType: "audio/wav",        role: "viewer" },
  { extension: ".ogg",  mimeType: "audio/ogg",        role: "viewer" },
  { extension: ".mp4",  mimeType: "video/mp4",        role: "viewer" },
  { extension: ".webm", mimeType: "video/webm",       role: "viewer" }
]
```

## 7. Architecture

```
apps/media-viewer-app/
  manifest.json
  package.json
  src/
    main.ts                    # Registers app with runtime
    App.tsx                    # Root component, mode router
    components/
      ImageViewer.tsx          # Image display, zoom, pan
      AudioPlayer.tsx          # Audio transport controls
      VideoPlayer.tsx          # Video display and controls
      TransportBar.tsx         # Shared seek/volume/duration bar
      InfoPanel.tsx            # Slide-in metadata panel
      Toolbar.tsx              # Top toolbar (zoom, nav, info toggle)
    services/
      media-loader.ts          # File loading, object URL management
      folder-navigator.ts      # Folder enumeration, prev/next logic
      metadata-extractor.ts    # Dimension, duration, size extraction
      svg-sanitizer.ts         # SVG content sanitization
    hooks/
      useZoom.ts               # Zoom state and transforms
      usePan.ts                # Pan offset state with drag handling
      useMediaPlayback.ts      # Play/pause, seek, volume state
      useFolderNavigation.ts   # Previous/next file navigation
    ai/
      context.ts               # Provides file metadata to AI
      actions.ts               # "Describe this image" action
    types.ts
  tests/
    unit/
      folder-navigator.test.ts # Folder sort/filter logic
      metadata-extractor.test.ts
      svg-sanitizer.test.ts
    integration/
      image-viewer.test.ts     # Zoom, pan, nav flows
      audio-player.test.ts     # Playback control flows
      video-player.test.ts     # Playback and fullscreen flows
```

No Rust backend crate needed. All decoding uses browser-native capabilities (HTMLImageElement, HTMLAudioElement, HTMLVideoElement).

## 8. Data Model

### 8.1 Viewer State

```typescript
interface MediaViewerState {
  currentFile: FileHandle | null;
  mediaType: "image" | "audio" | "video" | null;
  imageState: {
    zoom: number;            // 0.1 to 10.0 (1.0 = actual size)
    panX: number;
    panY: number;
    fitMode: "fit" | "actual";
    infoPanelOpen: boolean;
  };
  playbackState: {
    playing: boolean;
    currentTime: number;
    duration: number;
    volume: number;          // 0.0 to 1.0
    muted: boolean;
  };
  folderState: {
    siblingFiles: FileHandle[];
    currentIndex: number;
  };
}
```

### 8.2 File Metadata

```typescript
interface MediaMetadata {
  fileName: string;
  fileSize: number;          // bytes
  mimeType: string;
  width?: number;            // images and video
  height?: number;           // images and video
  duration?: number;         // audio and video, seconds
}
```

### 8.3 Manifest

```typescript
{
  id: "com.cortexos.media-viewer",
  name: "Media Viewer",
  version: "1.0.0",
  description: "View images, play audio and video files",
  firstParty: true,
  bundled: true,
  entry: { frontend: "src/main.ts" },
  window: {
    defaultWidth: 800,
    defaultHeight: 600,
    minWidth: 400,
    minHeight: 300,
    resizable: true,
    singleInstance: true
  },
  permissions: {
    required: ["runtime.state", "runtime.lifecycle", "files.read"],
    optional: ["clipboard.write", "ai.context", "ai.invoke"]
  },
  fileAssociations: [
    { extension: ".png",  mimeType: "image/png",        role: "viewer" },
    { extension: ".jpg",  mimeType: "image/jpeg",       role: "viewer" },
    { extension: ".jpeg", mimeType: "image/jpeg",       role: "viewer" },
    { extension: ".gif",  mimeType: "image/gif",        role: "viewer" },
    { extension: ".svg",  mimeType: "image/svg+xml",    role: "viewer" },
    { extension: ".webp", mimeType: "image/webp",       role: "viewer" },
    { extension: ".bmp",  mimeType: "image/bmp",        role: "viewer" },
    { extension: ".mp3",  mimeType: "audio/mpeg",       role: "viewer" },
    { extension: ".wav",  mimeType: "audio/wav",        role: "viewer" },
    { extension: ".ogg",  mimeType: "audio/ogg",        role: "viewer" },
    { extension: ".mp4",  mimeType: "video/mp4",        role: "viewer" },
    { extension: ".webm", mimeType: "video/webm",       role: "viewer" }
  ],
  ai: {
    surfaces: { assistantPanel: true, contextMenu: true, inlineSuggestions: false },
    contextProviders: ["media-viewer-context"],
    actions: [
      {
        id: "describe-image",
        label: "Describe this image",
        description: "Provides an AI-generated description of the current image",
        confirmationRequired: false,
        destructive: false
      }
    ]
  },
  accessibility: { highContrastSupport: true, screenReaderSupport: true, keyboardNavigation: true },
  category: "media"
}
```

## 9. Public Interfaces

### 9.1 AI Context

```typescript
interface MediaViewerAIContext {
  fileName: string;
  mediaType: "image" | "audio" | "video";
  mimeType: string;
  fileSize: number;
  width?: number;
  height?: number;
  duration?: number;
  zoom?: number;
}
```

### 9.2 File Open Command

The app accepts the standard `files.open` command with a file handle, dispatched by cortex-files when the user opens an associated file type.

## 10. Internal Interfaces

### 10.1 Media Loader

```typescript
interface MediaLoader {
  loadImage(handle: FileHandle): Promise<{ objectUrl: string; width: number; height: number }>;
  loadAudio(handle: FileHandle): Promise<{ objectUrl: string; duration: number }>;
  loadVideo(handle: FileHandle): Promise<{ objectUrl: string; width: number; height: number; duration: number }>;
  revokeCurrent(): void;
}
```

### 10.2 Folder Navigator

```typescript
interface FolderNavigator {
  enumerateSiblings(fileHandle: FileHandle): Promise<FileHandle[]>;
  getPrevious(currentIndex: number, files: FileHandle[]): FileHandle | null;
  getNext(currentIndex: number, files: FileHandle[]): FileHandle | null;
}
```

### 10.3 SVG Sanitizer

```typescript
interface SvgSanitizer {
  sanitize(rawSvgText: string): string;  // Returns stripped SVG with scripts/event handlers/external refs removed
}
```

## 11. State Management

- **Ephemeral**: Zoom transform CSS, pan drag offset, playback progress (updated on animation frame), toolbar hover states.
- **Session**: Current file handle, zoom level, pan offset, volume, muted state, info panel open/closed. Persisted via cortex-runtime session state for hot-reload survival.
- **Persistent**: Last volume level, last muted state. Key: `com.cortexos.media-viewer.preferences`.
- Object URLs are not persisted. They are recreated from file handles on restore.

## 12. Failure Modes and Error Handling

| Failure | Behavior |
|---------|----------|
| File not found | Display error dialog: "File not found. It may have been moved or deleted." Log as warn. |
| Unsupported format | Display error dialog: "This file format is not supported." Log as info. |
| Corrupt/unreadable file | Display error dialog: "Unable to load this file. The file may be corrupted." Log as warn with MIME type. |
| Browser cannot decode codec | Display error dialog: "This media format is not supported by your browser." Log as warn. |
| Folder enumeration fails | Disable prev/next navigation. Show toast: "Could not read folder contents." Viewer remains functional for current file. |
| SVG sanitization strips content | Render the sanitized SVG. Log info event if script or external reference was removed. |
| File too large for memory (>200 MB) | Display error dialog: "File is too large to display." Log as warn with file size. |
| cortex-files unavailable | Display offline banner. Disable file operations. Cannot open new files. |

All errors are non-blocking where possible. The viewer remains functional for the current file even if navigation fails.

## 13. Security and Permissions

- `files.read` is required. Media Viewer cannot function without filesystem read access.
- SVG content is sanitized before rendering: remove `<script>` elements, `on*` event handler attributes, `href`/`xlink:href` pointing to external resources (javascript:, data: URIs with script content), and `<use>` referencing external documents.
- Object URLs are scoped to the document and revoked on cleanup to prevent resource leaks.
- No `eval()`, no `innerHTML` with unsanitized content.
- AI action "Describe image" sends the image to the AI provider. User is informed that image data is shared with the AI service.
- No network requests are made by the viewer itself. All file access is via cortex-files.

## 14. Performance Requirements

- Image load and display: under 200 ms for files up to 20 MB.
- Zoom and pan must maintain 60 fps (CSS transform-based, no re-render).
- Folder enumeration for prev/next: under 100 ms for directories with up to 1000 files.
- Audio/video seek: under 100 ms for seek response.
- Memory: a single image loaded at a time. Previous/next releases the prior image's object URL.
- Startup first meaningful paint: under 400 ms.
- Bundle size: under 200 KB gzipped.

## 15. Accessibility Requirements

- All transport controls have ARIA labels: "Play", "Pause", "Seek bar", "Volume", "Mute", "Fullscreen".
- Seek and volume sliders have `aria-valuenow`, `aria-valuemin`, `aria-valuemax`.
- Image info panel uses `role="dialog"` with `aria-label="File information"`.
- Previous/next buttons have labels: "Previous image", "Next image".
- Zoom controls: "Zoom in", "Zoom out", "Fit to window", "Actual size".
- Keyboard navigation through all toolbar controls in logical order.
- Media playback announced via `aria-live="polite"` on status region: "Playing", "Paused", "End of media".
- Color is not the sole indicator of play state (icon changes from play to pause symbol).

## 16. Observability and Logging

Logged events:
- `media.launched` (info) -- App opened, no file details.
- `media.file.opened` (info) -- File opened, includes media type category ("image"/"audio"/"video") and MIME type. No file name or path.
- `media.file.error` (warn) -- File load failure, includes error type. No file content.
- `media.svg.sanitized` (info) -- SVG had content removed during sanitization.
- `media.ai.describe_invoked` (info) -- AI describe action triggered.
- `media.error` (warn) -- Playback error, seek error, or decode failure.

No PII is logged. File names, paths, and content are never included in log payloads.

## 17. Testing Requirements

### 17.1 Unit Tests

- Folder navigator: sorting, filtering by supported types, wrapping behavior, empty directory, single-file directory.
- Metadata extractor: dimension extraction from mock image objects, duration from mock media elements.
- SVG sanitizer: strips `<script>`, strips `onclick`/`onerror` attributes, strips external `href`, preserves safe SVG content.
- Zoom calculations: fit-to-window scale, actual-size, min/max clamping.

### 17.2 Integration Tests

- Image open: load PNG/JPG/GIF/SVG/WebP/BMP, verify display and metadata.
- Zoom/pan: zoom in, zoom out, fit-to-window, actual-size, pan with drag.
- Prev/next: navigate through a folder of mixed files, verify only supported media is visited.
- Audio playback: play, pause, seek, volume change, verify state updates.
- Video playback: play, pause, seek, volume, fullscreen toggle.
- File association: verify registered extensions match spec.

### 17.3 Accessibility Tests

- AX tree validation for image viewer with info panel open.
- AX tree validation for audio player with transport bar.
- Keyboard-only navigation through all controls.
- Screen reader announcement of playback state changes.

## 18. Acceptance Criteria

- [ ] All 12 file types (PNG, JPG, JPEG, GIF, SVG, WebP, BMP, MP3, WAV, OGG, MP4, WebM) open and display correctly.
- [ ] Image zoom range 10%--1000% works with keyboard, mouse wheel, and toolbar buttons.
- [ ] Pan works when zoomed beyond fit-to-window.
- [ ] Fit-to-window and actual-size reset zoom correctly.
- [ ] Previous/next navigates only through supported media files in the folder.
- [ ] Info panel displays correct dimensions, file size, and MIME type.
- [ ] Animated GIF plays and pauses on click.
- [ ] Audio play/pause, seek, volume, and mute all work.
- [ ] Video play/pause, seek, volume, mute, and fullscreen all work.
- [ ] SVG files are sanitized (no script execution from SVG).
- [ ] Corrupt or unsupported files show user-friendly error dialogs.
- [ ] App launches in under 400 ms.
- [ ] Zoom/pan maintains 60 fps.
- [ ] All three themes render correctly.
- [ ] Screen reader announces playback state changes.
- [ ] Keyboard shortcuts work for all documented actions.
- [ ] AI panel opens and provides file metadata context.
- [ ] Unit test coverage >= 80%.

## 19. Build Order and Dependencies
**Layer 11**. Depends on: 09 (app runtime), 11 (filesystem), 16 (theme tokens), 17 (first-party app parent)

### Prerequisites

- Spec 17 parent (first-party app framework).
- `@cortexos/ui-components` (shared UI library).
- `@cortexos/runtime-client` (for state persistence).
- `@cortexos/files-client` (for file reading and folder enumeration).
- `@cortexos/ai-client` (for AI surface).
- `@cortexos/theme` (design token consumer).

### Build Position

Media Viewer is the **fourth** first-party app to build (after Clock Utilities, Calculator, and Terminal Lite). It validates the cortex-files read path with binary content and file association registration.

No Rust crate needed. Pure frontend app using browser-native media decoding.

## 20. Non-Goals and Anti-Patterns

### Non-Goals

- Media editing of any kind.
- Playlist, queue, or library management.
- Network streaming or URL-based playback.
- EXIF data parsing or display.
- Thumbnail generation (that belongs in File Manager).

### Anti-Patterns

- Using `innerHTML` or `dangerouslySetInnerHTML` with unsanitized SVG content.
- Loading files via direct filesystem paths instead of cortex-files handles.
- Keeping object URLs alive after navigating away from a file (memory leak).
- Implementing custom media decoders (use browser-native capabilities).
- Blocking the UI thread during large file reads (use streaming or chunked loading).
- Hardcoding codec support lists (query `HTMLMediaElement.canPlayType` at runtime).

## 21. Implementation Instructions for Claude Code / Codex

### Subsystem Ownership

- Media Viewer owns: image display and zoom/pan, audio/video transport controls, folder navigation logic, SVG sanitization, metadata extraction from loaded media elements.
- Media Viewer does not own: file system access (delegates to cortex-files), audio/video decoding (delegates to browser), AI inference, window management.

### Recommended Implementation Order

1. Create `manifest.json` and validate against FirstPartyAppManifest schema.
2. Implement `services/media-loader.ts` -- file loading and object URL management. Write unit tests.
3. Implement `services/folder-navigator.ts` -- folder enumeration and prev/next logic. Write unit tests.
4. Implement `services/svg-sanitizer.ts` -- SVG stripping logic. Write thorough unit tests (security-critical).
5. Implement `components/ImageViewer.tsx` with zoom and pan.
6. Implement `components/AudioPlayer.tsx` with transport bar.
7. Implement `components/VideoPlayer.tsx` with standard controls.
8. Implement `components/InfoPanel.tsx` with metadata display.
9. Wire up `App.tsx` to route between image/audio/video modes based on MIME type.
10. Integrate `@cortexos/runtime-client` for session state persistence.
11. Integrate `@cortexos/ai-client` for AI surface and "Describe image" action.
12. Add keyboard shortcuts for all modes.
13. Accessibility audit and fixes.
14. Theme verification (light, dark, high-contrast).

### What Can Be Stubbed Initially

- AI "Describe image" action can return a placeholder response initially.
- Folder enumeration can return an empty list until cortex-files integration is complete (app works for single-file open).

### What Must Be Real in V1

- All 12 format types load and display/play correctly.
- Zoom (10%--1000%), pan, fit-to-window, actual-size.
- Previous/next folder navigation.
- Audio and video transport controls (play/pause/seek/volume).
- SVG sanitization (security-critical, cannot be deferred).
- Info panel with correct metadata.
- Theme support.
- Accessibility (keyboard navigation, screen reader).
- Object URL cleanup (no memory leaks).

### What Cannot Be Inferred

- SVG sanitization rules must be exhaustive (this is security-critical). Follow OWASP SVG sanitization guidelines.
- Zoom/pan transform origin and behavior (center on cursor for wheel zoom).
- Default window size (800x600 per manifest).
- Video letterboxing behavior (aspect ratio preserved with black bars).

### Stop Conditions

1. All unit tests pass with >= 80% coverage.
2. SVG sanitizer unit tests cover all known attack vectors (script injection, event handlers, external resources).
3. Integration tests for all three modes (image, audio, video) pass.
4. Accessibility tests pass.
5. Manifest validates.
6. No `innerHTML` with unsanitized content confirmed by code review and linter rule.
7. Object URL leak test: open 50 images sequentially, verify no growth in object URL count.
8. All three themes render correctly.

### Testing Gates

- SVG sanitizer tests must pass before any SVG rendering code is merged.
- Memory leak test (open/close 100 images) must pass before merge.
- Folder navigator tests must pass before prev/next UI is wired up.
- Keyboard navigation test must pass before merge.
