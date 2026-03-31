//! Policy service trait.
//!
//! All authorization checks go through this trait. The concrete
//! implementation is constructed in cortex-api and injected where needed.

use crate::error::Result;
use crate::types::PermissionGrant;
use cortex_core::{AppId, UserId};

/// Policy service trait.
///
/// Provides the core authorization surface for CortexOS. Every subsystem must
/// route its permission checks through a type that implements this trait -- no
/// direct bypass is permitted.
pub trait PolicyService: Send + Sync {
    /// Check whether the given user/app combination holds the named permission.
    ///
    /// Returns `Ok(true)` when the permission is granted, `Ok(false)` when it
    /// is not, and `Err` only when the check itself failed (e.g. a storage
    /// error).
    fn check_permission(
        &self,
        user_id: &UserId,
        app_id: &AppId,
        permission: &str,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    /// Persist a new permission grant.
    ///
    /// The caller is responsible for constructing the [`PermissionGrant`] with
    /// a unique `grant_id` and the current timestamp.
    fn grant(&self, grant: PermissionGrant)
        -> impl std::future::Future<Output = Result<()>> + Send;

    /// Revoke an existing permission grant by its ID.
    ///
    /// Returns [`PolicyError::GrantNotFound`] when the ID does not exist.
    fn revoke(&self, grant_id: &str) -> impl std::future::Future<Output = Result<()>> + Send;

    /// List all permission grants for a user/app pair.
    fn list_grants(
        &self,
        user_id: &UserId,
        app_id: &AppId,
    ) -> impl std::future::Future<Output = Result<Vec<PermissionGrant>>> + Send;
}
