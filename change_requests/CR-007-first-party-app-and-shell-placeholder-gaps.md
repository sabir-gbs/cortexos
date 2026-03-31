# CR-007: Desktop Shell And First-Party Apps Still Contain Placeholder Or Local-Only Core Behavior

Severity: High

## Problem

Several user-facing apps and shell paths still use placeholders, hardcoded data, or browser-local persistence in places where the specs require service-backed behavior.

## Evidence

Desktop shell placeholder content:

- [apps/desktop-shell/src/components/WindowFrame.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/components/WindowFrame.tsx): renders `App content renders here`
- [apps/desktop-shell/src/App.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/App.tsx): contains `Notifications placeholder`

File Manager uses hardcoded mock filesystem data instead of service-backed filesystem state:

- [apps/file-manager-app/src/App.tsx](/home/sabir/projects/cortexos/apps/file-manager-app/src/App.tsx)

Text Editor persists content and filename in browser `localStorage` instead of the virtual filesystem/service path:

- [apps/text-editor-app/src/App.tsx](/home/sabir/projects/cortexos/apps/text-editor-app/src/App.tsx)

These conflict with the project DoD and subsystem specs that require core user flows to be real, not placeholder-backed.

Relevant contract docs:

- [docs/specs/07_desktop_shell.md](/home/sabir/projects/cortexos/docs/specs/07_desktop_shell.md)
- [docs/specs/11_virtual_filesystem_and_storage_abstraction.md](/home/sabir/projects/cortexos/docs/specs/11_virtual_filesystem_and_storage_abstraction.md)
- [docs/specs/17b_text_editor_app.md](/home/sabir/projects/cortexos/docs/specs/17b_text_editor_app.md)
- [docs/specs/17d_file_manager_app.md](/home/sabir/projects/cortexos/docs/specs/17d_file_manager_app.md)
- [docs/specs/appendix_c_definition_of_done.md](/home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md)

## Expected Contract

Core app logic and shell behavior should not remain placeholder-backed once the implementation is claimed complete.

## Requested Change

- replace hardcoded/mock core flows with service-backed behavior
- explicitly identify which visible placeholders are allowed only for non-core polish vs which are release-blocking
- update docs if any shipped v1 behavior is intentionally reduced compared with the spec

## Verification

- shell windows render real app content, not a global placeholder frame
- notifications area is backed by the actual notifications service
- text editor save/open flows go through the filesystem contract
- file manager reads actual filesystem state rather than embedded demo data

## Affected Files

- [apps/desktop-shell/src/components/WindowFrame.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/components/WindowFrame.tsx)
- [apps/desktop-shell/src/App.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/App.tsx)
- [apps/file-manager-app/src/App.tsx](/home/sabir/projects/cortexos/apps/file-manager-app/src/App.tsx)
- [apps/text-editor-app/src/App.tsx](/home/sabir/projects/cortexos/apps/text-editor-app/src/App.tsx)
