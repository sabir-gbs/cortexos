# Spec 03: Identity, Authentication, Sessions, and User Profiles

**Status:** Implementation-grade
**Owner crate:** `cortex-auth`
**Depends on:** `cortex-core`, `cortex-db`, `cortex-policy`, `cortex-settings`
**Required by:** `cortex-api`, `cortex-runtime`, `apps/settings-app`, `apps/desktop-shell`

---

## 1. Purpose

Define the user identity model, authentication mechanisms, session lifecycle, and user profile management for CortexOS. This subsystem establishes who the user is, proves their identity, maintains their authenticated session, and stores their profile information. All other subsystems that require user context depend on `cortex-auth` as the source of truth for identity.

CortexOS is a single-user browser OS by default. Authentication exists to protect the user's session from unauthorized access on shared or physical devices, and to establish an identity context for permissions, settings, and AI preferences. Multi-user support is an optional extension enabled through SSO integration, not a v1 requirement.

---

## 2. Scope

**In scope for this spec:**

- User identity model: unique user ID, username, display name, avatar reference
- Local authentication: password-based authentication for the default single-user deployment
- Optional SSO: OAuth 2.0 / OIDC integration for multi-user or enterprise deployments
- Session management: token-based sessions with server-side session store
- User profiles: mutable profile data (display name, avatar, preferences)
- Credential storage: secure password hashing, credential validation
- Session lifecycle: creation, validation, refresh, expiration, destruction
- Logout and session invalidation
- Account lockout after repeated failed authentication attempts
- Integration hooks for `cortex-policy` (permission checks require user context)
- Integration hooks for `cortex-settings` (per-user settings resolution)

**Owned by this subsystem:**

- User record data model
- Credential storage and validation
- Session token generation, validation, and revocation
- Profile read/write operations
- Authentication API endpoints
- SSO adapter interface

---

## 3. Out of Scope

- **Authorization and permissions:** Owned by `cortex-policy` (spec 04). `cortex-auth` establishes identity; `cortex-policy` determines what that identity is allowed to do.
- **Settings storage:** Owned by `cortex-settings` (spec 05). User preferences that affect system behavior are stored there, not in the profile.
- **AI provider credentials:** Owned by `cortex-ai` (spec 06). API keys for LLM providers are not user authentication credentials.
- **Multi-tenancy isolation:** CortexOS is single-user by default. Namespace isolation for multiple concurrent users is not in v1.
- **Biometric authentication:** Not in v1. The interface is designed so biometric auth can be added as an additional factor later without changing the session model.
- **Password recovery via email:** Not in v1. The local-only mode has no email infrastructure. Recovery uses a local reset mechanism.
- **User registration flows for external users:** Not applicable. The OS creates the initial user during first-boot setup.

---

## 4. Objectives

1. Establish a stable, unique identity for every CortexOS user that persists across sessions.
2. Provide local password-based authentication as the default mechanism with no external dependencies.
3. Support optional SSO (OAuth 2.0 / OIDC) for environments that require centralized identity.
4. Maintain server-side session state with tamper-resistant tokens.
5. Enforce session expiration and renewal to limit the window of session hijacking.
6. Provide a user profile that stores non-security-critical identity data (display name, avatar).
7. Expose a clean internal interface so all other subsystems can resolve the current user without direct knowledge of authentication mechanisms.
8. Log all authentication events (login, logout, failed attempts) for audit purposes.

---

## 5. User-Visible Behavior

### 5.1 First Boot

On first boot, CortexOS presents a setup screen requiring the user to choose:
- A username (alphanumeric, 3-32 characters)
- A display name (1-64 characters, any Unicode)
- A password (minimum 8 characters)

This creates the initial user record and immediately establishes an authenticated session. There is no "guest" mode.

### 5.2 Login Screen

On subsequent boots, the user sees a login screen with:
- Username field (pre-filled if only one user exists)
- Password field
- Login button
- SSO login button (visible only if SSO is configured in system settings)

Failed login attempts display: "Invalid username or password." The message is deliberately identical for wrong username and wrong password to prevent username enumeration.

After 5 consecutive failed attempts for the same username, the account is locked for 60 seconds. The lockout duration doubles on each subsequent lockout: 60s, 120s, 240s, up to a maximum of 900 seconds (15 minutes). The counter resets to zero on a successful login.

### 5.3 Session Duration

A session remains active until:
- The user explicitly logs out
- The session token expires (default: 24 hours of inactivity, configurable via `system.session_idle_timeout_minutes`)
- The server is restarted and session tokens are invalidated (configurable: persistent vs volatile sessions)

When a session expires, the user is returned to the login screen. No unsaved data is silently lost; the desktop shell is responsible for prompting about unsaved state before logout.

### 5.4 Profile Management

The user can access their profile from the Settings app under the "Account" section:
- Change display name
- Change avatar (upload an image, stored in the virtual filesystem at `vfs://system/avatars/{user_id}/{timestamp}.png`)
- Change password (requires current password confirmation)
- View username (read-only after creation)

### 5.5 SSO Login (when configured)

When SSO is enabled, the login screen shows an additional "Sign in with [Provider]" button. Clicking it opens a popup/redirect to the OAuth 2.0 authorization endpoint. On success, CortexOS receives an ID token, validates it, and either:
- Maps the SSO identity to an existing local user (by email or subject claim)
- Creates a new local user record linked to the SSO identity

The SSO flow is optional. When not configured, no SSO UI is shown.

---

## 6. System Behavior

### 6.1 Authentication Flow (Local)

```
1. Client submits { username, password } to POST /api/v1/auth/login
2. cortex-auth looks up user by username (case-sensitive)
3. If user not found: return 401 Unauthorized with generic error message
4. If user found: verify password against stored Argon2id hash
5. If password incorrect: increment failed_attempt_counter, return 401
6. If account is locked: return 429 Too Many Requests with retry-after hint
7. On success: reset failed_attempt_counter to 0
8. Generate session token (cryptographically random, 256-bit, base64url-encoded)
9. Create session record in server-side store with: user_id, created_at, last_active_at, expires_at, client_info
10. Set HTTP-only, Secure, SameSite=Strict cookie: "cortex_session=<token>"
11. Return 200 with user profile data
```

### 6.2 Session Validation (on every authenticated request)

```
1. Extract session token from cookie
2. If no token present: return 401
3. Look up session in server-side store by token
4. If session not found: return 401, clear cookie
5. If session.expires_at < now: delete session, return 401, clear cookie
6. If session.last_active_at + idle_timeout < now: delete session, return 401, clear cookie
7. Update session.last_active_at = now
8. Attach user_id to request context
9. Continue processing request
```

### 6.3 Token Refresh

Sessions are not refreshed via a separate endpoint. The session remains valid as long as the user is active. Each authenticated request updates `last_active_at`. If the idle timeout elapses, the session expires naturally.

The maximum absolute session lifetime is 7 days (configurable via `system.session_max_lifetime_hours`). When `created_at + max_lifetime < now`, the session is terminated regardless of activity.

### 6.4 Logout

```
1. Client calls POST /api/v1/auth/logout
2. cortex-auth deletes the session record from the server-side store
3. The response clears the session cookie
4. The client redirects to the login screen
```

### 6.5 SSO Flow

```
1. Client calls GET /api/v1/auth/sso/redirect?provider=<provider_id>
2. cortex-auth generates a PKCE code_verifier and code_challenge
3. cortex-auth stores code_verifier in a temporary state record keyed by a state parameter
4. Returns 302 redirect to the OAuth 2.0 authorization URL with code_challenge
5. After user authenticates at the provider, provider redirects to callback URL
6. Client calls POST /api/v1/auth/sso/callback with { state, code }
7. cortex-auth retrieves stored code_verifier by state
8. cortex-auth exchanges code + code_verifier for tokens at the provider's token endpoint
9. cortex-auth validates the ID token (signature, issuer, audience, expiry)
10. cortex-auth extracts subject claim and email from ID token
11. cortex-auth looks up existing user by sso_subject or creates a new user record
12. Proceed with session creation (steps 8-11 of local auth flow)
```

### 6.6 Password Change

```
1. Client calls PUT /api/v1/auth/password with { current_password, new_password }
2. cortex-auth validates current_password against stored hash
3. If invalid: return 401
4. If valid: hash new_password with Argon2id, store new hash
5. Do NOT invalidate existing sessions (user already proved identity)
6. Return 200
```

---

## 7. Architecture

### 7.1 Crate Layout

```
cortex-auth/
  src/
    lib.rs           -- public re-exports
    identity.rs      -- User model, user ID generation
    credentials.rs   -- password hashing, verification
    session.rs       -- session token generation, validation, store
    profile.rs       -- profile read/write operations
    sso/
      mod.rs         -- SSO trait and dispatcher
      oauth2.rs      -- OAuth 2.0 / OIDC adapter
    api/
      mod.rs         -- auth API route handlers
      handlers.rs    -- login, logout, session validation middleware
    error.rs         -- auth-specific error types
```

### 7.2 Dependency Direction

```
cortex-core (types, error macros)
    |
cortex-db (storage interface)
    |
cortex-auth
    |
    +---> cortex-policy (user context for permission checks, read-only)
    +---> cortex-settings (read SSO config, session timeouts)
```

`cortex-auth` depends on `cortex-core`, `cortex-db`, `cortex-settings`. It does not depend on `cortex-ai`, `cortex-files`, or `cortex-runtime`.

### 7.3 Server-Side Session Store

Sessions are stored server-side in `cortex-db`. The store is a key-value table keyed by session token. The store must support:
- Insert (create session)
- Get by token (validate session)
- Delete by token (logout)
- Delete all sessions for a user (global logout)
- Delete expired sessions (periodic cleanup)

The session store runs a background cleanup task every 60 seconds that deletes all sessions where `expires_at < now`.

---

## 8. Data Model

### 8.1 User Record

```rust
struct User {
    /// Unique, immutable user identifier. Generated as UUIDv7 at creation.
    id: UserId,

    /// Unique, case-sensitive username. Immutable after creation.
    /// Constraints: 3-32 characters, ASCII alphanumeric + underscore.
    username: String,

    /// Display name shown in UI. Mutable by the user.
    /// Constraints: 1-64 characters, any Unicode.
    display_name: String,

    /// Reference to avatar file in virtual filesystem.
    /// Format: "vfs://system/avatars/{user_id}/{filename}" or None (default avatar).
    avatar_path: Option<String>,

    /// Hashed password using Argon2id.
    /// Present for locally-authenticated users.
    /// None for SSO-only users.
    password_hash: Option<PasswordHash>,

    /// Timestamp of user creation (UTC).
    created_at: DateTime<Utc>,

    /// Timestamp of last profile modification (UTC).
    updated_at: DateTime<Utc>,

    /// SSO linkage, if any.
    sso_link: Option<SsoLink>,

    /// Number of consecutive failed login attempts.
    failed_login_attempts: u32,

    /// If locked, the time at which the lockout expires.
    locked_until: Option<DateTime<Utc>>,
}

struct UserId(String); // UUIDv7, validated at construction

struct PasswordHash(String); // Argon2id encoded string

struct SsoLink {
    provider_id: String,   // matches a provider in cortex-settings SSO config
    subject: String,       // unique ID from the SSO provider
    email: Option<String>, // email claim from ID token
    linked_at: DateTime<Utc>,
}
```

### 8.2 Session Record

```rust
struct Session {
    /// The session token. 256-bit cryptographically random value, base64url-encoded.
    /// This is the primary key.
    token: SessionToken,

    /// The user this session belongs to.
    user_id: UserId,

    /// When the session was created.
    created_at: DateTime<Utc>,

    /// When the session was last active (updated on each authenticated request).
    last_active_at: DateTime<Utc>,

    /// When the session expires (created_at + max_lifetime).
    expires_at: DateTime<Utc>,

    /// Client information captured at session creation for audit.
    client_info: ClientInfo,
}

struct SessionToken(String); // base64url-encoded 256-bit random value

struct ClientInfo {
    user_agent: String,
    ip_address: Option<String>, // available in server context, None in pure-local mode
}
```

### 8.3 Invariants

1. `User.id` is immutable and unique across all users.
2. `User.username` is unique and immutable after creation.
3. `User.password_hash` is always Argon2id. No other hash algorithm is supported.
4. `Session.token` is unique across all active sessions.
5. A user may have multiple active sessions (e.g., multiple browser tabs).
6. `Session.expires_at` is always `created_at + max_lifetime`.
7. `Session.last_active_at` is never greater than the current time.
8. `User.failed_login_attempts` resets to 0 on successful login.
9. `User.locked_until` is `None` when `failed_login_attempts < 5`.
10. SSO-linked users with no password_hash can only authenticate via SSO.

---

## 9. Public Interfaces

### 9.1 HTTP API Endpoints

| Method | Path | Auth Required | Description |
|--------|------|---------------|-------------|
| POST | `/api/v1/auth/login` | No | Authenticate with username + password |
| POST | `/api/v1/auth/logout` | Yes | Terminate current session |
| POST | `/api/v1/auth/logout-all` | Yes | Terminate all sessions for current user |
| GET | `/api/v1/auth/session` | Yes | Get current session info |
| GET | `/api/v1/auth/sso/redirect` | No | Initiate SSO flow |
| POST | `/api/v1/auth/sso/callback` | No | Complete SSO flow |
| GET | `/api/v1/auth/profile` | Yes | Get current user profile |
| PUT | `/api/v1/auth/profile` | Yes | Update display name or avatar |
| PUT | `/api/v1/auth/password` | Yes | Change password |

### 9.2 Request/Response Types

```typescript
// POST /api/v1/auth/login
interface LoginRequest {
  username: string;
  password: string;
}
interface LoginResponse {
  user: UserProfile;
  session_expires_at: string; // ISO 8601
}

// GET /api/v1/auth/profile
interface UserProfile {
  id: string;
  username: string;
  display_name: string;
  avatar_url: string | null;  // resolved URL, not internal path
}

// PUT /api/v1/auth/profile
interface UpdateProfileRequest {
  display_name?: string;
  avatar_file_id?: string; // vfs file ID of uploaded avatar
}

// PUT /api/v1/auth/password
interface ChangePasswordRequest {
  current_password: string;
  new_password: string;
}

// GET /api/v1/auth/session
interface SessionInfo {
  user_id: string;
  created_at: string;
  expires_at: string;
  last_active_at: string;
}
```

### 9.3 Rust Public API

```rust
// cortex-auth public API (used by other crates)
pub trait AuthProvider: Send + Sync {
    /// Validate credentials and create a session.
    fn login(&self, username: &str, password: &str) -> Result<Session, AuthError>;

    /// Validate a session token and return the associated user ID.
    fn validate_session(&self, token: &SessionToken) -> Result<UserId, AuthError>;

    /// Terminate a specific session.
    fn logout(&self, token: &SessionToken) -> Result<(), AuthError>;

    /// Terminate all sessions for a user.
    fn logout_all(&self, user_id: &UserId) -> Result<(), AuthError>;

    /// Get the user profile.
    fn get_profile(&self, user_id: &UserId) -> Result<UserProfile, AuthError>;

    /// Update the user profile.
    fn update_profile(&self, user_id: &UserId, update: ProfileUpdate) -> Result<UserProfile, AuthError>;

    /// Change password (requires current password verification).
    fn change_password(&self, user_id: &UserId, current: &str, new: &str) -> Result<(), AuthError>;

    /// Create the initial user during first-boot setup.
    fn create_initial_user(&self, username: &str, display_name: &str, password: &str) -> Result<User, AuthError>;
}
```

---

## 10. Internal Interfaces

### 10.1 Session Store Trait

```rust
/// Internal session storage abstraction. Implemented by cortex-db backends.
pub trait SessionStore: Send + Sync {
    fn insert(&self, session: Session) -> Result<(), AuthError>;
    fn get(&self, token: &SessionToken) -> Result<Option<Session>, AuthError>;
    fn delete(&self, token: &SessionToken) -> Result<(), AuthError>;
    fn delete_all_for_user(&self, user_id: &UserId) -> Result<(), AuthError>;
    fn delete_expired(&self) -> Result<usize, AuthError>;
    fn update_last_active(&self, token: &SessionToken, now: DateTime<Utc>) -> Result<(), AuthError>;
}
```

### 10.2 User Store Trait

```rust
/// Internal user record storage. Implemented by cortex-db backends.
pub trait UserStore: Send + Sync {
    fn create(&self, user: User) -> Result<User, AuthError>;
    fn get_by_id(&self, id: &UserId) -> Result<Option<User>, AuthError>;
    fn get_by_username(&self, username: &str) -> Result<Option<User>, AuthError>;
    fn update(&self, user: &User) -> Result<User, AuthError>;
    fn get_by_sso_subject(&self, provider_id: &str, subject: &str) -> Result<Option<User>, AuthError>;
}
```

### 10.3 Credential Hasher

```rust
/// Password hashing and verification. Uses Argon2id.
pub struct CredentialHasher;

impl CredentialHasher {
    /// Hash a plaintext password with Argon2id.
    /// Parameters: m=19456, t=2, p=1 (OWASP recommended).
    pub fn hash(password: &str) -> Result<PasswordHash, AuthError>;

    /// Verify a plaintext password against a stored hash.
    pub fn verify(password: &str, hash: &PasswordHash) -> Result<bool, AuthError>;
}
```

### 10.4 SSO Provider Interface

```rust
/// Trait for SSO provider adapters. Implementations handle provider-specific logic.
pub trait SsoProviderAdapter: Send + Sync {
    /// Return the authorization URL for the OAuth 2.0 flow.
    fn authorization_url(&self, state: &str, code_challenge: &str) -> String;

    /// Exchange authorization code for tokens.
    fn exchange_code(&self, code: &str, code_verifier: &str) -> Result<SsoTokens, AuthError>;

    /// Validate an ID token and extract claims.
    fn validate_id_token(&self, token: &str) -> Result<SsoClaims, AuthError>;
}

struct SsoTokens {
    access_token: String,
    id_token: String,
    refresh_token: Option<String>,
}

struct SsoClaims {
    subject: String,
    email: Option<String>,
    issuer: String,
    audience: String,
    expires_at: DateTime<Utc>,
}
```

---

## 11. State Management

### 11.1 State Location

| State | Storage | Persistence |
|-------|---------|-------------|
| User records | cortex-db (SQLite) | Persistent |
| Password hashes | cortex-db (SQLite) | Persistent, never logged |
| Session records | cortex-db (SQLite) | Configurable: persistent (survives restart) or volatile (lost on restart) |
| Failed login counters | cortex-db (SQLite) | Persistent |
| SSO state (PKCE verifiers) | In-memory HashMap with TTL | Volatile (expires after 10 minutes) |
| Session cookie | Browser cookie | HTTP-only, Secure, SameSite=Strict |

### 11.2 Default Configuration

- Session persistence: persistent by default (survives server restart)
- Idle timeout: 1440 minutes (24 hours)
- Maximum session lifetime: 168 hours (7 days)
- Failed login lockout threshold: 5 attempts
- Initial lockout duration: 60 seconds
- Maximum lockout duration: 900 seconds (15 minutes)
- Session cleanup interval: 60 seconds

### 11.3 Concurrent Access

The session store uses database-level locking (SQLite WAL mode). Multiple concurrent requests for the same session are safe because session updates are idempotent (only `last_active_at` changes).

User record updates (profile changes, password changes, failed login counter increments) use row-level locking via SQLite transactions.

---

## 12. Failure Modes and Error Handling

### 12.1 Error Taxonomy

```rust
enum AuthError {
    /// Invalid username or password (identical message for both cases).
    InvalidCredentials,

    /// Account is temporarily locked due to too many failed attempts.
    AccountLocked { retry_after: Duration },

    /// Session token is missing, invalid, or expired.
    SessionInvalid,

    /// The requested username is already taken (during initial setup).
    UsernameTaken,

    /// Validation error on input data (e.g., password too short).
    ValidationError { field: String, reason: String },

    /// SSO provider returned an error.
    SsoError { provider: String, message: String },

    /// SSO state parameter expired or invalid.
    SsoStateInvalid,

    /// Internal storage error.
    StorageError { source: DbError },

    /// Password hashing failed (should never happen in normal operation).
    HashError,
}
```

### 12.2 Failure Scenarios

| Scenario | Behavior |
|----------|----------|
| Wrong password | Increment failed counter. Return 401 with generic "Invalid credentials" message. |
| Unknown username | Return 401 with identical "Invalid credentials" message. Do not reveal whether username exists. |
| Locked account | Return 429 with `Retry-After` header indicating seconds until lockout expires. |
| Expired session | Delete session from store. Return 401. Client shows login screen. |
| Corrupt session token | Treat as invalid session. Return 401. |
| Database unavailable | Return 503 Service Unavailable. Do not fall back to any in-memory authentication. |
| Password hash verification failure (internal error) | Log error at ERROR level. Return 500. Do not reveal hashing details to client. |
| SSO provider unreachable | Return 502 Bad Gateway with message "SSO provider unavailable." |
| SSO callback with invalid state | Return 400. Delete the stored PKCE verifier. |
| SSO ID token validation failure | Log warning. Return 401. Do not create user or session. |
| Avatar upload fails | Return 400 with specific error. Profile update is not applied. |

### 12.3 Recovery Behavior

- **After server restart:** If session persistence is enabled, existing sessions remain valid. If volatile, all sessions are lost and users must re-authenticate.
- **After database corruption:** `cortex-db` is responsible for recovery. `cortex-auth` reports storage errors and does not attempt auto-repair.
- **After failed login lockout:** The lockout expires automatically based on time. No manual unlock is needed.

---

## 13. Security and Permissions

### 13.1 Password Security

- Algorithm: Argon2id
- Parameters: memory=19456 KiB, iterations=2, parallelism=1 (OWASP recommended minimum)
- Minimum password length: 8 characters
- Maximum password length: 128 characters (prevents DoS via extremely long passwords)
- Passwords are never logged, never included in error messages, never stored in plaintext

### 13.2 Session Token Security

- Token generation: 256-bit cryptographically random value from `OsRng`
- Token encoding: base64url (no padding)
- Token storage in client: HTTP-only cookie (not accessible to JavaScript)
- Cookie flags: `Secure` (HTTPS only), `SameSite=Strict`, `HttpOnly`
- Token is not a JWT. It is an opaque reference to a server-side session record.
- Tokens are never included in URLs or log output.

### 13.3 SSO Security

- PKCE is mandatory for all SSO flows (authorization code + PKCE)
- `state` parameter is mandatory and validated on callback
- ID token validation checks: signature, issuer, audience, expiry
- Authorization code is single-use; replay results in error
- SSO client secrets are stored in `cortex-settings` under `system.sso_providers` and are never exposed to the client

### 13.4 Rate Limiting

- Login endpoint: maximum 10 requests per minute per IP address
- SSO callback endpoint: maximum 20 requests per minute per IP address
- Rate limiting is enforced before credential validation

### 13.5 Audit Trail

Every authentication event generates an audit log entry:
- Login success (includes user_id, not username)
- Login failure (includes source IP, NOT the attempted username)
- Logout
- Password change
- Account lockout
- Session expiration
- SSO login success/failure

Audit entries are written to `cortex-observability` via the logging interface. They are not stored in the user/session tables.

---

## 14. Performance Requirements

| Operation | Maximum Latency (p99) |
|-----------|----------------------|
| Login (credential verification) | 2000ms (dominated by Argon2id hashing) |
| Session validation (per request) | 5ms |
| Profile read | 10ms |
| Profile update | 20ms |
| Logout | 10ms |
| Session cleanup (batch) | 100ms for 1000 expired sessions |

### 14.1 Argon2id Performance Note

Argon2id is intentionally slow. The parameters chosen (m=19456, t=2, p=1) result in approximately 200-500ms per hash/verify operation on typical hardware. This is by design. Do not reduce the parameters without a documented security justification.

### 14.2 Session Validation Optimization

Session validation occurs on every authenticated API request. The session lookup is a single indexed query by primary key (session token). The `last_active_at` update is a lightweight write. For high-frequency scenarios, this update may be batched (update every 60 seconds instead of every request) but this must not extend the effective idle timeout beyond the configured value.

---

## 15. Accessibility Requirements

### 15.1 Login Screen

- All form fields have visible labels and associated `<label>` elements
- Error messages are announced to screen readers via `aria-live="polite"`
- Keyboard navigation: Tab order is username -> password -> login button
- Enter key submits the login form when focus is in either field
- Focus is moved to the first error message on validation failure
- Color is not the sole indicator of error state (icon + text also present)
- Minimum contrast ratio 4.5:1 for all text

### 15.2 Profile Settings

- Avatar upload has a text alternative describing the current avatar
- Form validation errors are associated with fields via `aria-describedby`
- All controls are keyboard-accessible

---

## 16. Observability and Logging

### 16.1 Log Events

| Event | Level | Fields |
|-------|-------|--------|
| Login success | INFO | user_id, ip, user_agent |
| Login failure | WARN | ip (NOT username, NOT password) |
| Account lockout | WARN | user_id, retry_after_seconds |
| Session expired | INFO | user_id, session_age_seconds |
| Logout | INFO | user_id |
| Password change | INFO | user_id |
| SSO redirect | INFO | provider_id, ip |
| SSO callback success | INFO | user_id, provider_id |
| SSO callback failure | WARN | provider_id, error_type |
| Profile update | INFO | user_id, fields_changed |
| Initial user created | INFO | user_id |

### 16.2 Metrics

| Metric | Type | Labels |
|--------|------|--------|
| auth_login_total | Counter | status (success/failure) |
| auth_login_duration_ms | Histogram | |
| auth_session_active | Gauge | |
| auth_session_expired_total | Counter | |
| auth_password_hash_duration_ms | Histogram | |

### 16.3 Sensitive Data

The following data is NEVER logged:
- Passwords (plaintext or hashed)
- Session tokens
- SSO access tokens or refresh tokens
- SSO client secrets

---

## 17. Testing Requirements

### 17.1 Unit Tests

| Test | Description |
|------|-------------|
| `test_password_hash_and_verify` | Hash a password, verify it matches, verify wrong password does not match |
| `test_password_hash_different_salts` | Same password hashed twice produces different hashes |
| `test_session_token_uniqueness` | Generate 10000 tokens, assert all unique |
| `test_session_token_entropy` | Verify token length is correct (256-bit = 32 bytes base64url) |
| `test_username_validation` | Valid: "alice", "bob_123". Invalid: "a", "ab", "a b", "a@b", empty, 33 chars |
| `test_display_name_validation` | Valid: "Alice", "A". Invalid: empty, 65 chars |
| `test_password_validation` | Valid: 8+ chars. Invalid: 7 chars, empty, 129 chars |
| `test_failed_login_counter_increment` | Failed login increments counter |
| `test_failed_login_counter_reset_on_success` | Successful login resets counter |
| `test_account_lockout_after_5_failures` | 5th failure triggers lockout |
| `test_account_lockout_duration_doubles` | Verify progressive lockout durations |
| `test_session_expiration` | Session with past expires_at is rejected |
| `test_session_idle_timeout` | Session with old last_active_at is rejected |
| `test_session_max_lifetime` | Session exceeding max lifetime is rejected |
| `test_logout_deletes_session` | Session token is invalid after logout |
| `test_logout_all_deletes_all_sessions` | All sessions for user are invalid after logout_all |
| `test_profile_update_display_name` | Update display name, verify persisted |
| `test_password_change` | Change password, verify old password fails, new password works |
| `test_password_change_wrong_current` | Reject password change with wrong current password |

### 17.2 Integration Tests

| Test | Description |
|------|-------------|
| `test_full_login_flow` | Create user -> login -> make authenticated request -> logout |
| `test_login_with_locked_account` | Lock account -> attempt login -> verify 429 -> wait for unlock -> verify login succeeds |
| `test_multiple_sessions` | Login twice -> verify both sessions valid -> logout one -> verify other still valid |
| `test_session_persistence_across_restart` | Create session -> restart DB -> verify session still valid |
| `test_sso_flow_mock` | Mock SSO provider -> initiate -> callback -> verify user created and session established |
| `test_initial_user_creation` | Verify first-boot user creation with valid data |
| `test_initial_user_creation_duplicate_username` | Verify rejection of duplicate username |

### 17.3 Security Tests

| Test | Description |
|------|-------------|
| `test_no_username_enumeration` | Login with nonexistent user and wrong password produce identical responses |
| `test_session_token_not_in_logs` | Login, search all log output for session token, verify absent |
| `test_password_not_in_logs` | Login with wrong password, search logs, verify password absent |
| `test_expired_session_is_deleted` | Create session with past expiry, validate, verify session removed from store |
| `test_rate_limiting_login` | Send 11 login requests in rapid succession, verify 11th returns 429 |

---

## 18. Acceptance Criteria

### 18.1 Functional Checklist

- [ ] A user can be created during first-boot setup with username, display name, and password
- [ ] A user can log in with correct credentials and receive a valid session
- [ ] A user cannot log in with incorrect credentials
- [ ] Failed login attempts are counted and trigger lockout after 5 failures
- [ ] Lockout duration follows the exponential backoff schedule: 60, 120, 240, ..., 900 seconds
- [ ] A locked account is unlocked automatically after the lockout duration expires
- [ ] A user can log out, after which their session token is invalid
- [ ] A user can log out all sessions, after which all their session tokens are invalid
- [ ] Session expiration (idle timeout and max lifetime) works correctly
- [ ] A user can update their display name
- [ ] A user can change their password (with current password verification)
- [ ] A user can upload an avatar
- [ ] SSO login works end-to-end when configured (tested with a mock provider)
- [ ] SSO UI is hidden when no SSO providers are configured
- [ ] Login screen is accessible (keyboard navigable, screen reader compatible)
- [ ] All authentication events produce audit log entries
- [ ] Passwords and session tokens never appear in logs

### 18.2 Performance Checklist

- [ ] Login completes in under 2 seconds at p99
- [ ] Session validation completes in under 5ms at p99
- [ ] Session cleanup processes 1000 expired sessions in under 100ms

### 18.3 Security Checklist

- [ ] Password hashing uses Argon2id with OWASP-recommended parameters
- [ ] Session tokens are 256-bit random values
- [ ] Session cookies are HttpOnly, Secure, SameSite=Strict
- [ ] Username enumeration is not possible (identical error responses)
- [ ] Rate limiting is enforced on the login endpoint
- [ ] SSO flows use PKCE
- [ ] SSO ID tokens are fully validated (signature, issuer, audience, expiry)

---

## 19. Build Order and Dependencies
**Layer 2**. Depends on: 01 (repo/toolchain), 02 (core architecture, error taxonomy)

### 19.1 Build Sequence

`cortex-auth` must be built after:
1. `cortex-core` (shared types, error macros)
2. `cortex-db` (storage interface, SQLite backend)
3. `cortex-settings` (reads SSO config, session timeout config)

`cortex-auth` must be built before:
1. `cortex-api` (HTTP route registration)
2. `cortex-policy` (user context for permission checks)
3. `cortex-runtime` (app sandboxing needs user context)
4. `apps/desktop-shell` (login screen, session management)
5. `apps/settings-app` (profile management UI)

### 19.2 External Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| argon2 | 0.5 | Password hashing |
| uuid | 1.x | UUIDv7 generation for user IDs |
| rand | 0.8 | Cryptographic random number generation |
| base64 | 0.22 | Session token encoding |
| serde | 1.x | Serialization/deserialization |
| serde_json | 1.x | JSON serialization |
| chrono | 0.4 | Timestamp handling |
| sha2 | 0.10 | PKCE code challenge generation |
|jsonwebtoken | 9.x | SSO ID token validation |

---

## 20. Non-Goals and Anti-Patterns

### 20.1 Non-Goals

- Multi-user concurrent sessions with isolation (not v1)
- Biometric authentication (not v1)
- Passwordless authentication (not v1)
- Role-based access control (RBAC) or multi-role users (single-user OS)
- Email-based password reset (no email infrastructure in local mode)
- LDAP or Active Directory integration (not v1)
- Two-factor authentication (not v1, but the design does not preclude it)
- User-to-user messaging or social features

### 20.2 Anti-Patterns

1. **NEVER store passwords in plaintext.** Always use Argon2id.
2. **NEVER reveal whether a username exists.** Return identical error messages for "wrong username" and "wrong password."
3. **NEVER include session tokens or passwords in log output.**
4. **NEVER trust client-side session validation.** All session checks are server-side.
5. **NEVER allow password changes without current password verification.**
6. **NEVER use JWTs for sessions.** Sessions are opaque server-side records. JWTs are used only for SSO ID token validation.
7. **NEVER skip rate limiting on authentication endpoints.**
8. **NEVER allow first-party apps to bypass authentication.** All apps go through the same session validation middleware.
9. **NEVER silently downgrade authentication.** If SSO is configured and the SSO provider is unreachable, do not fall back to a local password. Return an error.
10. **NEVER hash passwords on the client side.** Transport is secured by HTTPS. Client-side hashing breaks the server's ability to verify against stored Argon2id hashes.

---

## 21. Implementation Instructions for Claude Code / Codex

### 21.1 Subsystem Ownership Reminder

You are implementing `cortex-auth`. You own:
- User identity model and storage
- Credential hashing and verification
- Session lifecycle (create, validate, refresh, expire, destroy)
- Profile CRUD operations
- SSO adapter interface and OAuth 2.0 implementation
- Authentication API route handlers

You do NOT own:
- Authorization/permissions (that is `cortex-policy`)
- Settings storage (that is `cortex-settings`)
- File storage for avatars (that is `cortex-files`)
- HTTP server setup (that is `cortex-api`)

### 21.2 Recommended Crate Structure

```
crates/cortex-auth/
  Cargo.toml
  src/
    lib.rs           -- pub mod identity; pub mod session; etc.
    identity.rs      -- User, UserId, UserProfile structs and validation
    credentials.rs   -- CredentialHasher (Argon2id hash + verify)
    session.rs       -- Session, SessionToken, SessionStore interactions
    profile.rs       -- profile update logic
    sso/
      mod.rs         -- SsoProviderAdapter trait
      oauth2.rs      -- OAuth2/OIDC implementation
    api/
      mod.rs         -- route registration function
      handlers.rs    -- handler functions for each endpoint
    error.rs         -- AuthError enum
    validation.rs    -- input validation helpers
```

### 21.3 What Can Be Stubbed Initially

- SSO adapter implementations (`sso/oauth2.rs`) can be stubbed with `todo!()` in early builds if SSO is not the first priority.
- Rate limiting can initially be a simple in-memory counter. A production-grade rate limiter can replace it later.
- Avatar file handling can be stubbed by accepting the upload but returning a placeholder avatar URL.

### 21.4 What Must Be Real in v1

- Argon2id password hashing and verification (not a stub, not bcrypt, not SHA)
- Session token generation using `OsRng` (not a predictable source)
- Server-side session store backed by SQLite via `cortex-db`
- Login/logout/session validation API endpoints
- User profile read/update
- Account lockout logic
- Failed login counter
- Session expiration (idle timeout and max lifetime)
- Audit logging for all authentication events

### 21.5 What Cannot Be Inferred

- Argon2id parameters must be exactly m=19456, t=2, p=1. Do not guess or use different parameters.
- Session tokens must be exactly 256 bits from `OsRng`. Do not use UUIDs or timestamps as tokens.
- Cookie flags must be `HttpOnly`, `Secure`, `SameSite=Strict`. Do not omit any of these.
- The lockout schedule is 5 failures then 60s, doubling, capped at 900s. Do not implement a different schedule.
- Username validation is exactly 3-32 ASCII alphanumeric + underscore. Do not allow Unicode in usernames.

### 21.6 Stop Conditions

The subsystem is considered done when:
1. All unit tests pass (see section 17.1)
2. All integration tests pass (see section 17.2)
3. All security tests pass (see section 17.3)
4. The acceptance criteria checklist (section 18) is fully verified
5. No compiler warnings in `cortex-auth`
6. `cargo clippy --all-targets` produces no warnings for `cortex-auth`
7. `cargo audit` shows no known vulnerabilities in `cortex-auth` dependencies

### 21.7 Testing Gates

Before marking this subsystem as complete, verify:
- `cargo test -p cortex-auth` passes with 100% of listed tests present and passing
- `cargo test -p cortex-auth -- --nocapture` shows correct log output (no sensitive data)
- Manual test: create user, login, make authenticated request, logout, verify session invalid
- Manual test: login 6 times with wrong password, verify lockout on 6th attempt
