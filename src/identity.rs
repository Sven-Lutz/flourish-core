//! Flourish product identity, authentication lifecycle, and Space ownership.
//!
//! A [`User`] is independent of authentication-provider credentials and of
//! integration accounts. Every client mode, including `LocalOnly`, carries
//! stable user, device, and Personal Space identifiers; local fallback is not
//! an anonymous-singleton architecture.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::{
    AuthIdentityBindingId, AuthSessionId, DeviceId, MembershipId, RecoveryChallengeId, SpaceId,
    Timestamp, UserId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserStatus {
    Active,
    Deactivated,
}

/// Stable Flourish product identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    id: UserId,
    status: UserStatus,
    personal_space_ref: SpaceId,
    created_at: Timestamp,
    deactivated_at: Option<Timestamp>,
}

impl User {
    pub fn new(id: UserId, personal_space_ref: SpaceId, created_at: Timestamp) -> Self {
        Self {
            id,
            status: UserStatus::Active,
            personal_space_ref,
            created_at,
            deactivated_at: None,
        }
    }

    pub fn id(&self) -> &UserId {
        &self.id
    }
    pub const fn status(&self) -> UserStatus {
        self.status
    }
    pub fn personal_space_ref(&self) -> &SpaceId {
        &self.personal_space_ref
    }
    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }
    pub fn deactivated_at(&self) -> Option<&Timestamp> {
        self.deactivated_at.as_ref()
    }

    /// Idempotently deactivates the account, preserving the first timestamp.
    pub fn deactivate(&mut self, at: Timestamp) {
        if self.deactivated_at.is_none() {
            self.status = UserStatus::Deactivated;
            self.deactivated_at = Some(at);
        }
    }
}

/// Provider used only to authenticate a Flourish user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthProvider {
    Password,
    Passkey,
    GoogleOAuth,
    MicrosoftOAuth,
    AppleOAuth,
    MagicLink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthIdentityBindingViolation {
    EmptyExternalSubject,
    RevocationBeforeLink,
}

impl Display for AuthIdentityBindingViolation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyExternalSubject => "external_subject must not be empty",
            Self::RevocationBeforeLink => "binding revocation cannot precede link time",
        })
    }
}
impl Error for AuthIdentityBindingViolation {}

/// Authentication credential binding; never a product or integration identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthIdentityBinding {
    id: AuthIdentityBindingId,
    user_ref: UserId,
    provider: AuthProvider,
    external_subject: String,
    linked_at: Timestamp,
    revoked_at: Option<Timestamp>,
}

impl AuthIdentityBinding {
    pub fn new(
        id: AuthIdentityBindingId,
        user_ref: UserId,
        provider: AuthProvider,
        external_subject: String,
        linked_at: Timestamp,
    ) -> Result<Self, AuthIdentityBindingViolation> {
        if external_subject.trim().is_empty() {
            return Err(AuthIdentityBindingViolation::EmptyExternalSubject);
        }
        Ok(Self {
            id,
            user_ref,
            provider,
            external_subject,
            linked_at,
            revoked_at: None,
        })
    }

    pub fn id(&self) -> &AuthIdentityBindingId {
        &self.id
    }
    pub fn user_ref(&self) -> &UserId {
        &self.user_ref
    }
    pub const fn provider(&self) -> AuthProvider {
        self.provider
    }
    pub fn external_subject(&self) -> &str {
        &self.external_subject
    }
    pub fn linked_at(&self) -> &Timestamp {
        &self.linked_at
    }
    pub fn revoked_at(&self) -> Option<&Timestamp> {
        self.revoked_at.as_ref()
    }
    pub fn is_active(&self) -> bool {
        self.revoked_at.is_none()
    }

    pub fn revoke(&mut self, at: Timestamp) -> Result<(), AuthIdentityBindingViolation> {
        if at < self.linked_at {
            return Err(AuthIdentityBindingViolation::RevocationBeforeLink);
        }
        if self.revoked_at.is_none() {
            self.revoked_at = Some(at);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientPlatform {
    Desktop,
    Web,
    Mobile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceStatus {
    Active,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceViolation {
    EmptyDisplayName,
    RevocationBeforeRegistration,
}

impl Display for DeviceViolation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyDisplayName => "display_name must not be empty",
            Self::RevocationBeforeRegistration => "device revocation cannot precede registration",
        })
    }
}
impl Error for DeviceViolation {}

/// Registered Desktop, Web, or Mobile client instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    id: DeviceId,
    user_ref: UserId,
    platform: ClientPlatform,
    display_name: String,
    status: DeviceStatus,
    registered_at: Timestamp,
    last_seen_at: Option<Timestamp>,
    revoked_at: Option<Timestamp>,
}

impl Device {
    pub fn new(
        id: DeviceId,
        user_ref: UserId,
        platform: ClientPlatform,
        display_name: String,
        registered_at: Timestamp,
    ) -> Result<Self, DeviceViolation> {
        if display_name.trim().is_empty() {
            return Err(DeviceViolation::EmptyDisplayName);
        }
        Ok(Self {
            id,
            user_ref,
            platform,
            display_name,
            status: DeviceStatus::Active,
            registered_at,
            last_seen_at: None,
            revoked_at: None,
        })
    }

    pub fn id(&self) -> &DeviceId {
        &self.id
    }
    pub fn user_ref(&self) -> &UserId {
        &self.user_ref
    }
    pub const fn platform(&self) -> ClientPlatform {
        self.platform
    }
    pub fn display_name(&self) -> &str {
        &self.display_name
    }
    pub const fn status(&self) -> DeviceStatus {
        self.status
    }
    pub fn registered_at(&self) -> &Timestamp {
        &self.registered_at
    }
    pub fn last_seen_at(&self) -> Option<&Timestamp> {
        self.last_seen_at.as_ref()
    }
    pub fn revoked_at(&self) -> Option<&Timestamp> {
        self.revoked_at.as_ref()
    }

    pub fn record_activity(&mut self, at: Timestamp) {
        if self
            .last_seen_at
            .as_ref()
            .is_none_or(|previous| at > *previous)
        {
            self.last_seen_at = Some(at);
        }
    }

    pub fn revoke(&mut self, at: Timestamp) -> Result<(), DeviceViolation> {
        if at < self.registered_at {
            return Err(DeviceViolation::RevocationBeforeRegistration);
        }
        if self.revoked_at.is_none() {
            self.status = DeviceStatus::Revoked;
            self.revoked_at = Some(at);
        }
        Ok(())
    }
}

/// Persistable one-way digest; raw bearer secrets never enter domain records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CredentialDigest([u8; 32]);

impl CredentialDigest {
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthSessionViolation {
    ExpiryNotAfterIssue,
    DeviceUserMismatch,
    DeviceRevoked,
    RevocationBeforeIssue,
}

impl Display for AuthSessionViolation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ExpiryNotAfterIssue => "AuthSession must expire after it is issued",
            Self::DeviceUserMismatch => "AuthSession user must own its Device",
            Self::DeviceRevoked => "a revoked Device cannot create an AuthSession",
            Self::RevocationBeforeIssue => "session revocation cannot precede issue time",
        })
    }
}
impl Error for AuthSessionViolation {}

/// Authenticated, expiring, revocable user/device session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthSession {
    id: AuthSessionId,
    user_ref: UserId,
    device_ref: DeviceId,
    credential_digest: CredentialDigest,
    issued_at: Timestamp,
    expires_at: Timestamp,
    revoked_at: Option<Timestamp>,
}

impl AuthSession {
    pub fn new(
        id: AuthSessionId,
        user_ref: UserId,
        device: &Device,
        credential_digest: CredentialDigest,
        issued_at: Timestamp,
        expires_at: Timestamp,
    ) -> Result<Self, AuthSessionViolation> {
        if expires_at <= issued_at {
            return Err(AuthSessionViolation::ExpiryNotAfterIssue);
        }
        if device.user_ref() != &user_ref {
            return Err(AuthSessionViolation::DeviceUserMismatch);
        }
        if matches!(device.status(), DeviceStatus::Revoked) {
            return Err(AuthSessionViolation::DeviceRevoked);
        }
        Ok(Self {
            id,
            user_ref,
            device_ref: device.id().clone(),
            credential_digest,
            issued_at,
            expires_at,
            revoked_at: None,
        })
    }

    pub fn id(&self) -> &AuthSessionId {
        &self.id
    }
    pub fn user_ref(&self) -> &UserId {
        &self.user_ref
    }
    pub fn device_ref(&self) -> &DeviceId {
        &self.device_ref
    }
    pub const fn credential_digest(&self) -> &CredentialDigest {
        &self.credential_digest
    }
    pub fn issued_at(&self) -> &Timestamp {
        &self.issued_at
    }
    pub fn expires_at(&self) -> &Timestamp {
        &self.expires_at
    }
    pub fn revoked_at(&self) -> Option<&Timestamp> {
        self.revoked_at.as_ref()
    }

    /// Active only when the whole account-device-session chain is valid.
    pub fn is_active(&self, user: &User, device: &Device, now: &Timestamp) -> bool {
        self.user_ref == *user.id()
            && self.device_ref == *device.id()
            && matches!(user.status(), UserStatus::Active)
            && matches!(device.status(), DeviceStatus::Active)
            && self.revoked_at.is_none()
            && now >= &self.issued_at
            && now < &self.expires_at
    }

    /// Idempotent logout/revocation.
    pub fn revoke(&mut self, at: Timestamp) -> Result<(), AuthSessionViolation> {
        if at < self.issued_at {
            return Err(AuthSessionViolation::RevocationBeforeIssue);
        }
        if self.revoked_at.is_none() {
            self.revoked_at = Some(at);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryChallengeViolation {
    ExpiryNotAfterIssue,
    AlreadyConsumed,
    Expired,
    ConsumptionBeforeIssue,
}

impl Display for RecoveryChallengeViolation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ExpiryNotAfterIssue => "RecoveryChallenge must expire after it is issued",
            Self::AlreadyConsumed => "RecoveryChallenge has already been consumed",
            Self::Expired => "RecoveryChallenge has expired",
            Self::ConsumptionBeforeIssue => "recovery consumption cannot precede issue time",
        })
    }
}
impl Error for RecoveryChallengeViolation {}

/// Single-use, expiring account-recovery challenge containing no raw secret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryChallenge {
    id: RecoveryChallengeId,
    user_ref: UserId,
    credential_digest: CredentialDigest,
    issued_at: Timestamp,
    expires_at: Timestamp,
    consumed_at: Option<Timestamp>,
}

impl RecoveryChallenge {
    pub fn new(
        id: RecoveryChallengeId,
        user_ref: UserId,
        credential_digest: CredentialDigest,
        issued_at: Timestamp,
        expires_at: Timestamp,
    ) -> Result<Self, RecoveryChallengeViolation> {
        if expires_at <= issued_at {
            return Err(RecoveryChallengeViolation::ExpiryNotAfterIssue);
        }
        Ok(Self {
            id,
            user_ref,
            credential_digest,
            issued_at,
            expires_at,
            consumed_at: None,
        })
    }

    pub fn id(&self) -> &RecoveryChallengeId {
        &self.id
    }
    pub fn user_ref(&self) -> &UserId {
        &self.user_ref
    }
    pub const fn credential_digest(&self) -> &CredentialDigest {
        &self.credential_digest
    }
    pub fn issued_at(&self) -> &Timestamp {
        &self.issued_at
    }
    pub fn expires_at(&self) -> &Timestamp {
        &self.expires_at
    }
    pub fn consumed_at(&self) -> Option<&Timestamp> {
        self.consumed_at.as_ref()
    }

    pub fn consume(&mut self, at: Timestamp) -> Result<(), RecoveryChallengeViolation> {
        if self.consumed_at.is_some() {
            return Err(RecoveryChallengeViolation::AlreadyConsumed);
        }
        if at < self.issued_at {
            return Err(RecoveryChallengeViolation::ConsumptionBeforeIssue);
        }
        if at >= self.expires_at {
            return Err(RecoveryChallengeViolation::Expired);
        }
        self.consumed_at = Some(at);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceKind {
    Personal,
    Shared,
}

/// Ownership boundary for durable operational objects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Space {
    id: SpaceId,
    kind: SpaceKind,
    owner_user_ref: UserId,
    created_at: Timestamp,
}

impl Space {
    pub fn new(
        id: SpaceId,
        kind: SpaceKind,
        owner_user_ref: UserId,
        created_at: Timestamp,
    ) -> Self {
        Self {
            id,
            kind,
            owner_user_ref,
            created_at,
        }
    }
    pub fn id(&self) -> &SpaceId {
        &self.id
    }
    pub const fn kind(&self) -> SpaceKind {
        self.kind
    }
    pub fn owner_user_ref(&self) -> &UserId {
        &self.owner_user_ref
    }
    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembershipRole {
    Owner,
    Member,
    Viewer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembershipViolation {
    PersonalSpaceMembershipNotAllowed,
    OwnerRoleRequiresSpaceOwner,
    RevocationBeforeCreation,
}

impl Display for MembershipViolation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::PersonalSpaceMembershipNotAllowed => "a PERSONAL Space cannot have a Membership",
            Self::OwnerRoleRequiresSpaceOwner => {
                "the OWNER role may only be held by the Space owner"
            }
            Self::RevocationBeforeCreation => "membership revocation cannot precede creation",
        })
    }
}
impl Error for MembershipViolation {}

/// Minimal future-compatible membership boundary for a Shared Space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Membership {
    id: MembershipId,
    space_ref: SpaceId,
    user_ref: UserId,
    role: MembershipRole,
    created_at: Timestamp,
    revoked_at: Option<Timestamp>,
}

impl Membership {
    pub fn new(
        id: MembershipId,
        space: &Space,
        user_ref: UserId,
        role: MembershipRole,
        created_at: Timestamp,
    ) -> Result<Self, MembershipViolation> {
        if matches!(space.kind(), SpaceKind::Personal) {
            return Err(MembershipViolation::PersonalSpaceMembershipNotAllowed);
        }
        if matches!(role, MembershipRole::Owner) && user_ref != *space.owner_user_ref() {
            return Err(MembershipViolation::OwnerRoleRequiresSpaceOwner);
        }
        Ok(Self {
            id,
            space_ref: space.id().clone(),
            user_ref,
            role,
            created_at,
            revoked_at: None,
        })
    }

    pub fn id(&self) -> &MembershipId {
        &self.id
    }
    pub fn space_ref(&self) -> &SpaceId {
        &self.space_ref
    }
    pub fn user_ref(&self) -> &UserId {
        &self.user_ref
    }
    pub const fn role(&self) -> MembershipRole {
        self.role
    }
    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }
    pub fn revoked_at(&self) -> Option<&Timestamp> {
        self.revoked_at.as_ref()
    }
    pub fn is_active(&self) -> bool {
        self.revoked_at.is_none()
    }

    pub fn revoke(&mut self, at: Timestamp) -> Result<(), MembershipViolation> {
        if at < self.created_at {
            return Err(MembershipViolation::RevocationBeforeCreation);
        }
        if self.revoked_at.is_none() {
            self.revoked_at = Some(at);
        }
        Ok(())
    }
}

/// Account/connectivity mode consumed consistently by all client platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityRuntimeMode {
    LocalOnly,
    AccountOnline,
    AccountOffline,
    ReauthenticationRequired,
}

/// Explicit identity envelope passed to trusted domain/storage boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientIdentityContext {
    user_ref: UserId,
    personal_space_ref: SpaceId,
    device_ref: DeviceId,
    mode: IdentityRuntimeMode,
}

impl ClientIdentityContext {
    pub const fn new(
        user_ref: UserId,
        personal_space_ref: SpaceId,
        device_ref: DeviceId,
        mode: IdentityRuntimeMode,
    ) -> Self {
        Self {
            user_ref,
            personal_space_ref,
            device_ref,
            mode,
        }
    }
    pub fn user_ref(&self) -> &UserId {
        &self.user_ref
    }
    pub fn personal_space_ref(&self) -> &SpaceId {
        &self.personal_space_ref
    }
    pub fn device_ref(&self) -> &DeviceId {
        &self.device_ref
    }
    pub const fn mode(&self) -> IdentityRuntimeMode {
        self.mode
    }
    pub const fn is_account_backed(&self) -> bool {
        !matches!(self.mode, IdentityRuntimeMode::LocalOnly)
    }
}
