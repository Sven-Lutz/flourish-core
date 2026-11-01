//! Kernel value types and typed identifiers.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// Error returned when a required textual contract value is empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptyValueError {
    type_name: &'static str,
}

impl EmptyValueError {
    fn new(type_name: &'static str) -> Self {
        Self { type_name }
    }
}

impl Display for EmptyValueError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} must not be empty", self.type_name)
    }
}

impl Error for EmptyValueError {}

macro_rules! non_empty_string_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// Creates the value after rejecting empty or whitespace-only text.
            pub fn new(value: impl Into<String>) -> Result<Self, EmptyValueError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(EmptyValueError::new(stringify!($name)));
                }
                Ok(Self(value))
            }

            /// Returns the underlying contract string.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = EmptyValueError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }
    };
}

non_empty_string_type!(
    /// Stable identifier for a Capability Pack.
    PackId
);
non_empty_string_type!(
    /// Stable identifier for a Protocol.
    ProtocolId
);
non_empty_string_type!(
    /// Stable identifier for a pack-owned learning strategy.
    StrategyId
);
non_empty_string_type!(
    /// Stable identifier for a registered Context Fact.
    ContextFactRef
);
non_empty_string_type!(
    /// Stable event-topic name.
    EventTopic
);
non_empty_string_type!(
    /// Opaque reference whose payload remains owned by another boundary.
    OpaqueRef
);
non_empty_string_type!(
    /// Stable identifier for a deterministic Composer rule.
    ComposerRuleId
);
non_empty_string_type!(
    /// Stable identifier for a schema-owned typed Composer need.
    ComposerNeedId
);
non_empty_string_type!(
    /// Stable observation-type identifier.
    ObservationType
);
non_empty_string_type!(
    /// Stable pack-owned entity-kind identifier.
    PackEntityKind
);
non_empty_string_type!(
    /// Stable integration binding identifier.
    IntegrationBindingRef
);
non_empty_string_type!(
    /// Stable identifier for a catalog Method definition.
    MethodId
);
non_empty_string_type!(
    /// Stable identifier for a catalog Integration Capability.
    IntegrationCapabilityId
);
non_empty_string_type!(
    /// Stable identifier for a governed Evidence Claim.
    EvidenceClaimId
);
non_empty_string_type!(
    /// Stable identifier for a governed Design Hypothesis.
    DesignHypothesisId
);
non_empty_string_type!(
    /// Stable, namespaced integration operation identifier.
    IntegrationOperationId
);
non_empty_string_type!(
    /// Stable idempotency key supplied by an explicit acceptance command.
    IdempotencyKey
);

/// A validated Crockford-base32 ULID.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(String);

impl Id {
    /// Parses the exact 26-character canonical uppercase ULID representation.
    pub fn new(value: impl Into<String>) -> Result<Self, IdError> {
        let value = value.into();
        if value.len() != 26 {
            return Err(IdError::InvalidLength(value.len()));
        }
        if !value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'A'..=b'H' | b'J'..=b'N' | b'P'..=b'T' | b'V'..=b'Z'))
        {
            return Err(IdError::InvalidCharacter);
        }
        if value.as_bytes()[0] > b'7' {
            return Err(IdError::Overflow);
        }
        Ok(Self(value))
    }

    /// Returns the canonical ULID string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Id {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for Id {
    type Err = IdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

/// Validation error for a canonical ULID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdError {
    /// The identifier was not 26 bytes long.
    InvalidLength(usize),
    /// The identifier used lowercase or a character excluded by Crockford base32.
    InvalidCharacter,
    /// The first character would exceed the 128-bit ULID space.
    Overflow,
}

impl Display for IdError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(length) => {
                write!(formatter, "ULID must contain 26 characters, found {length}")
            }
            Self::InvalidCharacter => {
                formatter.write_str("ULID contains a non-canonical character")
            }
            Self::Overflow => formatter.write_str("ULID exceeds the 128-bit value space"),
        }
    }
}

impl Error for IdError {}

macro_rules! typed_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(Id);

        impl $name {
            /// Creates the typed identifier from a validated ULID.
            pub fn new(id: Id) -> Self {
                Self(id)
            }

            /// Parses a canonical ULID into this typed identifier.
            pub fn parse(value: impl Into<String>) -> Result<Self, IdError> {
                Id::new(value).map(Self)
            }

            /// Returns the underlying ULID.
            pub fn id(&self) -> &Id {
                &self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                Display::fmt(&self.0, formatter)
            }
        }
    };
}

typed_id!(
    /// Identifier for a Pursuit.
    PursuitId
);
typed_id!(
    /// Identifier for a Practice Item.
    PracticeItemId
);
typed_id!(
    /// Identifier for a Session.
    SessionId
);
typed_id!(
    /// Identifier for an Attempt.
    AttemptId
);
typed_id!(
    /// Identifier for an Observation.
    ObservationId
);
typed_id!(
    /// Identifier for a Decision Record.
    DecisionRecordId
);
typed_id!(
    /// Identifier for a Reflection Outcome.
    ReflectionOutcomeId
);
typed_id!(
    /// Identifier for a Calibration Record.
    CalibrationRecordId
);
typed_id!(
    /// Identifier for a Commitment Device.
    CommitmentDeviceId
);
typed_id!(
    /// Identifier for a Journal Access Grant.
    JournalAccessGrantId
);
typed_id!(
    /// Identifier for a Context Snapshot.
    ContextSnapshotId
);
typed_id!(
    /// Identifier for a Recommendation.
    RecommendationId
);
typed_id!(
    /// Identifier for an Alternative.
    AlternativeId
);
typed_id!(
    /// Identifier for Content Registry content.
    ContentRegistryId
);
typed_id!(
    /// Identifier for a Domain Profile.
    DomainProfileId
);
typed_id!(
    /// Identifier for an Event.
    EventId
);
typed_id!(
    /// Identifier for a durable, inactive Path Draft.
    PathDraftId
);
typed_id!(
    /// Identifier for an immutable AI Proposal.
    AIProposalId
);
typed_id!(
    /// Identifier for a Personal Pattern.
    PersonalPatternId
);
typed_id!(
    /// Identifier for a Path Template import receipt.
    PathTemplateImportId
);
typed_id!(
    /// Identifier for a concrete Integration Binding.
    IntegrationBindingId
);
typed_id!(
    /// Identifier for a stable Flourish account/user identity.
    UserId
);
typed_id!(
    /// Identifier for a Personal or Shared ownership Space.
    SpaceId
);
typed_id!(
    /// Identifier for a user's Membership in a Space.
    MembershipId
);
typed_id!(
    /// Identifier for a registered client Device.
    DeviceId
);
typed_id!(
    /// Identifier for an authenticated Device Session.
    AuthSessionId
);
typed_id!(
    /// Identifier for an authentication-provider identity binding.
    AuthIdentityBindingId
);
typed_id!(
    /// Identifier for a single-use account recovery challenge.
    RecoveryChallengeId
);

/// Exact SHA-256 digest used by canonical contract payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bytes32([u8; 32]);

impl Bytes32 {
    /// Wraps an exact 32-byte digest.
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the digest bytes.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Exact UTC timestamp used by durable Flourish records.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(String);

impl Timestamp {
    /// Parses `YYYY-MM-DDTHH:MM:SS.sssZ` and validates basic calendar/time ranges.
    pub fn new(value: impl Into<String>) -> Result<Self, TimestampError> {
        let value = value.into();
        let bytes = value.as_bytes();
        let separators = [
            (4, b'-'),
            (7, b'-'),
            (10, b'T'),
            (13, b':'),
            (16, b':'),
            (19, b'.'),
            (23, b'Z'),
        ];
        if bytes.len() != 24
            || separators
                .iter()
                .any(|(index, expected)| bytes[*index] != *expected)
        {
            return Err(TimestampError);
        }
        if bytes
            .iter()
            .enumerate()
            .filter(|(index, _)| !separators.iter().any(|(separator, _)| separator == index))
            .any(|(_, byte)| !byte.is_ascii_digit())
        {
            return Err(TimestampError);
        }
        let parse = |range: std::ops::Range<usize>| -> Result<u32, TimestampError> {
            value[range].parse().map_err(|_| TimestampError)
        };
        let month = parse(5..7)?;
        let day = parse(8..10)?;
        let hour = parse(11..13)?;
        let minute = parse(14..16)?;
        let second = parse(17..19)?;
        if !(1..=12).contains(&month)
            || !(1..=31).contains(&day)
            || hour > 23
            || minute > 59
            || second > 59
        {
            return Err(TimestampError);
        }
        Ok(Self(value))
    }

    /// Returns the canonical UTC timestamp string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Error returned for a non-canonical UTC timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimestampError;

impl Display for TimestampError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("timestamp must use YYYY-MM-DDTHH:MM:SS.sssZ with valid ranges")
    }
}

impl Error for TimestampError {}

/// A strict `MAJOR.MINOR.PATCH` semantic version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemVer(String);

impl SemVer {
    /// Parses a three-component numeric semantic version.
    pub fn new(value: impl Into<String>) -> Result<Self, SemVerError> {
        let value = value.into();
        let components: Vec<&str> = value.split('.').collect();
        if components.len() != 3
            || components.iter().any(|component| {
                component.is_empty()
                    || !component.bytes().all(|byte| byte.is_ascii_digit())
                    || (component.len() > 1 && component.starts_with('0'))
            })
        {
            return Err(SemVerError);
        }
        Ok(Self(value))
    }

    /// Returns the exact semantic-version string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Error returned for a non-canonical semantic version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemVerError;

impl Display for SemVerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("semantic version must use canonical MAJOR.MINOR.PATCH form")
    }
}

impl Error for SemVerError {}

/// Declarative, non-executable typed value used by contract fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralValue {
    /// Explicit absence.
    Null,
    /// Boolean value.
    Boolean(bool),
    /// Signed integer.
    Integer(i64),
    /// Unsigned integer.
    Unsigned(u64),
    /// Decimal encoded canonically by the owning schema.
    Decimal(String),
    /// Text value.
    Text(String),
    /// Ordered list.
    List(Vec<LiteralValue>),
    /// String-keyed object.
    Object(BTreeMap<String, LiteralValue>),
}

/// Typed reference to a kernel or pack-namespaced entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedRef {
    /// Pursuit subject.
    Pursuit(PursuitId),
    /// Practice Item subject.
    PracticeItem(PracticeItemId),
    /// Session subject.
    Session(SessionId),
    /// Attempt subject.
    Attempt(AttemptId),
    /// Observation subject.
    Observation(ObservationId),
    /// Decision Record subject.
    DecisionRecord(DecisionRecordId),
    /// Reflection Outcome subject.
    ReflectionOutcome(ReflectionOutcomeId),
    /// Calibration Record subject.
    CalibrationRecord(CalibrationRecordId),
    /// Commitment Device subject.
    CommitmentDevice(CommitmentDeviceId),
    /// Inactive user-owned Path Draft subject.
    PathDraft(PathDraftId),
    /// Immutable AI Proposal subject.
    AIProposal(AIProposalId),
    /// Personal Pattern subject.
    PersonalPattern(PersonalPatternId),
    /// Pack-owned extension with an explicit namespaced kind and ULID.
    PackOwned { kind: PackEntityKind, id: Id },
}

/// Scope supported by a Commitment Device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeRef {
    /// A single Pursuit.
    Pursuit(PursuitId),
    /// A single Session.
    Session(SessionId),
    /// A single Practice Item.
    PracticeItem(PracticeItemId),
    /// The user's whole composed Flourish system.
    Global,
}
