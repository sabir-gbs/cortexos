# ISSUE-006: File Manager uses an absolute root path that the Files API rejects

Severity: High

Status: Open

## Summary

The blank-window issue is no longer the main blocker for `Files`. The `Files` app now renders inside its window, but its initial directory fetch fails because the frontend requests `GET /api/v1/files/list?path=/` while the backend validates virtual filesystem paths as relative-only and rejects `/` as invalid.

This is a separate contract issue from the iframe/static-asset problem.

## Live Evidence

At `http://localhost:5173` after login with `admin / uglyducks22!`:

- the `Files` window opens
- the iframe renders the app shell
- the app body shows:
  - `Error: Failed to load directory: 400 Bad Request`

Direct HTTP reproduction against the live backend:

```text
GET /api/v1/files/list?path=/
HTTP/1.1 400 Bad Request
{"error":{"code":"VAL_001","category":"validation","message":"invalid path: path violation: path must be relative, not absolute","details":null,"retryable":false}}
```

## Code Evidence

Frontend request:

- [App.tsx](/home/sabir/projects/cortexos/apps/file-manager-app/src/App.tsx)
  - initializes `currentPath` as `/`
  - calls `fetch(`${API_BASE}/api/v1/files/list?${params}`)` with `path=/`

Backend validation:

- [files.rs](/home/sabir/projects/cortexos/crates/cortex-api/src/routes/files.rs)
  - `VirtualPath::new(path)` rejects invalid paths before listing

The backend error text confirms the contract:

- `path must be relative, not absolute`

## Why This Matters

The desktop launch path can now open and render app windows, but `Files` still looks broken to the user because its first data request fails immediately.

This also indicates a contract mismatch between the file-manager frontend and the platform filesystem API:

- frontend mental model: root is `/`
- backend contract: root must be represented as a relative virtual path

## Recommended Fix

Choose one contract and make it consistent everywhere.

Preferred direction:

1. keep the backend virtual path contract authoritative
2. normalize the file-manager frontend so root is represented using the backend’s accepted root form
3. add regression coverage for the root-directory case

Likely fix points:

- [App.tsx](/home/sabir/projects/cortexos/apps/file-manager-app/src/App.tsx)
  - normalize root path handling before building request query params
  - ensure breadcrumb/UI display can still show `Root` while the API receives a valid relative root value

Potential backend fallback, only if explicitly desired:

- allow `/` as an alias for root and normalize it server-side before `VirtualPath::new`

That is less strict and may be acceptable, but only if it is aligned with the owning file-system spec and used consistently across first-party apps.

## Required Verification

After the fix:

1. log in to the live desktop shell
2. open `Files`
3. confirm the root directory loads without a `400`
4. navigate into at least one folder and back
5. add or update tests covering the root-path contract

