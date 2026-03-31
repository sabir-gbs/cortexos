# 17g. Clock and Utility Apps

## 1. Purpose

The Clock and Utility app provides time-related tools for CortexOS: a digital and analog clock display, a countdown timer with alarm, a stopwatch with lap tracking, a multi-alarm system with snooze and day-of-week scheduling, and a world clock for comparing time across timezones. All tools share a single app window with a tab-based UI.

## 2. Scope

- **Clock**: Digital display (12h/24h toggle) and analog clock face with hour, minute, and second hands.
- **Timer**: Countdown timer. Set duration (hours, minutes, seconds), start, pause, resume, reset. Alarm sounds when timer reaches zero. Multiple concurrent timers supported (up to 4).
- **Stopwatch**: Start, stop, resume, reset. Lap recording with lap times and total elapsed time display.
- **Alarm**: Set alarms with time, label, repeat days (Mon-Sun), and sound. Dismiss or snooze (5 minutes). Multiple alarms supported (up to 20). Alarms fire even when the app is closed (via runtime notification system).
- **World Clock**: Display current time in multiple cities/timezones. Add/remove cities. Shows time difference relative to local time.
- Tab-based UI to switch between Clock, Timer, Stopwatch, Alarm, and World Clock.
- App location: `apps/clock-utils-app`.

## 3. Out of Scope

- Calendar or date picker (belongs in a separate Calendar app).
- Time zone conversion calculator (the world clock shows current times, not arbitrary conversions).
- Sleep tracking or bedtime reminders.
- Custom alarm sounds (V1 uses system default sounds).
- Widget or desktop clock (future consideration).
- Pomodoro or productivity timer modes.
- Sunrise/sunset or astronomical data.
- International clock sync protocol (NTP client).

## 4. Objectives

1. Provide essential time-keeping utilities in a single cohesive app.
2. Validate background notification delivery via cortex-runtime for alarm firing.
3. Demonstrate tab-based navigation within a single app window.
4. Serve as the reference for a utility app with persistent settings (alarms survive app restart).

## 5. User-Visible Behavior

### 5.1 Tab Navigation

- Five tabs across the top of the window: Clock, Timer, Stopwatch, Alarm, World Clock.
- Each tab shows an icon and label. Active tab is visually emphasized.
- Clicking a tab or using `Ctrl+1` through `Ctrl+5` switches between tabs.
- Tab state is maintained independently. Switching tabs does not lose stopwatch state or timer progress.

### 5.2 Clock Tab

- **Digital display**: Large text showing current time in HH:MM:SS format. AM/PM indicator shown in 12-hour mode.
- **Analog display**: Circular clock face below the digital display. Hour hand, minute hand, second hand. Hour markers (12 marks, no numbers). Current date displayed below the clock face.
- **12h/24h toggle**: A toggle switch in the clock tab switches between 12-hour and 24-hour display. Setting persists across sessions.
- Time updates every second. Analog second hand sweeps (updates per second, not continuously).

### 5.3 Timer Tab

- **Set panel**: Three scroll/input fields for hours (0-23), minutes (0-59), seconds (0-59). Default: 0h 5m 0s.
- **Timer display**: When running, shows remaining time in HH:MM:SS, large centered text. Circular progress ring around the display showing remaining time proportionally.
- **Controls**: Start button (begins countdown), Pause button (visible while running), Resume button (visible while paused), Reset button (returns to set panel).
- **Completion**: When timer reaches zero, plays an alarm sound and shows a notification: "Timer complete." The display flashes until the user clicks Dismiss.
- **Multiple timers**: A "+" button allows creating additional timers (up to 4). Running timers are shown as mini-cards below the main timer. Each mini-card shows remaining time and a pause/reset button.

### 5.4 Stopwatch Tab

- **Display**: Large time display showing elapsed time in MM:SS.cc format (centiseconds).
- **Controls**: Start button (begins timing), Stop button (pauses), Resume button (continues), Reset button (clears to zero and clears laps).
- **Lap button**: Visible while running. Pressing Lap records the current elapsed time as a lap. Each lap entry shows: lap number, lap time (delta from previous lap), total elapsed time.
- **Lap list**: Scrollable list below the display. Sorted newest-first. Best lap time highlighted in green, worst lap time highlighted in red (when 3+ laps exist).
- Maximum laps: 100. Oldest laps are pruned.

### 5.5 Alarm Tab

- **Alarm list**: Scrollable list of configured alarms. Each entry shows: time (HH:MM), label (if set), repeat days (e.g., "Mon, Wed, Fri"), enabled/disabled toggle.
- **Add alarm**: A "+" button or "Add Alarm" button opens an alarm editor.
- **Alarm editor**: Time picker (hour, minute), label text input (optional, max 50 chars), repeat days selector (checkboxes for Mon-Sun), sound selector (V1: system default only), enable/disable toggle.
- **Alarm firing**: When an alarm triggers, a system notification appears: "Alarm: `<label>`" (or "Alarm" if no label). The alarm view opens automatically if the app is running.
- **Dismiss/Snooze**: Notification provides Dismiss and Snooze (5 min) buttons. Snooze re-fires the alarm after 5 minutes. Snooze is available up to 3 times per alarm trigger, then it auto-dismisses.
- **Alarm persistence**: Alarms are stored in cortex-files and persist across app restarts and system reboots. Enabled alarms fire even when the app is closed (via runtime notification scheduling).
- Maximum alarms: 20.

### 5.6 World Clock Tab

- **City list**: Scrollable list of added cities. Each entry shows: city name, current time, date, and time difference from local (e.g., "+5:30" or "-8:00").
- **Add city**: A "+" button opens a searchable list of available cities grouped by timezone. Search filters by city name or timezone abbreviation.
- **Remove city**: Swipe-to-delete or right-click "Remove" on a city entry.
- **Available cities**: A curated list of 100 major cities covering all standard timezones. The list is hardcoded, not fetched from a network service.
- **Time update**: City times update every minute. Daylight saving time adjustments are applied based on the timezone database embedded in the app.
- Maximum cities: 12.

### 5.7 Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+1` | Switch to Clock tab |
| `Ctrl+2` | Switch to Timer tab |
| `Ctrl+3` | Switch to Stopwatch tab |
| `Ctrl+4` | Switch to Alarm tab |
| `Ctrl+5` | Switch to World Clock tab |
| `Space` | Start/Stop/Resume (Timer or Stopwatch, context-dependent) |
| `L` | Record lap (Stopwatch tab, when running) |
| `R` | Reset (Timer or Stopwatch, context-dependent) |
| `Escape` | Dismiss alarm notification |

## 6. System Behavior

### 6.1 Time Source

- All time is sourced from the JavaScript `Date` object and `performance.now()` for high-precision stopwatch timing.
- The app does not perform NTP synchronization. It relies on the system clock.
- Timer and stopwatch use `setInterval` at 10 ms for centisecond precision. Clock display updates at 1-second intervals.

### 6.2 Alarm Scheduling

- Alarms are scheduled through the cortex-runtime notification system: `runtime.notifications.schedule(alarmTime, payload)`.
- The runtime delivers the notification at the scheduled time, even if the Clock app is not running.
- On alarm delivery, the runtime shows a system notification. If the Clock app is running, it also opens the Alarm tab and highlights the firing alarm.
- Repeat-day alarms are rescheduled for the next occurrence after each firing.
- One-time alarms (no repeat days) are automatically disabled after firing.

### 6.3 Timer Implementation

- Timers track the target completion timestamp (not remaining duration) to prevent drift from `setInterval` inaccuracies.
- On pause, the remaining duration is calculated and stored. On resume, a new target timestamp is computed.
- Timer alarm sound is played through the Web Audio API using a short tone (800 ms, two beeps).

### 6.4 Stopwatch Implementation

- The stopwatch tracks the start timestamp and accumulates paused durations to compute elapsed time accurately.
- Lap times are computed as the delta between the current total elapsed time and the previous lap's total elapsed time.
- Centisecond precision is maintained for display. Internal timing uses `performance.now()` for sub-millisecond accuracy.

### 6.5 World Clock Timezone Data

- Timezone data is embedded in the app as a static JSON structure mapping cities to IANA timezone identifiers (e.g., "America/New_York").
- The `Intl.DateTimeFormat` API with `timeZone` option is used to compute the current time in each timezone.
- Daylight saving time transitions are handled automatically by the JavaScript engine's ICU data.
- Time differences are computed as the offset between the local timezone and the target timezone at the current moment.

### 6.6 App Lifecycle

- Single-instance app. Launching a second instance brings the existing window to focus.
- Alarms are persisted to cortex-files on every change. On launch, the alarm list is loaded and enabled alarms are rescheduled with the runtime.
- Timers do not persist across sessions. If the app closes while a timer is running, it is lost.
- Stopwatch state does not persist across sessions.
- World clock city list persists across sessions (stored in cortex-files).
- Clock display format (12h/24h) persists across sessions.

## 7. Architecture

```
apps/clock-utils-app/
  manifest.json
  package.json
  src/
    main.ts                    # Registers app with runtime, schedules alarms
    App.tsx                    # Root component, tab navigation
    components/
      TabBar.tsx               # Five-tab navigation
      clock/
        ClockTab.tsx           # Clock tab container
        DigitalClock.tsx       # Digital time display
        AnalogClock.tsx        # SVG analog clock face
      timer/
        TimerTab.tsx           # Timer tab container
        TimerSetup.tsx         # Hours/minutes/seconds input
        TimerDisplay.tsx       # Countdown display with progress ring
        TimerCard.tsx          # Mini-card for concurrent timers
      stopwatch/
        StopwatchTab.tsx       # Stopwatch tab container
        StopwatchDisplay.tsx   # Elapsed time display
        LapList.tsx            # Scrollable lap entries
        LapEntry.tsx           # Single lap: number, lap time, total
      alarm/
        AlarmTab.tsx           # Alarm tab container
        AlarmList.tsx          # List of configured alarms
        AlarmEditor.tsx        # Add/edit alarm form
        AlarmNotification.tsx  # In-app alarm firing overlay
      world-clock/
        WorldClockTab.tsx      # World clock tab container
        CityList.tsx           # Added cities display
        CitySearch.tsx         # Searchable city picker
        CityEntry.tsx          # Single city: name, time, offset
    services/
      time-utils.ts            # Time formatting, duration calculations
      timer-engine.ts          # Countdown timer logic
      stopwatch-engine.ts      # Stopwatch logic with laps
      alarm-scheduler.ts       # Alarm CRUD and runtime scheduling
      alarm-storage.ts         # Persist alarms to cortex-files
      world-clock-storage.ts   # Persist city list to cortex-files
      timezone-data.ts         # Static city-to-timezone mapping
      sound-player.ts          # Web Audio API alarm tone
    hooks/
      useClock.ts              # Current time, 1-second tick
      useTimer.ts              # Timer state: multiple timers
      useStopwatch.ts          # Stopwatch state: elapsed, laps
      useAlarms.ts             # Alarm list CRUD and firing
      useWorldClock.ts         # City list and time updates
    types.ts
  tests/
    unit/
      time-utils.test.ts       # Formatting, duration math
      timer-engine.test.ts     # Start, pause, resume, reset, complete
      stopwatch-engine.test.ts # Start, stop, lap, reset, lap delta
      alarm-scheduler.test.ts  # Schedule, reschedule, repeat logic
      timezone-data.test.ts    # City mapping completeness
    integration/
      clock-display.test.ts    # Tab switching, time display
      timer-flow.test.ts       # Set, start, pause, resume, complete
      stopwatch-flow.test.ts   # Start, lap, stop, reset
      alarm-flow.test.ts       # Create, fire, dismiss, snooze, repeat
      world-clock-flow.test.ts # Add city, verify time display
      persistence.test.ts      # Alarms and cities persist across reload
```

No Rust backend crate needed. All logic is client-side TypeScript.

## 8. Data Model

### 8.1 Alarm

```typescript
interface Alarm {
  id: string;                       // UUID
  time: string;                     // "HH:MM" in 24-hour format
  label: string;                    // Max 50 chars, empty string if unset
  repeatDays: number[];             // 0=Sun, 1=Mon, ... 6=Sat. Empty = one-time
  enabled: boolean;
  sound: "default";                 // V1: only "default"
  lastFired: string | null;         // ISO 8601, prevents duplicate firing
}
```

### 8.2 Timer State

```typescript
interface TimerInstance {
  id: string;                       // UUID
  duration: number;                 // Total seconds
  targetTimestamp: number | null;   // null when not running
  remainingOnPause: number | null;  // Remaining seconds when paused
  state: "setup" | "running" | "paused" | "complete";
}
```

### 8.3 Stopwatch State

```typescript
interface StopwatchState {
  startTimestamp: number | null;    // performance.now() value
  accumulatedMs: number;            // Total ms from previous run segments
  laps: Lap[];
  running: boolean;
}

interface Lap {
  id: string;
  number: number;
  totalMs: number;                  // Total elapsed at lap time
  lapMs: number;                    // Delta from previous lap
}
```

### 8.4 World Clock City

```typescript
interface WorldClockCity {
  id: string;                       // City identifier from timezone-data
  city: string;                     // Display name
  timezone: string;                 // IANA timezone identifier
  order: number;                    // Display order
}
```

### 8.5 App State

```typescript
interface ClockAppState {
  activeTab: "clock" | "timer" | "stopwatch" | "alarm" | "world-clock";
  clockFormat: "12h" | "24h";
  timers: TimerInstance[];          // Max 4
  stopwatch: StopwatchState;
  alarms: Alarm[];                  // Max 20
  worldClockCities: WorldClockCity[]; // Max 12
}
```

### 8.6 Manifest

```typescript
{
  id: "com.cortexos.clock",
  name: "Clock",
  version: "1.0.0",
  description: "Clock, timer, stopwatch, alarms, and world clock",
  firstParty: true,
  bundled: true,
  entry: { frontend: "src/main.ts" },
  window: {
    defaultWidth: 480,
    defaultHeight: 560,
    minWidth: 380,
    minHeight: 450,
    resizable: true,
    singleInstance: true
  },
  permissions: {
    required: ["runtime.state", "runtime.lifecycle", "runtime.notifications", "files.read", "files.write"],
    optional: ["ai.context", "ai.invoke"]
  },
  ai: {
    surfaces: { assistantPanel: true, contextMenu: false, inlineSuggestions: false },
    contextProviders: ["clock-context"],
    actions: []
  },
  accessibility: { highContrastSupport: true, screenReaderSupport: true, keyboardNavigation: true },
  category: "utilities"
}
```

## 9. Public Interfaces

### 9.1 AI Context

```typescript
interface ClockAIContext {
  activeTab: string;
  localTime: string;
  alarmCount: number;
  timerCount: number;
  worldClockCities: string[];
}
```

### 9.2 Commands Exposed

None. Clock does not expose commands to other apps.

## 10. Internal Interfaces

### 10.1 Timer Engine

```typescript
interface TimerEngine {
  create(durationSec: number): TimerInstance;
  start(timer: TimerInstance): TimerInstance;
  pause(timer: TimerInstance): TimerInstance;
  resume(timer: TimerInstance): TimerInstance;
  reset(timer: TimerInstance): TimerInstance;
  getRemaining(timer: TimerInstance): number;  // Seconds remaining
  isComplete(timer: TimerInstance): boolean;
}
```

### 10.2 Stopwatch Engine

```typescript
interface StopwatchEngine {
  start(state: StopwatchState): StopwatchState;
  stop(state: StopwatchState): StopwatchState;
  reset(state: StopwatchState): StopwatchState;
  recordLap(state: StopwatchState): StopwatchState;
  getElapsedMs(state: StopwatchState): number;
  getLapCount(state: StopwatchState): number;
}
```

### 10.3 Alarm Scheduler

```typescript
interface AlarmScheduler {
  schedule(alarm: Alarm): void;
  cancel(alarmId: string): void;
  getNextOccurrence(alarm: Alarm): Date;
  shouldFire(alarm: Alarm, now: Date): boolean;
}
```

### 10.4 Alarm Storage

```typescript
interface AlarmStorage {
  loadAlarms(): Promise<Alarm[]>;
  saveAlarms(alarms: Alarm[]): Promise<void>;
}
```

### 10.5 Sound Player

```typescript
interface SoundPlayer {
  playAlarm(): void;
  playTimerComplete(): void;
  stop(): void;
}
```

## 11. State Management

- **Ephemeral**: Active tab, timer setup input values, alarm editor form state, city search query, stopwatch display formatting.
- **Session**: Running timers, running stopwatch, clock format (12h/24h). Persisted via cortex-runtime session state for hot-reload.
- **Persistent**: Alarms (stored at `/clock/alarms.json` in cortex-files), world clock city list (stored at `/clock/world-cities.json` in cortex-files), clock format preference (stored at `/clock/preferences.json`).
- State key: `com.cortexos.clock.session`.

## 12. Failure Modes and Error Handling

| Failure | Behavior |
|---------|----------|
| Runtime notifications unavailable | Alarms still fire when the app is open (in-app only). Show warning on first alarm creation: "Background alarms may not work. Alarms will only fire while Clock is open." |
| Sound playback failure | Show visual-only alarm notification (flashing display). Log as warning. |
| Alarm storage corruption | Load empty alarm list. Show toast: "Could not load alarms. Your alarms may need to be reconfigured." |
| World clock city storage corruption | Load default cities (local timezone only). Show toast: "Could not load world clock cities." |
| Timer overrun (missed completion) | Check target timestamp on each tick. If target is in the past, fire immediately. Never skip alarm. |
| Max alarms reached (20) | Show toast: "Maximum 20 alarms. Remove an alarm to add a new one." |
| Max timers reached (4) | Show toast: "Maximum 4 concurrent timers." Disable the "+" button. |
| Max laps reached (100) | Show toast: "Maximum 100 laps recorded." Lap button is disabled. |
| Max world clock cities (12) | Show toast: "Maximum 12 cities. Remove a city to add a new one." |

All errors are non-blocking. The app remains fully functional after any error.

## 13. Security and Permissions

- `runtime.notifications` is required for background alarm delivery.
- `files.read` and `files.write` are required for persisting alarms and preferences.
- No network access needed. All timezone data is embedded.
- No user data is transmitted externally.
- Alarm sounds are generated via Web Audio API (no external audio files).

## 14. Performance Requirements

- Clock face render and update: under 16 ms per frame (smooth second hand animation).
- Timer tick update: under 5 ms.
- Stopwatch centisecond update: under 5 ms.
- Alarm list render: under 16 ms for 20 alarms.
- World clock time computation: under 10 ms for 12 cities.
- Alarm firing latency: under 500 ms from scheduled time to notification display.
- Startup first meaningful paint: under 300 ms.
- Memory: negligible. No large data structures. Timer and stopwatch state under 10 KB.
- Bundle size: under 150 KB gzipped (including timezone city data).

## 15. Accessibility Requirements

- Tab buttons have `role="tab"` with `aria-selected` state.
- Clock digital display has `role="timer"` with `aria-live="polite"` announcing time each minute (not each second to avoid verbosity).
- Analog clock face has `aria-label` with current time as text.
- Timer remaining display has `role="timer"` with `aria-live="polite"`.
- Stopwatch display has `role="timer"`.
- Alarm list items have `role="listitem"` with time, label, and repeat days as accessible text.
- Alarm toggle has `aria-pressed` state.
- Lap entries have `role="listitem"` with lap number, lap time, and total time.
- World clock entries have `role="listitem"` with city name and current time.
- Keyboard navigation: Tab between tabs, arrow keys within tab content. Space for start/stop.
- Focus is visible on all interactive elements.

## 16. Observability and Logging

Logged events:
- `clock.launched` (info) -- App opened. Payload: `{ tab: string }`.
- `clock.tab.switched` (info) -- Tab changed. Payload: `{ tab: string }`.
- `clock.timer.started` (info) -- Timer started. Payload: `{ durationBucket: string }`. No exact duration.
- `clock.timer.complete` (info) -- Timer completed.
- `clock.stopwatch.lap` (debug) -- Lap recorded.
- `clock.alarm.created` (info) -- Alarm created.
- `clock.alarm.fired` (info) -- Alarm fired. Payload: `{ repeat: boolean }`.
- `clock.alarm.snoozed` (info) -- Alarm snoozed.
- `clock.alarm.dismissed` (info) -- Alarm dismissed.
- `clock.world-clock.city-added` (info) -- City added. Payload: `{ timezone: string }`. No city name.
- `clock.error` (warn) -- Alarm scheduling or storage failure. Payload: `{ errorType: string }`.

No PII is logged. Alarm labels, specific times, and city names are never included in log payloads.

## 17. Testing Requirements

### 17.1 Unit Tests

- Time utils: format 12h/24h, duration formatting (HH:MM:SS), centisecond formatting.
- Timer engine: create, start, pause, resume, reset, complete detection, remaining calculation, multiple concurrent timers.
- Stopwatch engine: start, stop, resume, reset, lap recording, lap delta calculation, max 100 laps pruning.
- Alarm scheduler: next occurrence calculation for one-time and repeat-day alarms, should-fire logic, last-fired deduplication.
- Timezone data: all 100 cities have valid IANA identifiers, no duplicate cities.

### 17.2 Integration Tests

- Clock tab: verify digital display updates, 12h/24h toggle persists.
- Timer flow: set 5-second timer, start, verify completion alarm fires.
- Multiple timers: start 2 timers, verify independent operation.
- Stopwatch flow: start, record 3 laps, stop, verify lap times and total, reset.
- Alarm flow: create alarm 10 seconds in future, verify notification fires, dismiss. Create repeat-day alarm, verify rescheduling.
- Snooze flow: create alarm, fire, snooze, verify re-fire after 5 minutes.
- World clock flow: add city, verify time display, verify offset calculation, remove city.
- Persistence: create alarms, reload app, verify alarms loaded and rescheduled.

### 17.3 Accessibility Tests

- AX tree validation for all five tabs.
- Keyboard navigation: switch tabs, start/stop timer, record lap.
- Screen reader announcement of time changes.

## 18. Acceptance Criteria

- [ ] Digital clock displays current time, updates every second.
- [ ] Analog clock face shows correct time with hour, minute, second hands.
- [ ] 12h/24h toggle works and persists across sessions.
- [ ] Timer counts down correctly from set duration.
- [ ] Timer alarm sounds and shows notification on completion.
- [ ] Multiple concurrent timers (up to 4) operate independently.
- [ ] Stopwatch measures elapsed time with centisecond precision.
- [ ] Lap recording shows lap time (delta) and total elapsed time.
- [ ] Best/worst lap highlighting works with 3+ laps.
- [ ] Alarm fires at the correct time with notification.
- [ ] Alarm snooze re-fires after 5 minutes (max 3 snoozes).
- [ ] Repeat-day alarms reschedule for the next occurrence.
- [ ] Alarms persist across app restart and are rescheduled on launch.
- [ ] Alarm fires even when app is closed (via runtime notifications).
- [ ] World clock displays correct current time for added cities.
- [ ] Time offset from local is correctly calculated.
- [ ] World clock city list persists across sessions.
- [ ] Tab switching preserves state (running timer/stopwatch continue).
- [ ] All keyboard shortcuts work as documented.
- [ ] App launches in under 300 ms.
- [ ] All three themes render correctly.
- [ ] Screen reader announces time and alarm events.
- [ ] Unit test coverage >= 80%.

## 19. Build Order and Dependencies
**Layer 11**. Depends on: 09 (app runtime), 16 (theme tokens), 17 (first-party app parent)

### Prerequisites

- Spec 17 parent (first-party app framework).
- `@cortexos/ui-components` (shared UI library).
- `@cortexos/runtime-client` (for state persistence, lifecycle, and notification scheduling).
- `@cortexos/files-client` (for alarm and preference persistence).
- `@cortexos/ai-client` (for AI surface).
- `@cortexos/theme` (design token consumer).

### Build Position

Clock and Utility is the **first** first-party app to build (alongside File Manager as a P0 pair). It is the simplest app that validates runtime notification scheduling and preference persistence, serving as a baseline for the app framework.

No Rust crate needed. Pure frontend app.

## 20. Non-Goals and Anti-Patterns

### Non-Goals

- Calendar or date picker.
- Custom alarm sounds (beyond system default).
- Desktop clock widgets.
- Pomodoro or productivity timer modes.
- Sleep tracking or bedtime features.
- NTP time synchronization.
- Sunrise/sunset data.

### Anti-Patterns

- Using `setInterval` alone for timer accuracy (must track target timestamp to prevent drift).
- Storing alarm times in local timezone without timezone awareness (use UTC internally for scheduling, display in local time).
- Playing audio files from external sources (generate tones via Web Audio API).
- Blocking the main thread with synchronous time calculations.
- Hardcoding timezone offsets instead of using `Intl.DateTimeFormat` with IANA identifiers (DST would break).

## 21. Implementation Instructions for Claude Code / Codex

### Subsystem Ownership

- Clock app owns: time display, timer engine, stopwatch engine, alarm scheduling, world clock logic, alarm storage, sound generation.
- Clock app does not own: notification delivery (delegates to cortex-runtime), file I/O (delegates to cortex-files), window management.

### Recommended Implementation Order

1. Create `manifest.json` and validate against FirstPartyAppManifest schema.
2. Implement `services/time-utils.ts` -- time formatting, duration calculations. Write unit tests.
3. Implement `services/timer-engine.ts` -- countdown timer with target-timestamp tracking. Write comprehensive unit tests.
4. Implement `services/stopwatch-engine.ts` -- elapsed time with laps. Write comprehensive unit tests.
5. Implement `services/timezone-data.ts` -- static city-to-timezone mapping (100 cities). Write unit tests verifying all entries.
6. Implement `services/alarm-scheduler.ts` -- next occurrence calculation, repeat-day logic. Write unit tests.
7. Implement `services/sound-player.ts` -- Web Audio API tone generation.
8. Implement `components/TabBar.tsx` with five-tab navigation.
9. Implement Clock tab: `DigitalClock.tsx`, `AnalogClock.tsx`.
10. Implement Timer tab: `TimerSetup.tsx`, `TimerDisplay.tsx`, `TimerCard.tsx`.
11. Implement Stopwatch tab: `StopwatchDisplay.tsx`, `LapList.tsx`, `LapEntry.tsx`.
12. Implement Alarm tab: `AlarmList.tsx`, `AlarmEditor.tsx`, `AlarmNotification.tsx`.
13. Implement World Clock tab: `CityList.tsx`, `CitySearch.tsx`, `CityEntry.tsx`.
14. Implement `services/alarm-storage.ts` and `services/world-clock-storage.ts` for persistence.
15. Wire up `App.tsx` connecting all tabs, engines, and storage.
16. Integrate `runtime.notifications` for background alarm delivery.
17. Integrate `@cortexos/ai-client` for AI surface.
18. Accessibility audit and fixes.
19. Theme verification (light, dark, high-contrast).

### What Can Be Stubbed Initially

- AI context provider can return minimal data initially.
- Analog clock face can be a placeholder SVG initially, refined for visual polish later.
- Sound player can use a simple beep initially, refined for pleasant tone later.
- City search can be a simple text filter initially.

### What Must Be Real in V1

- Accurate digital clock with 12h/24h toggle.
- Analog clock with correct hand positions.
- Timer with accurate countdown and completion alarm.
- Multiple concurrent timers (up to 4).
- Stopwatch with centisecond precision and lap tracking.
- Alarm creation, firing, dismiss, snooze (5 min, max 3).
- Repeat-day alarms with correct rescheduling.
- Background alarm delivery via runtime notifications.
- Alarm persistence across app restarts.
- World clock with 100-city database and correct time computation.
- Tab-based UI with state preservation.
- All keyboard shortcuts.
- Theme support.
- Accessibility (keyboard navigation, screen reader).

### What Cannot Be Inferred

- Default clock format (12h, matching system locale).
- Default timer duration (5 minutes).
- Snooze duration (5 minutes).
- Max snooze count (3 per alarm trigger).
- Alarm sound specification (two 800 ms beeps at 440 Hz, 200 ms gap).
- Number of available cities (100).
- Default window size (480x560 per manifest).
- Update interval for clock (1 second), timer (10 ms), stopwatch (10 ms).

### Stop Conditions

1. All unit tests pass with >= 80% coverage.
2. Timer engine tests cover start, pause, resume, reset, and completion detection.
3. Stopwatch engine tests cover lap delta calculation and max 100 laps.
4. Alarm scheduler tests cover repeat-day logic and next occurrence calculation.
5. Integration tests for alarm persistence across reload pass.
6. Background alarm fires when app is closed (runtime notification integration).
7. All five tabs render correctly and switch without losing state.
8. All three themes render correctly.
9. Stopwatch shows centisecond precision.
10. World clock shows correct times for all 12 added cities.

### Testing Gates

- Timer and stopwatch engine unit tests must pass before UI work begins.
- Alarm scheduler unit tests must pass before alarm UI work begins.
- Timezone data completeness test must pass before world clock UI work begins.
- Alarm persistence integration test must pass before merge.
- Background alarm integration test must pass before merge.
- Accessibility tests must pass before merge.
