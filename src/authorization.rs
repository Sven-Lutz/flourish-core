//! Kernel privacy/authorization boundary for Space- and Membership-scoped access.
//!
//! This is the pure decision logic a `PermissionService` (see
//! [`crate::ServiceId::PermissionService`]) enforces below the UI layer. No
//! presentation surface may substitute for this check.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::{
    AuthSession, Device, Membership, MembershipRole, SensitivityClass, Space, SpaceKind, Timestamp,
    User,
};

/// Kind of access being requested against Space-owned data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessAction {
    /// Read the data.
    Read,
    /// Create or mutate the data.
    Write,
}

/// Fully scoped request evaluated by the trusted authorization boundary.
#[derive(Debug, Clone, Copy)]
pub struct SpaceAccessRequest<'a> {
    pub action: AccessAction,
    pub sensitivity: SensitivityClass,
    pub space: &'a Space,
    pub membership: Option<&'a Membership>,
}

/// Reason access was denied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorizationViolation {
    /// The presented session does not authenticate the claimed actor right now.
    SessionInactive,
    /// The claimed Personal Space is not the actor's stable Personal Space.
    PersonalSpaceOwnershipMismatch,
    /// A Personal Space is private to its owner; there is no membership path in.
    PersonalSpaceIsPrivate,
    /// The actor has no active Membership in the Shared Space.
    NoActiveMembership,
    /// Sensitive personal domains remain private to the owner even in a Shared Space.
    SensitiveDomainRequiresOwner,
    /// A `VIEWER` Membership cannot perform a write.
    ViewerCannotWrite,
}

impl Display for AuthorizationViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::SessionInactive => "session is not an active session for the claimed actor",
            Self::PersonalSpaceOwnershipMismatch => {
                "PERSONAL Space does not match the actor's Personal Space"
            }
            Self::PersonalSpaceIsPrivate => "a PERSONAL Space is private to its owner",
            Self::NoActiveMembership => "actor has no active Membership in this Space",
            Self::SensitiveDomainRequiresOwner => {
                "sensitive personal data is visible only to the Space owner"
            }
            Self::ViewerCannotWrite => "a VIEWER Membership cannot write",
        })
    }
}

impl Error for AuthorizationViolation {}

/// Evaluates whether `actor`, authenticated by `session`, may perform `action`
/// on data of `sensitivity` owned by `space`.
///
/// The owner of a Space always has full access to their own data. Anyone else
/// requires an active `Membership` in a `SHARED` Space, and even then may
/// never reach a non-`Standard` sensitivity class: journal/reflection,
/// wellbeing, and biometric domains remain private to the owner in Alpha.
pub fn authorize_space_access(
    actor: &User,
    device: &Device,
    request: SpaceAccessRequest<'_>,
    session: &AuthSession,
    now: &Timestamp,
) -> Result<(), AuthorizationViolation> {
    if !session.is_active(actor, device, now) {
        return Err(AuthorizationViolation::SessionInactive);
    }

    if matches!(request.space.kind(), SpaceKind::Personal)
        && (request.space.owner_user_ref() != actor.id()
            || request.space.id() != actor.personal_space_ref())
    {
        return Err(AuthorizationViolation::PersonalSpaceOwnershipMismatch);
    }

    if actor.id() == request.space.owner_user_ref() {
        return Ok(());
    }

    if matches!(request.space.kind(), SpaceKind::Personal) {
        return Err(AuthorizationViolation::PersonalSpaceIsPrivate);
    }

    let membership = request
        .membership
        .filter(|membership| {
            membership.user_ref() == actor.id()
                && membership.space_ref() == request.space.id()
                && membership.is_active()
        })
        .ok_or(AuthorizationViolation::NoActiveMembership)?;

    if !matches!(request.sensitivity, SensitivityClass::Standard) {
        return Err(AuthorizationViolation::SensitiveDomainRequiresOwner);
    }

    if matches!(request.action, AccessAction::Write)
        && matches!(membership.role(), MembershipRole::Viewer)
    {
        return Err(AuthorizationViolation::ViewerCannotWrite);
    }

    Ok(())
}
