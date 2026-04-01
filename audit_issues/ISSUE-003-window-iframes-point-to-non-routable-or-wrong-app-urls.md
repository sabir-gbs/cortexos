# ISSUE-003: Window Iframes Point To Non-Routable Or Wrong App URLs

Severity: Critical

## What Happens

Even if the launch and window-creation path is repaired, the shell’s app rendering path is still broken because window iframes point to URLs that do not resolve to the intended app.

## Evidence

Shell manifest table:

- [types.ts](/home/sabir/projects/cortexos/apps/desktop-shell/src/types.ts)
  - uses entry points like:
  - `/apps/file-manager/index.html`
  - `/apps/terminal-lite/index.html`
  - `/apps/settings-app/index.html`

Window rendering path:

- [WindowFrame.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/components/WindowFrame.tsx)
  - renders `<iframe src={entryPoint} ... />`

Live HTTP checks:

- `GET http://localhost:5173/apps/file-manager/index.html` returns the desktop shell HTML, not the File Manager app
- `GET http://localhost:5173/apps/settings-app/index.html` returns the desktop shell HTML, not the Settings app

Also, the real app package folders are named differently:

- `apps/file-manager-app/`
- `apps/terminal-lite-app/`
- `apps/calculator-app/`

So the shell’s builtin app URL map is not aligned with the actual served paths.

## Why It Matters

Fixing the 422 alone will not make apps render correctly. The next layer is broken too: opened windows would iframe the wrong page.

## Recommended Fix

- define a real served app URL contract for first-party apps
- make `BUILTIN_APPS.entry_point` match that served contract exactly
- avoid hardcoding synthetic app URLs in the shell unless they are guaranteed to exist in dev and production
- add an end-to-end check that a launched first-party app window loads app-specific content rather than the shell index page

