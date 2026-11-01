//! Product V2 additive contracts.
//!
//! These types are pure and storage-independent. A Personal Path is always
//! identified by its existing [`PursuitId`] plus a version;
//! this module deliberately defines no independent Path identifier or Path
//! lifecycle.

use crate::{
    AIProposalId, Bytes32, CalibrationRecordId, ContentRegistryId, DecisionKind, DecisionOrigin,
    DecisionOutcome, DecisionRecord, DecisionRecordId, DerivationRef, DesignHypothesisId,
    EventTopic, EventTopicDefinition, EvidenceClaimId, FactCategory, IdempotencyKey,
    IntegrationBindingId, IntegrationCapabilityId, IntegrationOperationId, LiteralValue, MethodId,
    ObservationId, OwningRef, PackId, PathCompositionAction, PathDraftId, PathTemplateImportId,
    PersonalPatternId, ProtocolAlternative, ProtocolId, ProtocolResponseChoice, PursuitId,
    RecommendationRef, ReflectionOutcomeId, SemVer, SensitivityClass, ServiceId, SessionId,
    Timestamp, TypedRef,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Epistemic classification preserved across deterministic and AI boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpistemicClass {
    /// Explicit user statement or authoritative source.
    Known,
    /// Recorded event or data.
    Observed,
    /// Derived interpretation.
    Inferred,
    /// Testable explanation.
    Hypothesis,
    /// Proposed action.
    Recommendation,
}

/// One Path-scoped operating state; never a global person rank or score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingState {
    Stabilize,
    GainControl,
    BuildCapacity,
    Explore,
    Pursue,
    Master,
    Optimize,
    Maintain,
    Recover,
}

/// Exact catalog Method reference.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersionedMethodRef {
    pub method_id: MethodId,
    pub method_version: SemVer,
}

/// Exact immutable Protocol reference.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersionedProtocolRef {
    pub protocol_id: ProtocolId,
    pub protocol_version: SemVer,
}

/// Exact Integration Capability reference.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersionedIntegrationCapabilityRef {
    pub capability_id: IntegrationCapabilityId,
    pub capability_version: SemVer,
}

/// Capability identity owned by a Capability Pack.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackOwnedCapabilityRef {
    pub owning_pack_ref: PackId,
    pub capability_key: String,
}

impl PackOwnedCapabilityRef {
    /// Creates a pack-owned reference after rejecting an empty capability key.
    pub fn new(
        owning_pack_ref: PackId,
        capability_key: impl Into<String>,
    ) -> Result<Self, PathViolation> {
        let capability_key = capability_key.into();
        ensure_text("capability_key", &capability_key)?;
        Ok(Self {
            owning_pack_ref,
            capability_key,
        })
    }

    /// Revalidates a value reconstructed at an I/O boundary.
    pub fn validate(&self) -> Result<(), PathViolation> {
        ensure_text("capability_key", &self.capability_key)
    }
}

/// Optional broad life context; it is not a goal or lifecycle entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifeAreaContext {
    pub key: String,
    pub label: String,
}

impl LifeAreaContext {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Result<Self, PathViolation> {
        let key = key.into();
        let label = label.into();
        ensure_text("life_area.key", &key)?;
        ensure_text("life_area.label", &label)?;
        Ok(Self { key, label })
    }

    /// Revalidates a value reconstructed at an I/O boundary.
    pub fn validate(&self) -> Result<(), PathViolation> {
        ensure_text("life_area.key", &self.key)?;
        ensure_text("life_area.label", &self.label)
    }
}

/// Stability stance of a user-owned Direction statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionStance {
    Stable,
    Provisional,
    Exploratory,
}

/// User-owned Direction value embedded in one Path composition version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectionSpec {
    pub statement: String,
    pub stance: DirectionStance,
    pub life_area: Option<LifeAreaContext>,
}

impl DirectionSpec {
    pub fn new(
        statement: impl Into<String>,
        stance: DirectionStance,
        life_area: Option<LifeAreaContext>,
    ) -> Result<Self, PathViolation> {
        let statement = statement.into();
        ensure_text("direction.statement", &statement)?;
        Ok(Self {
            statement,
            stance,
            life_area,
        })
    }

    /// Revalidates a value reconstructed at an I/O boundary.
    pub fn validate(&self) -> Result<(), PathViolation> {
        ensure_text("direction.statement", &self.statement)?;
        if let Some(life_area) = &self.life_area {
            life_area.validate()?;
        }
        Ok(())
    }
}

/// Provenance permitted for a user-owned Path constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathConstraintProvenance {
    UserStated,
    AuthoritativeSource,
    PersonalObservation,
}

/// Constraint carried by one immutable Path version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathConstraint {
    pub key: String,
    pub value: LiteralValue,
    pub provenance: PathConstraintProvenance,
    pub provenance_ref: Option<TypedRef>,
}

impl PathConstraint {
    pub fn new(
        key: impl Into<String>,
        value: LiteralValue,
        provenance: PathConstraintProvenance,
        provenance_ref: Option<TypedRef>,
    ) -> Result<Self, PathViolation> {
        let result = Self {
            key: key.into(),
            value,
            provenance,
            provenance_ref,
        };
        result.validate()?;
        Ok(result)
    }

    /// Revalidates provenance after decoding or composing nested values.
    pub fn validate(&self) -> Result<(), PathViolation> {
        ensure_text("constraint.key", &self.key)?;
        match (self.provenance, &self.provenance_ref) {
            (PathConstraintProvenance::UserStated, None) => {}
            (PathConstraintProvenance::UserStated, Some(_)) => {
                return Err(PathViolation::UnexpectedConstraintProvenanceRef)
            }
            (PathConstraintProvenance::AuthoritativeSource, Some(_))
            | (PathConstraintProvenance::PersonalObservation, Some(_)) => {}
            _ => return Err(PathViolation::MissingConstraintProvenanceRef),
        }
        if self.provenance == PathConstraintProvenance::PersonalObservation
            && !matches!(self.provenance_ref, Some(TypedRef::Observation(_)))
        {
            return Err(PathViolation::PersonalConstraintRequiresObservation);
        }
        Ok(())
    }
}

/// User preference for explanation detail on one Path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplanationPreference {
    Brief,
    Standard,
    Detailed,
}

/// Complete source-independent composition for one Pursuit-anchored Path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathComposition {
    composition_schema_version: u32,
    direction: DirectionSpec,
    capability_refs: Vec<PackOwnedCapabilityRef>,
    method_refs: Vec<VersionedMethodRef>,
    protocol_refs: Vec<VersionedProtocolRef>,
    integration_capability_refs: Vec<VersionedIntegrationCapabilityRef>,
    experiment_refs: Vec<SessionId>,
    operating_state: OperatingState,
    constraints: Vec<PathConstraint>,
    user_overrides: BTreeMap<String, LiteralValue>,
    explanation_preference: ExplanationPreference,
}

impl PathComposition {
    /// Builds a composition using the same validator for manual, template, and AI origins.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        composition_schema_version: u32,
        direction: DirectionSpec,
        capability_refs: Vec<PackOwnedCapabilityRef>,
        method_refs: Vec<VersionedMethodRef>,
        protocol_refs: Vec<VersionedProtocolRef>,
        integration_capability_refs: Vec<VersionedIntegrationCapabilityRef>,
        experiment_refs: Vec<SessionId>,
        operating_state: OperatingState,
        constraints: Vec<PathConstraint>,
        user_overrides: BTreeMap<String, LiteralValue>,
        explanation_preference: ExplanationPreference,
    ) -> Result<Self, PathViolation> {
        if composition_schema_version == 0 {
            return Err(PathViolation::ZeroSchemaVersion);
        }
        direction.validate()?;
        for reference in &capability_refs {
            reference.validate()?;
        }
        ensure_unique(&capability_refs, "capability_refs")?;
        ensure_unique(&method_refs, "method_refs")?;
        ensure_unique(&protocol_refs, "protocol_refs")?;
        ensure_unique(&integration_capability_refs, "integration_capability_refs")?;
        ensure_unique(&experiment_refs, "experiment_refs")?;
        let mut constraint_keys = BTreeSet::new();
        for constraint in &constraints {
            constraint.validate()?;
            if !constraint_keys.insert(constraint.key.as_str()) {
                return Err(PathViolation::DuplicateEntry("constraints"));
            }
        }
        if user_overrides.keys().any(|key| key.trim().is_empty()) {
            return Err(PathViolation::EmptyText("user_overrides.key"));
        }
        Ok(Self {
            composition_schema_version,
            direction,
            capability_refs,
            method_refs,
            protocol_refs,
            integration_capability_refs,
            experiment_refs,
            operating_state,
            constraints,
            user_overrides,
            explanation_preference,
        })
    }

    pub const fn composition_schema_version(&self) -> u32 {
        self.composition_schema_version
    }

    pub fn direction(&self) -> &DirectionSpec {
        &self.direction
    }

    pub fn capability_refs(&self) -> &[PackOwnedCapabilityRef] {
        &self.capability_refs
    }

    pub fn method_refs(&self) -> &[VersionedMethodRef] {
        &self.method_refs
    }

    pub fn protocol_refs(&self) -> &[VersionedProtocolRef] {
        &self.protocol_refs
    }

    pub fn integration_capability_refs(&self) -> &[VersionedIntegrationCapabilityRef] {
        &self.integration_capability_refs
    }

    pub fn experiment_refs(&self) -> &[SessionId] {
        &self.experiment_refs
    }

    pub const fn operating_state(&self) -> OperatingState {
        self.operating_state
    }

    pub fn constraints(&self) -> &[PathConstraint] {
        &self.constraints
    }

    pub fn user_overrides(&self) -> &BTreeMap<String, LiteralValue> {
        &self.user_overrides
    }

    pub const fn explanation_preference(&self) -> ExplanationPreference {
        self.explanation_preference
    }

    /// Fails closed on every catalog/user reference before an acceptance transaction.
    pub fn validate_references(
        &self,
        registry: &dyn PathReferenceRegistry,
    ) -> Result<(), PathReferenceViolation> {
        for reference in &self.capability_refs {
            if !registry.has_capability(reference) {
                return Err(PathReferenceViolation::UnknownCapability(reference.clone()));
            }
        }
        for reference in &self.method_refs {
            if !registry.has_method(reference) {
                return Err(PathReferenceViolation::UnknownMethod(reference.clone()));
            }
        }
        for reference in &self.protocol_refs {
            if !registry.has_protocol(reference) {
                return Err(PathReferenceViolation::UnknownProtocol(reference.clone()));
            }
        }
        for reference in &self.integration_capability_refs {
            if !registry.has_integration(reference) {
                return Err(PathReferenceViolation::UnknownIntegration(
                    reference.clone(),
                ));
            }
        }
        for reference in &self.experiment_refs {
            if !registry.has_experiment(reference) {
                return Err(PathReferenceViolation::UnknownExperiment(reference.clone()));
            }
        }
        Ok(())
    }

    /// Computes the SHA-256 digest of a deterministic, length-prefixed encoding.
    pub fn sha256(&self) -> Bytes32 {
        digest(&self.canonical_bytes())
    }

    fn canonical_bytes(&self) -> Vec<u8> {
        let mut output = Vec::new();
        push_u64(&mut output, u64::from(self.composition_schema_version));
        encode_direction(&mut output, &self.direction);
        push_len(&mut output, self.capability_refs.len());
        for reference in &self.capability_refs {
            push_str(&mut output, reference.owning_pack_ref.as_str());
            push_str(&mut output, &reference.capability_key);
        }
        push_len(&mut output, self.method_refs.len());
        for reference in &self.method_refs {
            encode_method_ref(&mut output, reference);
        }
        push_len(&mut output, self.protocol_refs.len());
        for reference in &self.protocol_refs {
            encode_protocol_ref(&mut output, reference);
        }
        push_len(&mut output, self.integration_capability_refs.len());
        for reference in &self.integration_capability_refs {
            encode_integration_ref(&mut output, reference);
        }
        push_len(&mut output, self.experiment_refs.len());
        for id in &self.experiment_refs {
            push_str(&mut output, id.id().as_str());
        }
        output.push(self.operating_state as u8);
        push_len(&mut output, self.constraints.len());
        for constraint in &self.constraints {
            push_str(&mut output, &constraint.key);
            encode_literal(&mut output, &constraint.value);
            output.push(constraint.provenance as u8);
            encode_optional_typed_ref(&mut output, constraint.provenance_ref.as_ref());
        }
        push_len(&mut output, self.user_overrides.len());
        for (key, value) in &self.user_overrides {
            push_str(&mut output, key);
            encode_literal(&mut output, value);
        }
        output.push(self.explanation_preference as u8);
        output
    }
}

/// Invalid Path composition, draft, template, delta, or accepted version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathViolation {
    EmptyText(&'static str),
    ZeroSchemaVersion,
    DuplicateEntry(&'static str),
    MissingConstraintProvenanceRef,
    UnexpectedConstraintProvenanceRef,
    PersonalConstraintRequiresObservation,
    InvalidPathVersionChain,
    AcceptanceDecisionMismatch,
    DraftOriginMismatch,
    ClosedDraftCannotReceiveRevision,
    InvalidDraftRevision,
    InvalidTemplateReceipt,
    EmptyDelta,
    InvalidDeltaBase,
    /// A delta operation named an entry the base composition does not contain.
    DeltaTargetNotFound(&'static str),
    /// The cited AI proposal was already expired when the Path was accepted.
    AiProposalExpired,
    /// The cited AI proposal does not carry a Path delta at all.
    AiProposalWrongPayloadKind,
    /// The cited AI proposal was computed against a different Path version than the one accepted onto.
    AiProposalStaleBase,
    /// Accepting an AI proposal requires the composition it was computed against.
    MissingPreviousComposition,
    /// The accepted composition is not what applying the proposal's delta produces.
    AiProposalCompositionMismatch,
}

impl Display for PathViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyText(field) => write!(formatter, "{field} must not be empty"),
            Self::ZeroSchemaVersion => formatter.write_str("schema version must be greater than zero"),
            Self::DuplicateEntry(field) => write!(formatter, "{field} contains a duplicate"),
            Self::MissingConstraintProvenanceRef => formatter.write_str(
                "authoritative-source and personal-observation constraints require provenance_ref",
            ),
            Self::UnexpectedConstraintProvenanceRef => formatter
                .write_str("user-stated constraints must not carry provenance_ref"),
            Self::PersonalConstraintRequiresObservation => formatter
                .write_str("personal-observation constraint must cite an Observation"),
            Self::DeltaTargetNotFound(field) => {
                write!(formatter, "a delta operation targets a missing {field} entry")
            }
            Self::AiProposalExpired => {
                formatter.write_str("the cited AI proposal was already expired at acceptance time")
            }
            Self::AiProposalWrongPayloadKind => {
                formatter.write_str("the cited AI proposal does not carry a Path delta")
            }
            Self::AiProposalStaleBase => formatter.write_str(
                "the cited AI proposal was computed against a different Path version",
            ),
            Self::MissingPreviousComposition => formatter.write_str(
                "accepting an AI proposal requires the composition its delta was computed against",
            ),
            Self::AiProposalCompositionMismatch => formatter.write_str(
                "the accepted composition is not what applying the AI proposal's delta produces",
            ),
            Self::InvalidPathVersionChain => formatter.write_str(
                "path versions start at one and previous_path_version must be the immediate predecessor",
            ),
            Self::AcceptanceDecisionMismatch => formatter.write_str(
                "accepted Path version requires a matching explicit Path or AI-proposal decision",
            ),
            Self::DraftOriginMismatch => formatter
                .write_str("draft origin does not match its template/proposal provenance"),
            Self::ClosedDraftCannotReceiveRevision => {
                formatter.write_str("closed Path draft cannot receive another revision")
            }
            Self::InvalidDraftRevision => {
                formatter.write_str("draft revision must be positive and internally consistent")
            }
            Self::InvalidTemplateReceipt => formatter
                .write_str("Path template import receipt requires source and provenance"),
            Self::EmptyDelta => formatter.write_str("Path delta must contain at least one change"),
            Self::InvalidDeltaBase => {
                formatter.write_str("Path delta base version must be greater than zero")
            }
        }
    }
}

impl Error for PathViolation {}

/// Read-only reference resolver used by the local acceptance validator.
pub trait PathReferenceRegistry {
    fn has_capability(&self, reference: &PackOwnedCapabilityRef) -> bool;
    fn has_method(&self, reference: &VersionedMethodRef) -> bool;
    fn has_protocol(&self, reference: &VersionedProtocolRef) -> bool;
    fn has_integration(&self, reference: &VersionedIntegrationCapabilityRef) -> bool;
    fn has_experiment(&self, reference: &SessionId) -> bool;
}

/// Unknown references reject instead of being silently omitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathReferenceViolation {
    UnknownCapability(PackOwnedCapabilityRef),
    UnknownMethod(VersionedMethodRef),
    UnknownProtocol(VersionedProtocolRef),
    UnknownIntegration(VersionedIntegrationCapabilityRef),
    UnknownExperiment(SessionId),
}

impl Display for PathReferenceViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("Path composition contains an unresolved reference")
    }
}

impl Error for PathReferenceViolation {}

/// Source revision from an inactive draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathDraftRevisionRef {
    pub draft_id: PathDraftId,
    pub revision: u64,
}

/// Manifest reference to the one accepted composition version for a Pursuit.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathCompositionRef {
    pub pursuit_ref: PursuitId,
    pub path_version: u64,
}

impl PathCompositionRef {
    pub fn new(pursuit_ref: PursuitId, path_version: u64) -> Result<Self, PathViolation> {
        if path_version == 0 {
            return Err(PathViolation::InvalidPathVersionChain);
        }
        Ok(Self {
            pursuit_ref,
            path_version,
        })
    }

    /// Validates a reference reconstructed without the smart constructor.
    pub fn validate(&self) -> Result<(), PathViolation> {
        if self.path_version == 0 {
            Err(PathViolation::InvalidPathVersionChain)
        } else {
            Ok(())
        }
    }
}

/// Immutable accepted composition anchored to one existing Pursuit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PursuitPathVersion {
    pursuit_ref: PursuitId,
    path_version: u64,
    previous_path_version: Option<u64>,
    composition: PathComposition,
    composition_sha256: Bytes32,
    accepted_decision_ref: DecisionRecordId,
    source_draft_revision_ref: Option<PathDraftRevisionRef>,
    source_proposal_ref: Option<AIProposalId>,
    accepted_command_id: IdempotencyKey,
    accepted_at: Timestamp,
}

/// Recomputes the composition a Path delta produces from an exact base.
///
/// This is a pure function over two immutable values: it writes nothing and has no lifecycle. It
/// exists so that accepting an AI proposal can be checked rather than trusted — the caller-supplied
/// composition must equal what the proposal's own delta produces from the previous composition.
pub fn apply_path_delta(
    base: &PathComposition,
    delta: &PathDelta,
) -> Result<PathComposition, PathViolation> {
    delta.validate()?;
    let mut direction = base.direction().clone();
    let mut capability_refs = base.capability_refs().to_vec();
    let mut method_refs = base.method_refs().to_vec();
    let mut protocol_refs = base.protocol_refs().to_vec();
    let mut integration_capability_refs = base.integration_capability_refs().to_vec();
    let mut experiment_refs = base.experiment_refs().to_vec();
    let mut operating_state = base.operating_state();
    let mut constraints = base.constraints().to_vec();
    let mut user_overrides = base.user_overrides().clone();
    let mut explanation_preference = base.explanation_preference();

    for operation in &delta.operations {
        match operation {
            PathChange::SetDirection(spec) => {
                spec.validate()?;
                direction = spec.clone();
            }
            PathChange::SetOperatingState(state) => operating_state = *state,
            PathChange::AddCapability(reference) => {
                reference.validate()?;
                if capability_refs.contains(reference) {
                    return Err(PathViolation::DuplicateEntry("capability_refs"));
                }
                capability_refs.push(reference.clone());
            }
            PathChange::RemoveCapability(reference) => {
                let before = capability_refs.len();
                capability_refs.retain(|existing| existing != reference);
                if capability_refs.len() == before {
                    return Err(PathViolation::DeltaTargetNotFound("capability_refs"));
                }
            }
            PathChange::ReorderCapability(reference, index) => {
                let position = capability_refs
                    .iter()
                    .position(|existing| existing == reference)
                    .ok_or(PathViolation::DeltaTargetNotFound("capability_refs"))?;
                let item = capability_refs.remove(position);
                let index = usize::try_from(*index)
                    .unwrap_or(usize::MAX)
                    .min(capability_refs.len());
                capability_refs.insert(index, item);
            }
            PathChange::AddMethod(reference) => {
                if method_refs.contains(reference) {
                    return Err(PathViolation::DuplicateEntry("method_refs"));
                }
                method_refs.push(reference.clone());
            }
            PathChange::RemoveMethod(reference) => {
                let before = method_refs.len();
                method_refs.retain(|existing| existing != reference);
                if method_refs.len() == before {
                    return Err(PathViolation::DeltaTargetNotFound("method_refs"));
                }
            }
            PathChange::AddProtocol(reference) => {
                if protocol_refs.contains(reference) {
                    return Err(PathViolation::DuplicateEntry("protocol_refs"));
                }
                protocol_refs.push(reference.clone());
            }
            PathChange::RemoveProtocol(reference) => {
                let before = protocol_refs.len();
                protocol_refs.retain(|existing| existing != reference);
                if protocol_refs.len() == before {
                    return Err(PathViolation::DeltaTargetNotFound("protocol_refs"));
                }
            }
            PathChange::AddIntegration(reference) => {
                if integration_capability_refs.contains(reference) {
                    return Err(PathViolation::DuplicateEntry("integration_capability_refs"));
                }
                integration_capability_refs.push(reference.clone());
            }
            PathChange::RemoveIntegration(reference) => {
                let before = integration_capability_refs.len();
                integration_capability_refs.retain(|existing| existing != reference);
                if integration_capability_refs.len() == before {
                    return Err(PathViolation::DeltaTargetNotFound(
                        "integration_capability_refs",
                    ));
                }
            }
            PathChange::AddExperiment(reference) => {
                if experiment_refs.contains(reference) {
                    return Err(PathViolation::DuplicateEntry("experiment_refs"));
                }
                experiment_refs.push(reference.clone());
            }
            PathChange::RemoveExperiment(reference) => {
                let before = experiment_refs.len();
                experiment_refs.retain(|existing| existing != reference);
                if experiment_refs.len() == before {
                    return Err(PathViolation::DeltaTargetNotFound("experiment_refs"));
                }
            }
            PathChange::SetConstraint(constraint) => {
                constraint.validate()?;
                constraints.retain(|existing| existing.key != constraint.key);
                constraints.push(constraint.clone());
            }
            PathChange::RemoveConstraint(key) => {
                let before = constraints.len();
                constraints.retain(|existing| &existing.key != key);
                if constraints.len() == before {
                    return Err(PathViolation::DeltaTargetNotFound("constraints"));
                }
            }
            PathChange::SetOverride(key, value) => {
                user_overrides.insert(key.clone(), value.clone());
            }
            PathChange::RemoveOverride(key) => {
                if user_overrides.remove(key).is_none() {
                    return Err(PathViolation::DeltaTargetNotFound("user_overrides"));
                }
            }
            PathChange::SetExplanationPreference(preference) => {
                explanation_preference = *preference;
            }
        }
    }

    PathComposition::new(
        base.composition_schema_version(),
        direction,
        capability_refs,
        method_refs,
        protocol_refs,
        integration_capability_refs,
        experiment_refs,
        operating_state,
        constraints,
        user_overrides,
        explanation_preference,
    )
}

impl PursuitPathVersion {
    /// Constructs one accepted immutable version after checking user-decision authority.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pursuit_ref: PursuitId,
        path_version: u64,
        previous_path_version: Option<u64>,
        composition: PathComposition,
        // The composition this version is accepted on top of. Required when a proposal is
        // cited, because its delta can only be checked against the exact base it came from.
        previous_composition: Option<&PathComposition>,
        acceptance_decision: &DecisionRecord,
        source_draft_revision_ref: Option<PathDraftRevisionRef>,
        // The proposal itself, not just its id: an id alone cannot be checked for expiry,
        // payload kind, or whether the accepted composition is the change that was proposed.
        source_proposal: Option<&AIProposal>,
        accepted_command_id: IdempotencyKey,
        accepted_at: Timestamp,
    ) -> Result<Self, PathViolation> {
        let valid_chain = match (path_version, previous_path_version) {
            (1, None) => true,
            (version, Some(previous)) if version > 1 && previous + 1 == version => true,
            _ => false,
        };
        if !valid_chain {
            return Err(PathViolation::InvalidPathVersionChain);
        }
        if source_draft_revision_ref
            .as_ref()
            .is_some_and(|reference| reference.revision == 0)
        {
            return Err(PathViolation::InvalidDraftRevision);
        }
        if !matches!(acceptance_decision.subject_ref(), TypedRef::Pursuit(id) if id == &pursuit_ref)
        {
            return Err(PathViolation::AcceptanceDecisionMismatch);
        }
        // An accepted AI proposal must be exactly the change that was proposed and reviewed. An id
        // alone proves none of that, so the proposal must (a) carry a Path delta, (b) not be
        // expired at acceptance time, (c) be based on the exact Path version being accepted onto,
        // and (d) recompute the accepted composition from the previous one. Without (d) a caller
        // could pair any accepted AiProposalResponse decision with an arbitrary composition.
        if let Some(proposal) = source_proposal {
            if proposal.is_expired_at(&accepted_at) {
                return Err(PathViolation::AiProposalExpired);
            }
            let AIProposalPayload::PathDelta(delta) = proposal.payload() else {
                return Err(PathViolation::AiProposalWrongPayloadKind);
            };
            let current_version =
                previous_path_version.ok_or(PathViolation::AiProposalStaleBase)?;
            if delta.base_path_version != current_version {
                return Err(PathViolation::AiProposalStaleBase);
            }
            let base_composition =
                previous_composition.ok_or(PathViolation::MissingPreviousComposition)?;
            if apply_path_delta(base_composition, delta)? != composition {
                return Err(PathViolation::AiProposalCompositionMismatch);
            }
        }
        let source_proposal_ref = source_proposal.map(|proposal| proposal.id().clone());
        let decision_matches = match source_proposal_ref.as_ref() {
            Some(proposal_id) => matches!(
                (
                    acceptance_decision.origin(),
                    acceptance_decision.decision_kind(),
                    acceptance_decision.outcome(),
                    acceptance_decision.recommendation_ref(),
                ),
                (
                    DecisionOrigin::SystemRecommended,
                    DecisionKind::AiProposalResponse,
                    DecisionOutcome::AiProposalResponse(
                        ProtocolResponseChoice::Accept
                            | ProtocolResponseChoice::Alternative
                            | ProtocolResponseChoice::Modify
                    ),
                    Some(RecommendationRef::AiProposal(id)),
                ) if id == proposal_id
            ),
            None => matches!(
                (
                    acceptance_decision.origin(),
                    acceptance_decision.decision_kind(),
                    acceptance_decision.outcome(),
                ),
                (
                    DecisionOrigin::UserInitiated,
                    DecisionKind::PathCompositionChange,
                    DecisionOutcome::PathCompositionChange(
                        PathCompositionAction::Accept | PathCompositionAction::Modify
                    ),
                )
            ),
        };
        if !decision_matches {
            return Err(PathViolation::AcceptanceDecisionMismatch);
        }
        let composition_sha256 = composition.sha256();
        Ok(Self {
            pursuit_ref,
            path_version,
            previous_path_version,
            composition,
            composition_sha256,
            accepted_decision_ref: acceptance_decision.id().clone(),
            source_draft_revision_ref,
            source_proposal_ref,
            accepted_command_id,
            accepted_at,
        })
    }

    pub fn pursuit_ref(&self) -> &PursuitId {
        &self.pursuit_ref
    }

    pub const fn path_version(&self) -> u64 {
        self.path_version
    }

    pub const fn previous_path_version(&self) -> Option<u64> {
        self.previous_path_version
    }

    pub fn composition(&self) -> &PathComposition {
        &self.composition
    }

    pub const fn composition_sha256(&self) -> Bytes32 {
        self.composition_sha256
    }

    pub fn accepted_decision_ref(&self) -> &DecisionRecordId {
        &self.accepted_decision_ref
    }

    pub fn source_draft_revision_ref(&self) -> Option<&PathDraftRevisionRef> {
        self.source_draft_revision_ref.as_ref()
    }

    pub fn source_proposal_ref(&self) -> Option<&AIProposalId> {
        self.source_proposal_ref.as_ref()
    }

    pub fn accepted_command_id(&self) -> &IdempotencyKey {
        &self.accepted_command_id
    }

    pub fn accepted_at(&self) -> &Timestamp {
        &self.accepted_at
    }
}

/// The sole Product V2 event-topic addition frozen by Core Contracts v1.4.
pub const PURSUIT_PATH_CHANGED_TOPIC: &str = "pursuit.path_changed";

/// Typed payload for the accepted Path-version transition event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PursuitPathChanged {
    pub pursuit_ref: PursuitId,
    pub old_path_version: Option<u64>,
    pub new_path_version: u64,
    pub decision_ref: DecisionRecordId,
    pub source_draft_revision_ref: Option<PathDraftRevisionRef>,
    pub source_proposal_ref: Option<AIProposalId>,
}

impl PursuitPathChanged {
    /// Derives the event payload only from an already validated accepted Path version.
    pub fn from_accepted_version(version: &PursuitPathVersion) -> Self {
        Self {
            pursuit_ref: version.pursuit_ref().clone(),
            old_path_version: version.previous_path_version(),
            new_path_version: version.path_version(),
            decision_ref: version.accepted_decision_ref().clone(),
            source_draft_revision_ref: version.source_draft_revision_ref().cloned(),
            source_proposal_ref: version.source_proposal_ref().cloned(),
        }
    }

    /// Produces the closed, identifier-only Event payload shape used by the event log.
    pub fn to_literal_value(&self) -> LiteralValue {
        let mut payload = BTreeMap::from([
            (
                "pursuit_ref".into(),
                LiteralValue::Text(self.pursuit_ref.id().as_str().into()),
            ),
            (
                "new_path_version".into(),
                LiteralValue::Unsigned(self.new_path_version),
            ),
            (
                "decision_ref".into(),
                LiteralValue::Text(self.decision_ref.id().as_str().into()),
            ),
        ]);
        payload.insert(
            "old_path_version".into(),
            self.old_path_version
                .map_or(LiteralValue::Null, LiteralValue::Unsigned),
        );
        payload.insert(
            "source_draft_revision_ref".into(),
            self.source_draft_revision_ref
                .as_ref()
                .map_or(LiteralValue::Null, |reference| {
                    LiteralValue::Object(BTreeMap::from([
                        (
                            "draft_id".into(),
                            LiteralValue::Text(reference.draft_id.id().as_str().into()),
                        ),
                        (
                            "revision".into(),
                            LiteralValue::Unsigned(reference.revision),
                        ),
                    ]))
                }),
        );
        payload.insert(
            "source_proposal_ref".into(),
            self.source_proposal_ref
                .as_ref()
                .map_or(LiteralValue::Null, |id| {
                    LiteralValue::Text(id.id().as_str().into())
                }),
        );
        LiteralValue::Object(payload)
    }
}

/// Canonical registry definition for `pursuit.path_changed`.
///
/// Its operational category and absent operational basis make it ineligible
/// for both technical-INVARIANT triggers and Commitment Device conditions.
pub fn pursuit_path_changed_topic_definition() -> EventTopicDefinition {
    EventTopicDefinition::new(
        EventTopic::new(PURSUIT_PATH_CHANGED_TOPIC)
            .expect("the frozen pursuit.path_changed topic is non-empty"),
        1,
        OwningRef::KernelService(ServiceId::ManifestResolver),
        FactCategory::Operational,
        None,
    )
    .expect("the frozen pursuit.path_changed topic definition is valid")
}

/// Provenance class of a Path Draft.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathDraftOrigin {
    Manual,
    AiProposal,
    Template,
    Hybrid,
}

/// Durable but inactive user-owned composition workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathDraft {
    pub id: PathDraftId,
    pub pursuit_ref: Option<PursuitId>,
    pub origin: PathDraftOrigin,
    pub template_import_ref: Option<PathTemplateImportId>,
    pub created_by_proposal_ref: Option<AIProposalId>,
    pub created_at: Timestamp,
    pub closed_at: Option<Timestamp>,
}

impl PathDraft {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: PathDraftId,
        pursuit_ref: Option<PursuitId>,
        origin: PathDraftOrigin,
        template_import_ref: Option<PathTemplateImportId>,
        created_by_proposal_ref: Option<AIProposalId>,
        created_at: Timestamp,
        closed_at: Option<Timestamp>,
    ) -> Result<Self, PathViolation> {
        let valid_origin = match origin {
            PathDraftOrigin::Manual => {
                template_import_ref.is_none() && created_by_proposal_ref.is_none()
            }
            PathDraftOrigin::AiProposal => {
                template_import_ref.is_none() && created_by_proposal_ref.is_some()
            }
            PathDraftOrigin::Template => {
                template_import_ref.is_some() && created_by_proposal_ref.is_none()
            }
            PathDraftOrigin::Hybrid => {
                template_import_ref.is_some() || created_by_proposal_ref.is_some()
            }
        };
        if !valid_origin {
            return Err(PathViolation::DraftOriginMismatch);
        }
        Ok(Self {
            id,
            pursuit_ref,
            origin,
            template_import_ref,
            created_by_proposal_ref,
            created_at,
            closed_at,
        })
    }

    pub fn accepts_revision(&self) -> Result<(), PathViolation> {
        self.closed_at
            .is_none()
            .then_some(())
            .ok_or(PathViolation::ClosedDraftCannotReceiveRevision)
    }
}

/// Immutable snapshot of one Path Draft revision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathDraftRevision {
    pub draft_ref: PathDraftId,
    pub revision: u64,
    pub base_path_version: Option<u64>,
    pub composition: PathComposition,
    pub composition_sha256: Bytes32,
    pub created_at: Timestamp,
}

impl PathDraftRevision {
    pub fn new(
        draft: &PathDraft,
        revision: u64,
        base_path_version: Option<u64>,
        composition: PathComposition,
        created_at: Timestamp,
    ) -> Result<Self, PathViolation> {
        draft.accepts_revision()?;
        if revision == 0 || base_path_version == Some(0) {
            return Err(PathViolation::InvalidDraftRevision);
        }
        let composition_sha256 = composition.sha256();
        Ok(Self {
            draft_ref: draft.id.clone(),
            revision,
            base_path_version,
            composition,
            composition_sha256,
            created_at,
        })
    }
}

/// Validation result of an imported, non-activating template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateValidationStatus {
    Valid,
    Rejected,
}

/// Provenance receipt for an optional external/user-provided Path Template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathTemplateImportReceipt {
    pub id: PathTemplateImportId,
    pub source_uri: String,
    pub author_or_publisher: Option<String>,
    pub source_version: Option<String>,
    pub content_sha256: Bytes32,
    pub license_and_provenance: LiteralValue,
    pub validation_status: TemplateValidationStatus,
    pub imported_at: Timestamp,
}

impl PathTemplateImportReceipt {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: PathTemplateImportId,
        source_uri: impl Into<String>,
        author_or_publisher: Option<String>,
        source_version: Option<String>,
        content_sha256: Bytes32,
        license_and_provenance: LiteralValue,
        validation_status: TemplateValidationStatus,
        imported_at: Timestamp,
    ) -> Result<Self, PathViolation> {
        let source_uri = source_uri.into();
        if source_uri.trim().is_empty() || license_and_provenance == LiteralValue::Null {
            return Err(PathViolation::InvalidTemplateReceipt);
        }
        if author_or_publisher
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
            || source_version
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
        {
            return Err(PathViolation::InvalidTemplateReceipt);
        }
        Ok(Self {
            id,
            source_uri,
            author_or_publisher,
            source_version,
            content_sha256,
            license_and_provenance,
            validation_status,
            imported_at,
        })
    }
}

/// One material change proposed for a Personal Path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathChange {
    SetDirection(DirectionSpec),
    SetOperatingState(OperatingState),
    AddCapability(PackOwnedCapabilityRef),
    RemoveCapability(PackOwnedCapabilityRef),
    ReorderCapability(PackOwnedCapabilityRef, u32),
    AddMethod(VersionedMethodRef),
    RemoveMethod(VersionedMethodRef),
    AddProtocol(VersionedProtocolRef),
    RemoveProtocol(VersionedProtocolRef),
    AddIntegration(VersionedIntegrationCapabilityRef),
    RemoveIntegration(VersionedIntegrationCapabilityRef),
    AddExperiment(SessionId),
    RemoveExperiment(SessionId),
    SetConstraint(PathConstraint),
    RemoveConstraint(String),
    SetOverride(String, LiteralValue),
    RemoveOverride(String),
    SetExplanationPreference(ExplanationPreference),
}

/// Pure proposal payload; it has no lifecycle and exposes no apply/write operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathDelta {
    pub pursuit_ref: PursuitId,
    pub base_path_version: u64,
    pub operations: Vec<PathChange>,
}

impl PathDelta {
    pub fn new(
        pursuit_ref: PursuitId,
        base_path_version: u64,
        operations: Vec<PathChange>,
    ) -> Result<Self, PathViolation> {
        let result = Self {
            pursuit_ref,
            base_path_version,
            operations,
        };
        result.validate()?;
        Ok(result)
    }

    /// Revalidates a decoded delta before it can enter a proposal.
    pub fn validate(&self) -> Result<(), PathViolation> {
        if self.base_path_version == 0 {
            return Err(PathViolation::InvalidDeltaBase);
        }
        if self.operations.is_empty() {
            return Err(PathViolation::EmptyDelta);
        }
        for operation in &self.operations {
            match operation {
                PathChange::SetDirection(direction) => direction.validate()?,
                PathChange::AddCapability(reference)
                | PathChange::RemoveCapability(reference)
                | PathChange::ReorderCapability(reference, _) => reference.validate()?,
                PathChange::SetConstraint(constraint) => constraint.validate()?,
                PathChange::RemoveConstraint(key)
                | PathChange::RemoveOverride(key)
                | PathChange::SetOverride(key, _)
                    if key.trim().is_empty() =>
                {
                    return Err(PathViolation::EmptyText("path_change.key"));
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn sha256(&self) -> Bytes32 {
        let mut output = Vec::new();
        push_str(&mut output, self.pursuit_ref.id().as_str());
        push_u64(&mut output, self.base_path_version);
        push_len(&mut output, self.operations.len());
        for operation in &self.operations {
            encode_path_change(&mut output, operation);
        }
        digest(&output)
    }
}

/// Governed evidence link from a Method definition.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MethodEvidenceRef {
    EvidenceClaim(EvidenceClaimId),
    DesignHypothesis(DesignHypothesisId),
}

/// Product-facing evidence characterization without changing governed grades.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodEvidenceStatus {
    Scientific,
    MixedContextDependent,
    DesignHypothesis,
    PractitionerFramework,
    Experimental,
}

/// Declared AI roles a Method can safely use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MethodAiRole {
    Discovery,
    Sparring,
    Tutor,
    Reflection,
    EvidenceExplanation,
    Orchestration,
    PatternAnalysis,
}

/// Versioned product interpretation between governed evidence and Protocol policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodDefinition {
    pub id: MethodId,
    pub version: SemVer,
    pub name: String,
    pub category: String,
    pub target_problem: String,
    pub intended_outcomes: Vec<String>,
    pub mechanism_summary: Option<String>,
    pub evidence_refs: Vec<MethodEvidenceRef>,
    pub evidence_status: MethodEvidenceStatus,
    pub populations: Vec<String>,
    pub contexts: Vec<String>,
    pub boundary_conditions: Vec<String>,
    pub costs: Vec<String>,
    pub risks_and_limitations: Vec<String>,
    pub compatible_methods: Vec<VersionedMethodRef>,
    pub conflicting_methods: Vec<VersionedMethodRef>,
    pub protocol_variants: Vec<VersionedProtocolRef>,
    pub ai_roles: Vec<MethodAiRole>,
    pub integration_hooks: Vec<VersionedIntegrationCapabilityRef>,
    pub explanation_content_ref: Option<ContentRegistryId>,
    pub provenance: LiteralValue,
}

impl MethodDefinition {
    /// Constructs a catalog Method while keeping evidence, Method, and Protocol roles distinct.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: MethodId,
        version: SemVer,
        name: impl Into<String>,
        category: impl Into<String>,
        target_problem: impl Into<String>,
        intended_outcomes: Vec<String>,
        mechanism_summary: Option<String>,
        evidence_refs: Vec<MethodEvidenceRef>,
        evidence_status: MethodEvidenceStatus,
        populations: Vec<String>,
        contexts: Vec<String>,
        boundary_conditions: Vec<String>,
        costs: Vec<String>,
        risks_and_limitations: Vec<String>,
        compatible_methods: Vec<VersionedMethodRef>,
        conflicting_methods: Vec<VersionedMethodRef>,
        protocol_variants: Vec<VersionedProtocolRef>,
        ai_roles: Vec<MethodAiRole>,
        integration_hooks: Vec<VersionedIntegrationCapabilityRef>,
        explanation_content_ref: Option<ContentRegistryId>,
        provenance: LiteralValue,
    ) -> Result<Self, MethodViolation> {
        let name = name.into();
        let category = category.into();
        let target_problem = target_problem.into();
        for (field, value) in [
            ("method.name", name.as_str()),
            ("method.category", category.as_str()),
            ("method.target_problem", target_problem.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(MethodViolation::EmptyText(field));
            }
        }
        if intended_outcomes.is_empty()
            || intended_outcomes
                .iter()
                .any(|value| value.trim().is_empty())
        {
            return Err(MethodViolation::MissingIntendedOutcome);
        }
        for (field, values) in [
            ("intended_outcomes", &intended_outcomes),
            ("populations", &populations),
            ("contexts", &contexts),
            ("boundary_conditions", &boundary_conditions),
            ("costs", &costs),
            ("risks_and_limitations", &risks_and_limitations),
        ] {
            ensure_unique(values, field).map_err(MethodViolation::Path)?;
        }
        ensure_unique(&evidence_refs, "method.evidence_refs").map_err(MethodViolation::Path)?;
        ensure_unique(&compatible_methods, "method.compatible_methods")
            .map_err(MethodViolation::Path)?;
        ensure_unique(&conflicting_methods, "method.conflicting_methods")
            .map_err(MethodViolation::Path)?;
        ensure_unique(&protocol_variants, "method.protocol_variants")
            .map_err(MethodViolation::Path)?;
        ensure_unique(&ai_roles, "method.ai_roles").map_err(MethodViolation::Path)?;
        ensure_unique(&integration_hooks, "method.integration_hooks")
            .map_err(MethodViolation::Path)?;
        let self_ref = VersionedMethodRef {
            method_id: id.clone(),
            method_version: version.clone(),
        };
        if compatible_methods.contains(&self_ref) || conflicting_methods.contains(&self_ref) {
            return Err(MethodViolation::SelfRelation);
        }
        if compatible_methods
            .iter()
            .any(|reference| conflicting_methods.contains(reference))
        {
            return Err(MethodViolation::ConflictingRelation);
        }
        if matches!(
            evidence_status,
            MethodEvidenceStatus::Scientific
                | MethodEvidenceStatus::MixedContextDependent
                | MethodEvidenceStatus::DesignHypothesis
        ) && evidence_refs.is_empty()
        {
            return Err(MethodViolation::EvidenceStatusWithoutEvidence);
        }
        if provenance == LiteralValue::Null {
            return Err(MethodViolation::MissingProvenance);
        }
        Ok(Self {
            id,
            version,
            name,
            category,
            target_problem,
            intended_outcomes,
            mechanism_summary,
            evidence_refs,
            evidence_status,
            populations,
            contexts,
            boundary_conditions,
            costs,
            risks_and_limitations,
            compatible_methods,
            conflicting_methods,
            protocol_variants,
            ai_roles,
            integration_hooks,
            explanation_content_ref,
            provenance,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodViolation {
    EmptyText(&'static str),
    MissingIntendedOutcome,
    SelfRelation,
    ConflictingRelation,
    EvidenceStatusWithoutEvidence,
    MissingProvenance,
    Path(PathViolation),
}

impl Display for MethodViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyText(field) => write!(formatter, "{field} must not be empty"),
            Self::MissingIntendedOutcome => {
                formatter.write_str("Method must declare a non-empty intended outcome")
            }
            Self::SelfRelation => formatter.write_str("Method must not relate to itself"),
            Self::ConflictingRelation => formatter
                .write_str("Method cannot be both compatible and conflicting with the same Method"),
            Self::EvidenceStatusWithoutEvidence => formatter
                .write_str("scientific/design evidence status requires a governed evidence ref"),
            Self::MissingProvenance => formatter.write_str("Method provenance is required"),
            Self::Path(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for MethodViolation {}

/// Catalog or user-store reference cited as an AI proposal input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpistemicBasisTarget {
    UserRecord(TypedRef),
    EvidenceClaim(EvidenceClaimId),
    DesignHypothesis(DesignHypothesisId),
    Method(VersionedMethodRef),
}

/// One explicitly typed input to AI reasoning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpistemicBasisRef {
    pub reference: EpistemicBasisTarget,
    pub class: EpistemicClass,
    pub source_or_derivation_ref: LiteralValue,
}

impl EpistemicBasisRef {
    pub fn new(
        reference: EpistemicBasisTarget,
        class: EpistemicClass,
        source_or_derivation_ref: LiteralValue,
    ) -> Result<Self, AIProposalViolation> {
        let result = Self {
            reference,
            class,
            source_or_derivation_ref,
        };
        result.validate()?;
        Ok(result)
    }

    /// Revalidates epistemic provenance after decoding provider output.
    pub fn validate(&self) -> Result<(), AIProposalViolation> {
        if self.class == EpistemicClass::Recommendation {
            return Err(AIProposalViolation::RecommendationUsedAsBasis);
        }
        if self.source_or_derivation_ref == LiteralValue::Null {
            return Err(AIProposalViolation::MissingBasisProvenance);
        }
        Ok(())
    }
}

/// References displayed in a proposal's explanation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalRationaleRef {
    EvidenceClaim(EvidenceClaimId),
    DesignHypothesis(DesignHypothesisId),
    Method(VersionedMethodRef),
    Observation(ObservationId),
    PersonalPattern(PersonalPatternId),
}

/// Target plus optimistic-concurrency version for an AI proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalTarget {
    pub target_ref: TypedRef,
    pub base_target_version: Option<u64>,
}

/// Provenance of the model/prompt that produced immutable proposal content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelProvenance {
    pub provider: String,
    pub model: String,
    pub model_version: String,
    pub prompt_template_version: String,
}

impl ModelProvenance {
    pub fn new(
        provider: impl Into<String>,
        model: impl Into<String>,
        model_version: impl Into<String>,
        prompt_template_version: impl Into<String>,
    ) -> Result<Self, AIProposalViolation> {
        let result = Self {
            provider: provider.into(),
            model: model.into(),
            model_version: model_version.into(),
            prompt_template_version: prompt_template_version.into(),
        };
        result.validate()?;
        Ok(result)
    }

    /// Revalidates model and prompt identity after decoding provider output.
    pub fn validate(&self) -> Result<(), AIProposalViolation> {
        if [
            self.provider.as_str(),
            self.model.as_str(),
            self.model_version.as_str(),
            self.prompt_template_version.as_str(),
        ]
        .iter()
        .any(|value| value.trim().is_empty())
        {
            return Err(AIProposalViolation::IncompleteModelProvenance);
        }
        Ok(())
    }
}

/// Pack-owned proposal type. The namespace prevents collision with kernel kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackNamespacedProposalKind(String);

impl PackNamespacedProposalKind {
    pub fn new(value: impl Into<String>) -> Result<Self, AIProposalViolation> {
        let value = value.into();
        if !is_namespaced(&value) {
            return Err(AIProposalViolation::UnnamespacedExtension);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Registered integration operation proposed for later user-confirmed execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationActionProposal {
    capability_ref: VersionedIntegrationCapabilityRef,
    operation_id: IntegrationOperationId,
    request: LiteralValue,
}

impl IntegrationActionProposal {
    /// Names only an operation actually registered by the referenced capability version.
    pub fn new(
        capability: &IntegrationCapability,
        operation_id: IntegrationOperationId,
        request: LiteralValue,
    ) -> Result<Self, IntegrationViolation> {
        capability.validate()?;
        if capability.operation(&operation_id).is_none() {
            return Err(IntegrationViolation::UnknownOperation);
        }
        Ok(Self {
            capability_ref: VersionedIntegrationCapabilityRef {
                capability_id: capability.id.clone(),
                capability_version: capability.version.clone(),
            },
            operation_id,
            request,
        })
    }

    pub fn capability_ref(&self) -> &VersionedIntegrationCapabilityRef {
        &self.capability_ref
    }

    pub fn operation_id(&self) -> &IntegrationOperationId {
        &self.operation_id
    }

    pub fn request(&self) -> &LiteralValue {
        &self.request
    }
}

/// Closed payload union. There is deliberately no generic mutation/command payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AIProposalPayload {
    PathDelta(PathDelta),
    ProtocolSelection(VersionedProtocolRef),
    CalibrationSuggestion {
        calibration_ref: CalibrationRecordId,
        proposed_value: LiteralValue,
    },
    ReflectionStructure {
        reflection_ref: ReflectionOutcomeId,
        structure: LiteralValue,
    },
    IntegrationAction(IntegrationActionProposal),
    PackSpecific {
        kind: PackNamespacedProposalKind,
        payload: LiteralValue,
    },
}

/// Stable discriminator derived from the closed payload union.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AIProposalKind {
    PathDelta,
    ProtocolSelection,
    CalibrationSuggestion,
    ReflectionStructure,
    IntegrationAction,
    PackSpecific(PackNamespacedProposalKind),
}

impl AIProposalPayload {
    pub fn kind(&self) -> AIProposalKind {
        match self {
            Self::PathDelta(_) => AIProposalKind::PathDelta,
            Self::ProtocolSelection(_) => AIProposalKind::ProtocolSelection,
            Self::CalibrationSuggestion { .. } => AIProposalKind::CalibrationSuggestion,
            Self::ReflectionStructure { .. } => AIProposalKind::ReflectionStructure,
            Self::IntegrationAction(_) => AIProposalKind::IntegrationAction,
            Self::PackSpecific { kind, .. } => AIProposalKind::PackSpecific(kind.clone()),
        }
    }

    fn targets_durable_state(&self) -> bool {
        !matches!(self, Self::ReflectionStructure { .. })
    }

    fn validate(&self) -> Result<(), AIProposalViolation> {
        if let Self::PathDelta(delta) = self {
            delta.validate().map_err(AIProposalViolation::Path)?;
        }
        Ok(())
    }
}

/// Confidence vocabulary for proposals and personal patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

/// Immutable typed AI output. No method on this type can mutate its target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AIProposal {
    id: AIProposalId,
    kind: AIProposalKind,
    output_class: EpistemicClass,
    payload_schema_version: u32,
    payload: AIProposalPayload,
    payload_sha256: Bytes32,
    target: ProposalTarget,
    epistemic_basis: Vec<EpistemicBasisRef>,
    rationale_refs: Vec<ProposalRationaleRef>,
    assumptions: Vec<String>,
    alternatives: Vec<ProtocolAlternative>,
    confidence: Confidence,
    requires_confirmation: bool,
    model_provenance: ModelProvenance,
    sensitivity_class: SensitivityClass,
    generated_at: Timestamp,
    expires_at: Option<Timestamp>,
}

impl AIProposal {
    /// Validates a provider candidate into an immutable proposal record.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: AIProposalId,
        output_class: EpistemicClass,
        payload_schema_version: u32,
        payload: AIProposalPayload,
        target: ProposalTarget,
        epistemic_basis: Vec<EpistemicBasisRef>,
        rationale_refs: Vec<ProposalRationaleRef>,
        assumptions: Vec<String>,
        alternatives: Vec<ProtocolAlternative>,
        confidence: Confidence,
        requires_confirmation: bool,
        model_provenance: ModelProvenance,
        sensitivity_class: SensitivityClass,
        generated_at: Timestamp,
        expires_at: Option<Timestamp>,
    ) -> Result<Self, AIProposalViolation> {
        if payload_schema_version == 0 {
            return Err(AIProposalViolation::ZeroPayloadSchemaVersion);
        }
        payload.validate()?;
        model_provenance.validate()?;
        if matches!(
            output_class,
            EpistemicClass::Known | EpistemicClass::Observed
        ) {
            return Err(AIProposalViolation::ModelClaimsFact);
        }
        if payload.targets_durable_state()
            && (output_class != EpistemicClass::Recommendation || !requires_confirmation)
        {
            return Err(AIProposalViolation::DirectDurableWriteAttempt);
        }
        if payload.targets_durable_state() && alternatives.is_empty() {
            return Err(AIProposalViolation::MissingAlternative);
        }
        if epistemic_basis.is_empty() {
            return Err(AIProposalViolation::MissingEpistemicBasis);
        }
        for basis in &epistemic_basis {
            basis.validate()?;
        }
        if assumptions.iter().any(|value| value.trim().is_empty()) {
            return Err(AIProposalViolation::EmptyAssumption);
        }
        if alternatives.iter().any(|alternative| {
            alternative.key.trim().is_empty() || alternative.label.trim().is_empty()
        }) {
            return Err(AIProposalViolation::InvalidAlternative);
        }
        let mut alternative_keys = BTreeSet::new();
        if alternatives
            .iter()
            .any(|alternative| !alternative_keys.insert(alternative.key.as_str()))
        {
            return Err(AIProposalViolation::DuplicateAlternative);
        }
        if let AIProposalPayload::PathDelta(delta) = &payload {
            if !matches!(&target.target_ref, TypedRef::Pursuit(id) if id == &delta.pursuit_ref)
                || target.base_target_version != Some(delta.base_path_version)
            {
                return Err(AIProposalViolation::PathDeltaTargetMismatch);
            }
        }
        if target.base_target_version == Some(0) {
            return Err(AIProposalViolation::InvalidBaseTargetVersion);
        }
        if expires_at
            .as_ref()
            .is_some_and(|expires| expires <= &generated_at)
        {
            return Err(AIProposalViolation::InvalidExpiration);
        }
        let payload_sha256 = proposal_payload_digest(&payload);
        let kind = payload.kind();
        Ok(Self {
            id,
            kind,
            output_class,
            payload_schema_version,
            payload,
            payload_sha256,
            target,
            epistemic_basis,
            rationale_refs,
            assumptions,
            alternatives,
            confidence,
            requires_confirmation,
            model_provenance,
            sensitivity_class,
            generated_at,
            expires_at,
        })
    }

    /// Read-only access to the typed proposal payload.
    pub fn payload(&self) -> &AIProposalPayload {
        &self.payload
    }

    pub fn id(&self) -> &AIProposalId {
        &self.id
    }

    pub fn kind(&self) -> &AIProposalKind {
        &self.kind
    }

    pub const fn output_class(&self) -> EpistemicClass {
        self.output_class
    }

    pub const fn payload_schema_version(&self) -> u32 {
        self.payload_schema_version
    }

    pub const fn payload_sha256(&self) -> Bytes32 {
        self.payload_sha256
    }

    pub fn target(&self) -> &ProposalTarget {
        &self.target
    }

    pub fn epistemic_basis(&self) -> &[EpistemicBasisRef] {
        &self.epistemic_basis
    }

    pub fn rationale_refs(&self) -> &[ProposalRationaleRef] {
        &self.rationale_refs
    }

    pub fn assumptions(&self) -> &[String] {
        &self.assumptions
    }

    pub fn alternatives(&self) -> &[ProtocolAlternative] {
        &self.alternatives
    }

    pub const fn confidence(&self) -> Confidence {
        self.confidence
    }

    pub const fn requires_confirmation(&self) -> bool {
        self.requires_confirmation
    }

    pub fn model_provenance(&self) -> &ModelProvenance {
        &self.model_provenance
    }

    pub const fn sensitivity_class(&self) -> SensitivityClass {
        self.sensitivity_class
    }

    pub fn generated_at(&self) -> &Timestamp {
        &self.generated_at
    }

    pub fn expires_at(&self) -> Option<&Timestamp> {
        self.expires_at.as_ref()
    }

    /// Whether the immutable proposal has expired at the supplied validation time.
    pub fn is_expired_at(&self, at: &Timestamp) -> bool {
        self.expires_at
            .as_ref()
            .is_some_and(|expires| expires <= at)
    }
}

/// Mechanical AI proposal rejection reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AIProposalViolation {
    ZeroPayloadSchemaVersion,
    ModelClaimsFact,
    DirectDurableWriteAttempt,
    MissingAlternative,
    MissingEpistemicBasis,
    RecommendationUsedAsBasis,
    MissingBasisProvenance,
    IncompleteModelProvenance,
    UnnamespacedExtension,
    EmptyAssumption,
    InvalidAlternative,
    DuplicateAlternative,
    PathDeltaTargetMismatch,
    InvalidBaseTargetVersion,
    InvalidExpiration,
    ProposalExpired,
    DispositionDecisionMismatch,
    InvalidDispositionSequence,
    Path(PathViolation),
}

impl Display for AIProposalViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ZeroPayloadSchemaVersion => "proposal payload schema version must be positive",
            Self::ModelClaimsFact => "AI output cannot claim KNOWN or OBSERVED durable facts",
            Self::DirectDurableWriteAttempt => {
                "durable-state proposal must be a RECOMMENDATION requiring confirmation"
            }
            Self::MissingAlternative => "durable-state proposal requires a meaningful alternative",
            Self::MissingEpistemicBasis => "AI proposal requires at least one typed basis ref",
            Self::RecommendationUsedAsBasis => "RECOMMENDATION cannot be cited as a factual basis",
            Self::MissingBasisProvenance => "epistemic basis requires source/derivation provenance",
            Self::IncompleteModelProvenance => "model provenance fields must all be present",
            Self::UnnamespacedExtension => "pack proposal kind must be namespaced",
            Self::EmptyAssumption => "proposal assumptions must not contain empty values",
            Self::InvalidAlternative => {
                "proposal alternatives require a stable key and user-readable label"
            }
            Self::DuplicateAlternative => "proposal alternative keys must be unique",
            Self::PathDeltaTargetMismatch => {
                "Path delta target/base version does not match payload"
            }
            Self::InvalidBaseTargetVersion => "base target version must be positive when present",
            Self::InvalidExpiration => "proposal expiration must be after generation",
            Self::ProposalExpired => "expired proposal cannot mutate durable state",
            Self::DispositionDecisionMismatch => "proposal disposition does not match its decision",
            Self::InvalidDispositionSequence => "proposal disposition sequence must be positive",
            Self::Path(_) => "proposal Path delta failed local validation",
        })
    }
}

impl Error for AIProposalViolation {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Path(error) => Some(error),
            _ => None,
        }
    }
}

/// Append-only result of proposal validation or user review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIProposalDispositionKind {
    RejectedByValidation,
    Accepted,
    Modified,
    Rejected,
    Deferred,
    Expired,
}

/// Immutable disposition appended after proposal creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AIProposalDisposition {
    pub proposal_ref: AIProposalId,
    pub sequence: u64,
    pub disposition: AIProposalDispositionKind,
    pub decision_ref: Option<DecisionRecordId>,
    pub reason_code: String,
    pub recorded_at: Timestamp,
}

impl AIProposalDisposition {
    pub fn new(
        proposal: &AIProposal,
        sequence: u64,
        disposition: AIProposalDispositionKind,
        decision: Option<&DecisionRecord>,
        reason_code: impl Into<String>,
        recorded_at: Timestamp,
    ) -> Result<Self, AIProposalViolation> {
        if sequence == 0 {
            return Err(AIProposalViolation::InvalidDispositionSequence);
        }
        if matches!(
            disposition,
            AIProposalDispositionKind::Accepted | AIProposalDispositionKind::Modified
        ) && proposal.is_expired_at(&recorded_at)
        {
            return Err(AIProposalViolation::ProposalExpired);
        }
        let expected_choice = match disposition {
            AIProposalDispositionKind::Accepted => Some(ProtocolResponseChoice::Accept),
            AIProposalDispositionKind::Modified => None,
            AIProposalDispositionKind::Rejected => Some(ProtocolResponseChoice::Decline),
            AIProposalDispositionKind::Deferred => Some(ProtocolResponseChoice::Defer),
            AIProposalDispositionKind::RejectedByValidation
            | AIProposalDispositionKind::Expired => {
                if decision.is_some() {
                    return Err(AIProposalViolation::DispositionDecisionMismatch);
                }
                None
            }
        };
        let decision_required = !matches!(
            disposition,
            AIProposalDispositionKind::RejectedByValidation | AIProposalDispositionKind::Expired
        );
        if decision_required {
            let Some(decision) = decision else {
                return Err(AIProposalViolation::DispositionDecisionMismatch);
            };
            let valid_common = decision.decision_kind() == &DecisionKind::AiProposalResponse
                && matches!(
                    decision.recommendation_ref(),
                    Some(RecommendationRef::AiProposal(id)) if id == proposal.id()
                );
            let valid_choice = match (expected_choice, decision.outcome()) {
                (
                    None,
                    DecisionOutcome::AiProposalResponse(
                        ProtocolResponseChoice::Modify | ProtocolResponseChoice::Alternative,
                    ),
                ) => true,
                (Some(expected), DecisionOutcome::AiProposalResponse(actual)) => {
                    expected == *actual
                }
                _ => false,
            };
            if !valid_common || !valid_choice {
                return Err(AIProposalViolation::DispositionDecisionMismatch);
            }
        }
        let reason_code = reason_code.into();
        if reason_code.trim().is_empty() {
            return Err(AIProposalViolation::DispositionDecisionMismatch);
        }
        Ok(Self {
            proposal_ref: proposal.id().clone(),
            sequence,
            disposition,
            decision_ref: decision.map(|value| value.id().clone()),
            reason_code,
            recorded_at,
        })
    }
}

/// AI role requested through the asynchronous provider adapter boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIRequestRole {
    Discovery,
    ComposerAssistance,
    Sparring,
    Tutoring,
    Reflection,
    EvidenceExplanation,
    Orchestration,
    PatternAnalysis,
}

/// Bounded provider request; it contains no store, event-bus, or command handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AIRequest {
    pub request_schema_version: u32,
    pub role: AIRequestRole,
    pub bounded_context: LiteralValue,
    pub permitted_proposal_kinds: Vec<AIProposalKind>,
    pub sensitivity_policy_ref: String,
    pub correlation_id: crate::Id,
}

/// Provider result prior to local proposal validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AIProviderResult {
    ProposalCandidate(AIProposalPayload),
    Unavailable(String),
    Failed(String),
}

/// Adapter boundary only. Implementations are orchestrated asynchronously outside render paths.
pub trait AIProvider {
    fn generate(&self, request: &AIRequest) -> AIProviderResult;
}

/// Pack-owned Personal Pattern vocabulary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackNamespacedPatternKind(String);

impl PackNamespacedPatternKind {
    pub fn new(value: impl Into<String>) -> Result<Self, PersonalPatternViolation> {
        let value = value.into();
        if !is_namespaced(&value) {
            return Err(PersonalPatternViolation::UnnamespacedPatternKind);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Processor class used to preserve deterministic versus model provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternProcessorKind {
    Deterministic,
    Ai,
}

/// Exact processor provenance for an immutable Personal Pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternProcessorProvenance {
    pub kind: PatternProcessorKind,
    pub model: Option<ModelProvenance>,
}

impl PatternProcessorProvenance {
    pub fn new(
        kind: PatternProcessorKind,
        model: Option<ModelProvenance>,
    ) -> Result<Self, PersonalPatternViolation> {
        let result = Self { kind, model };
        result.validate()?;
        Ok(result)
    }

    /// Revalidates deterministic-versus-AI processor provenance.
    pub fn validate(&self) -> Result<(), PersonalPatternViolation> {
        if matches!(
            (self.kind, self.model.is_some()),
            (PatternProcessorKind::Deterministic, true) | (PatternProcessorKind::Ai, false)
        ) {
            return Err(PersonalPatternViolation::ProcessorProvenanceMismatch);
        }
        if let Some(model) = &self.model {
            model
                .validate()
                .map_err(|_| PersonalPatternViolation::ProcessorProvenanceMismatch)?;
        }
        Ok(())
    }
}

/// Personal evidence inferred from a bounded observation window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalPattern {
    id: PersonalPatternId,
    subject_ref: TypedRef,
    pattern_type: PackNamespacedPatternKind,
    statement: String,
    epistemic_class: EpistemicClass,
    supporting_observation_refs: Vec<ObservationId>,
    derivation_ref: DerivationRef,
    processor_provenance: PatternProcessorProvenance,
    observed_from: Timestamp,
    observed_through: Timestamp,
    uncertainty: LiteralValue,
    sensitivity_class: SensitivityClass,
    supersedes: Option<PersonalPatternId>,
    /// The exact Method/version this pattern is evidence *about*, when it is Method-scoped.
    /// `None` means it says nothing about any particular Method. Consumers that reason about a
    /// specific Method's support must require an exact match here: sharing a subject with the
    /// Method under consideration is not evidence about that Method.
    relates_to_method: Option<VersionedMethodRef>,
    created_at: Timestamp,
}

impl PersonalPattern {
    /// Constructs personal evidence with no field or operation for scientific promotion.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: PersonalPatternId,
        subject_ref: TypedRef,
        pattern_type: PackNamespacedPatternKind,
        statement: impl Into<String>,
        epistemic_class: EpistemicClass,
        supporting_observation_refs: Vec<ObservationId>,
        derivation_ref: DerivationRef,
        processor_provenance: PatternProcessorProvenance,
        observed_from: Timestamp,
        observed_through: Timestamp,
        uncertainty: LiteralValue,
        sensitivity_class: SensitivityClass,
        supersedes: Option<PersonalPatternId>,
        relates_to_method: Option<VersionedMethodRef>,
        created_at: Timestamp,
    ) -> Result<Self, PersonalPatternViolation> {
        let statement = statement.into();
        if statement.trim().is_empty() {
            return Err(PersonalPatternViolation::EmptyStatement);
        }
        if !matches!(
            epistemic_class,
            EpistemicClass::Inferred | EpistemicClass::Hypothesis
        ) {
            return Err(PersonalPatternViolation::InvalidEpistemicClass);
        }
        processor_provenance.validate()?;
        if supporting_observation_refs.len() < 2 {
            return Err(PersonalPatternViolation::MissingObservations);
        }
        ensure_unique(&supporting_observation_refs, "supporting_observation_refs")
            .map_err(PersonalPatternViolation::Path)?;
        if !matches!(
            subject_ref,
            TypedRef::Pursuit(_)
                | TypedRef::PracticeItem(_)
                | TypedRef::Session(_)
                | TypedRef::Attempt(_)
                | TypedRef::PackOwned { .. }
        ) {
            return Err(PersonalPatternViolation::InvalidSubject);
        }
        if derivation_ref.derivation_version.trim().is_empty()
            || derivation_ref.method.trim().is_empty()
            || derivation_ref.source_observation_refs.is_empty()
        {
            return Err(PersonalPatternViolation::IncompleteDerivation);
        }
        let support: BTreeSet<_> = supporting_observation_refs.iter().collect();
        let derivation_sources: BTreeSet<_> =
            derivation_ref.source_observation_refs.iter().collect();
        if support != derivation_sources {
            return Err(PersonalPatternViolation::DerivationObservationMismatch);
        }
        if observed_from > observed_through {
            return Err(PersonalPatternViolation::InvalidTimeWindow);
        }
        if uncertainty == LiteralValue::Null {
            return Err(PersonalPatternViolation::MissingUncertainty);
        }
        if supersedes.as_ref() == Some(&id) {
            return Err(PersonalPatternViolation::SelfSupersession);
        }
        Ok(Self {
            id,
            subject_ref,
            pattern_type,
            statement,
            epistemic_class,
            supporting_observation_refs,
            derivation_ref,
            processor_provenance,
            observed_from,
            observed_through,
            uncertainty,
            sensitivity_class,
            supersedes,
            relates_to_method,
            created_at,
        })
    }

    pub fn id(&self) -> &PersonalPatternId {
        &self.id
    }

    pub fn subject_ref(&self) -> &TypedRef {
        &self.subject_ref
    }

    pub fn pattern_type(&self) -> &PackNamespacedPatternKind {
        &self.pattern_type
    }

    pub fn statement(&self) -> &str {
        &self.statement
    }

    pub const fn epistemic_class(&self) -> EpistemicClass {
        self.epistemic_class
    }

    pub fn supporting_observation_refs(&self) -> &[ObservationId] {
        &self.supporting_observation_refs
    }

    pub fn derivation_ref(&self) -> &DerivationRef {
        &self.derivation_ref
    }

    pub fn processor_provenance(&self) -> &PatternProcessorProvenance {
        &self.processor_provenance
    }

    pub fn observed_from(&self) -> &Timestamp {
        &self.observed_from
    }

    pub fn observed_through(&self) -> &Timestamp {
        &self.observed_through
    }

    pub fn uncertainty(&self) -> &LiteralValue {
        &self.uncertainty
    }

    pub const fn sensitivity_class(&self) -> SensitivityClass {
        self.sensitivity_class
    }

    pub fn supersedes(&self) -> Option<&PersonalPatternId> {
        self.supersedes.as_ref()
    }

    /// The exact Method this pattern is evidence about, when it is Method-scoped.
    pub fn relates_to_method(&self) -> Option<&VersionedMethodRef> {
        self.relates_to_method.as_ref()
    }

    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonalPatternViolation {
    UnnamespacedPatternKind,
    ProcessorProvenanceMismatch,
    EmptyStatement,
    InvalidEpistemicClass,
    MissingObservations,
    InvalidSubject,
    IncompleteDerivation,
    DerivationObservationMismatch,
    InvalidTimeWindow,
    MissingUncertainty,
    SelfSupersession,
    Path(PathViolation),
}

impl Display for PersonalPatternViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnnamespacedPatternKind => "Personal Pattern kind must be pack-namespaced",
            Self::ProcessorProvenanceMismatch => {
                "AI model provenance is required iff the Pattern processor is AI"
            }
            Self::EmptyStatement => "Personal Pattern statement must not be empty",
            Self::InvalidEpistemicClass => "Personal Pattern must be INFERRED or HYPOTHESIS",
            Self::MissingObservations => {
                "Personal Pattern requires multiple supporting observations"
            }
            Self::InvalidSubject => "Personal Pattern subject is not an authorized subject kind",
            Self::IncompleteDerivation => "Personal Pattern derivation must be reconstructable",
            Self::DerivationObservationMismatch => {
                "Personal Pattern support must match derivation Observation inputs"
            }
            Self::InvalidTimeWindow => "Personal Pattern observation window is inverted",
            Self::MissingUncertainty => "Personal Pattern uncertainty is required",
            Self::SelfSupersession => "Personal Pattern cannot supersede itself",
            Self::Path(_) => "Personal Pattern contains duplicate supporting observations",
        })
    }
}

impl Error for PersonalPatternViolation {}

/// How a Method execution was initiated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodExecutionInitiation {
    User,
    AcceptedRecommendation,
}

/// Thin execution metadata keyed by an existing Session; not a second run aggregate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodExecution {
    pub session_ref: SessionId,
    pub method_ref: VersionedMethodRef,
    pub protocol_ref: Option<VersionedProtocolRef>,
    pub initiation: MethodExecutionInitiation,
    pub source_proposal_ref: Option<AIProposalId>,
    pub source_decision_ref: Option<DecisionRecordId>,
    pub created_at: Timestamp,
}

impl MethodExecution {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_ref: SessionId,
        method_ref: VersionedMethodRef,
        protocol_ref: Option<VersionedProtocolRef>,
        initiation: MethodExecutionInitiation,
        source_proposal_ref: Option<AIProposalId>,
        source_decision_ref: Option<DecisionRecordId>,
        created_at: Timestamp,
    ) -> Result<Self, MethodExecutionViolation> {
        match initiation {
            MethodExecutionInitiation::User
                if source_proposal_ref.is_some() || source_decision_ref.is_some() =>
            {
                return Err(MethodExecutionViolation::UnexpectedRecommendationSource)
            }
            MethodExecutionInitiation::AcceptedRecommendation if source_decision_ref.is_none() => {
                return Err(MethodExecutionViolation::MissingDecision)
            }
            _ => {}
        }
        Ok(Self {
            session_ref,
            method_ref,
            protocol_ref,
            initiation,
            source_proposal_ref,
            source_decision_ref,
            created_at,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodExecutionViolation {
    UnexpectedRecommendationSource,
    MissingDecision,
}

impl Display for MethodExecutionViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnexpectedRecommendationSource => {
                "user-initiated Method execution cannot cite a recommendation source"
            }
            Self::MissingDecision => {
                "recommendation-initiated Method execution requires an accepted decision"
            }
        })
    }
}

impl Error for MethodExecutionViolation {}

/// Explicit integration availability/freshness state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationState {
    Available,
    ConnectedEmpty,
    TemporarilyFailed,
    Unavailable,
    Unauthorized,
    Unknown,
}

/// Integration operation mode. Write remains a separately activated capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationOperationMode {
    Read,
    PreviewWrite,
    Write,
}

/// Typed operation exposed by a specialist-tool adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationDefinition {
    pub operation_id: IntegrationOperationId,
    pub mode: IntegrationOperationMode,
    pub request_schema: LiteralValue,
    pub response_schema: LiteralValue,
    pub required_scopes: Vec<String>,
    pub target_allowlist_kind: Option<String>,
    pub idempotency_strategy: Option<String>,
    pub precondition_strategy: Option<String>,
    pub failure_schema: LiteralValue,
}

impl OperationDefinition {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: IntegrationOperationId,
        mode: IntegrationOperationMode,
        request_schema: LiteralValue,
        response_schema: LiteralValue,
        required_scopes: Vec<String>,
        target_allowlist_kind: Option<String>,
        idempotency_strategy: Option<String>,
        precondition_strategy: Option<String>,
        failure_schema: LiteralValue,
    ) -> Result<Self, IntegrationViolation> {
        let result = Self {
            operation_id,
            mode,
            request_schema,
            response_schema,
            required_scopes,
            target_allowlist_kind,
            idempotency_strategy,
            precondition_strategy,
            failure_schema,
        };
        result.validate()?;
        Ok(result)
    }

    /// Revalidates a decoded operation before registration or proposal use.
    pub fn validate(&self) -> Result<(), IntegrationViolation> {
        if !is_namespaced(self.operation_id.as_str()) {
            return Err(IntegrationViolation::OperationNotNamespaced);
        }
        if self
            .required_scopes
            .iter()
            .any(|scope| scope.trim().is_empty())
        {
            return Err(IntegrationViolation::EmptyScope);
        }
        let unique_scopes = self.required_scopes.iter().collect::<BTreeSet<_>>();
        if unique_scopes.len() != self.required_scopes.len() {
            return Err(IntegrationViolation::DuplicateScope);
        }
        if self.mode == IntegrationOperationMode::Write
            && [
                self.target_allowlist_kind.as_deref(),
                self.idempotency_strategy.as_deref(),
                self.precondition_strategy.as_deref(),
            ]
            .iter()
            .any(|value| value.is_none_or(|text| text.trim().is_empty()))
        {
            return Err(IntegrationViolation::UnsafeWriteContract);
        }
        Ok(())
    }
}

/// Versioned specialist-tool capability description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationCapability {
    pub id: IntegrationCapabilityId,
    pub version: SemVer,
    pub provider_family: String,
    pub operations: Vec<OperationDefinition>,
    pub manual_fallback: String,
    pub provenance_schema: LiteralValue,
}

impl IntegrationCapability {
    pub fn new(
        id: IntegrationCapabilityId,
        version: SemVer,
        provider_family: impl Into<String>,
        operations: Vec<OperationDefinition>,
        manual_fallback: impl Into<String>,
        provenance_schema: LiteralValue,
    ) -> Result<Self, IntegrationViolation> {
        let provider_family = provider_family.into();
        let manual_fallback = manual_fallback.into();
        let result = Self {
            id,
            version,
            provider_family,
            operations,
            manual_fallback,
            provenance_schema,
        };
        result.validate()?;
        Ok(result)
    }

    /// Revalidates a decoded capability and every nested operation.
    pub fn validate(&self) -> Result<(), IntegrationViolation> {
        if self.provider_family.trim().is_empty() || self.manual_fallback.trim().is_empty() {
            return Err(IntegrationViolation::MissingDescription);
        }
        if self.operations.is_empty() {
            return Err(IntegrationViolation::MissingOperation);
        }
        for operation in &self.operations {
            operation.validate()?;
        }
        let operation_ids: Vec<_> = self
            .operations
            .iter()
            .map(|item| &item.operation_id)
            .collect();
        ensure_unique(&operation_ids, "integration.operations").map_err(IntegrationViolation::Path)
    }

    pub fn operation(&self, id: &IntegrationOperationId) -> Option<&OperationDefinition> {
        self.operations
            .iter()
            .find(|value| &value.operation_id == id)
    }
}

/// Explicit snapshot freshness carried with a binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationFreshness {
    pub fetched_at: Option<Timestamp>,
    pub valid_through: Option<Timestamp>,
}

/// User-consented binding by reference; it stores no credentials or secrets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationBinding {
    pub id: IntegrationBindingId,
    pub capability_ref: VersionedIntegrationCapabilityRef,
    pub external_account_ref: String,
    pub granted_scopes: Vec<String>,
    pub consent_decision_ref: DecisionRecordId,
    pub state: IntegrationState,
    pub freshness: IntegrationFreshness,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl IntegrationBinding {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: IntegrationBindingId,
        capability_ref: VersionedIntegrationCapabilityRef,
        external_account_ref: impl Into<String>,
        granted_scopes: Vec<String>,
        consent_decision: &DecisionRecord,
        state: IntegrationState,
        freshness: IntegrationFreshness,
        created_at: Timestamp,
        updated_at: Timestamp,
    ) -> Result<Self, IntegrationViolation> {
        let external_account_ref = external_account_ref.into();
        if external_account_ref.trim().is_empty() {
            return Err(IntegrationViolation::MissingExternalAccount);
        }
        if granted_scopes.iter().any(|scope| scope.trim().is_empty()) {
            return Err(IntegrationViolation::EmptyScope);
        }
        if consent_decision.origin() != DecisionOrigin::UserInitiated {
            return Err(IntegrationViolation::ConsentNotUserInitiated);
        }
        if freshness
            .fetched_at
            .as_ref()
            .zip(freshness.valid_through.as_ref())
            .is_some_and(|(fetched, valid)| fetched > valid)
        {
            return Err(IntegrationViolation::InvalidFreshness);
        }
        Ok(Self {
            id,
            capability_ref,
            external_account_ref,
            granted_scopes,
            consent_decision_ref: consent_decision.id().clone(),
            state,
            freshness,
            created_at,
            updated_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrationViolation {
    OperationNotNamespaced,
    EmptyScope,
    DuplicateScope,
    UnsafeWriteContract,
    MissingDescription,
    MissingOperation,
    UnknownOperation,
    MissingExternalAccount,
    ConsentNotUserInitiated,
    InvalidFreshness,
    Path(PathViolation),
}

impl Display for IntegrationViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::OperationNotNamespaced => "integration operation id must be namespaced",
            Self::EmptyScope => "integration scopes must not contain empty values",
            Self::DuplicateScope => "integration scopes must be unique",
            Self::UnsafeWriteContract => {
                "WRITE operation requires allowlist, idempotency, and fresh-precondition strategies"
            }
            Self::MissingDescription => {
                "integration provider family and manual fallback are required"
            }
            Self::MissingOperation => "integration capability requires at least one operation",
            Self::UnknownOperation => {
                "integration action must name an operation registered by its capability"
            }
            Self::MissingExternalAccount => "integration binding requires an external account ref",
            Self::ConsentNotUserInitiated => "integration binding consent must be user-initiated",
            Self::InvalidFreshness => "integration freshness window is inverted",
            Self::Path(_) => "integration capability contains duplicate operations",
        })
    }
}

impl Error for IntegrationViolation {}

fn ensure_text(field: &'static str, value: &str) -> Result<(), PathViolation> {
    if value.trim().is_empty() {
        Err(PathViolation::EmptyText(field))
    } else {
        Ok(())
    }
}

fn ensure_unique<T: PartialEq>(values: &[T], field: &'static str) -> Result<(), PathViolation> {
    for (index, value) in values.iter().enumerate() {
        if values[index + 1..].contains(value) {
            return Err(PathViolation::DuplicateEntry(field));
        }
    }
    Ok(())
}

fn is_namespaced(value: &str) -> bool {
    let mut segments = value.split('.');
    let first = segments.next();
    let second = segments.next();
    first.is_some_and(|part| !part.trim().is_empty())
        && second.is_some_and(|part| !part.trim().is_empty())
        && segments.all(|part| !part.trim().is_empty())
}

fn digest(value: &[u8]) -> Bytes32 {
    const INITIAL: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const ROUND: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let bit_len = (value.len() as u64).wrapping_mul(8);
    let mut padded = value.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    let mut state = INITIAL;
    for chunk in padded.as_chunks::<64>().0 {
        let mut words = [0_u32; 64];
        for (index, bytes) in chunk.as_chunks::<4>().0.iter().enumerate() {
            words[index] = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for index in 0..64 {
            let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ ((!e) & g);
            let temporary1 = h
                .wrapping_add(sum1)
                .wrapping_add(choice)
                .wrapping_add(ROUND[index])
                .wrapping_add(words[index]);
            let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temporary2 = sum0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temporary1);
            d = c;
            c = b;
            b = a;
            a = temporary1.wrapping_add(temporary2);
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }
    let mut bytes = [0_u8; 32];
    for (index, word) in state.iter().enumerate() {
        bytes[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    Bytes32::new(bytes)
}

fn push_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn push_len(output: &mut Vec<u8>, value: usize) {
    push_u64(output, value as u64);
}

fn push_str(output: &mut Vec<u8>, value: &str) {
    push_len(output, value.len());
    output.extend_from_slice(value.as_bytes());
}

fn encode_direction(output: &mut Vec<u8>, value: &DirectionSpec) {
    push_str(output, &value.statement);
    output.push(value.stance as u8);
    match &value.life_area {
        Some(area) => {
            output.push(1);
            push_str(output, &area.key);
            push_str(output, &area.label);
        }
        None => output.push(0),
    }
}

fn encode_method_ref(output: &mut Vec<u8>, value: &VersionedMethodRef) {
    push_str(output, value.method_id.as_str());
    push_str(output, value.method_version.as_str());
}

fn encode_protocol_ref(output: &mut Vec<u8>, value: &VersionedProtocolRef) {
    push_str(output, value.protocol_id.as_str());
    push_str(output, value.protocol_version.as_str());
}

fn encode_integration_ref(output: &mut Vec<u8>, value: &VersionedIntegrationCapabilityRef) {
    push_str(output, value.capability_id.as_str());
    push_str(output, value.capability_version.as_str());
}

fn encode_literal(output: &mut Vec<u8>, value: &LiteralValue) {
    match value {
        LiteralValue::Null => output.push(0),
        LiteralValue::Boolean(value) => {
            output.push(1);
            output.push(u8::from(*value));
        }
        LiteralValue::Integer(value) => {
            output.push(2);
            output.extend_from_slice(&value.to_be_bytes());
        }
        LiteralValue::Unsigned(value) => {
            output.push(3);
            push_u64(output, *value);
        }
        LiteralValue::Decimal(value) => {
            output.push(4);
            push_str(output, value);
        }
        LiteralValue::Text(value) => {
            output.push(5);
            push_str(output, value);
        }
        LiteralValue::List(values) => {
            output.push(6);
            push_len(output, values.len());
            for value in values {
                encode_literal(output, value);
            }
        }
        LiteralValue::Object(values) => {
            output.push(7);
            push_len(output, values.len());
            for (key, value) in values {
                push_str(output, key);
                encode_literal(output, value);
            }
        }
    }
}

fn encode_optional_typed_ref(output: &mut Vec<u8>, value: Option<&TypedRef>) {
    match value {
        Some(value) => {
            output.push(1);
            encode_typed_ref(output, value);
        }
        None => output.push(0),
    }
}

fn encode_typed_ref(output: &mut Vec<u8>, value: &TypedRef) {
    let (tag, id) = match value {
        TypedRef::Pursuit(id) => (0, id.id().as_str()),
        TypedRef::PracticeItem(id) => (1, id.id().as_str()),
        TypedRef::Session(id) => (2, id.id().as_str()),
        TypedRef::Attempt(id) => (3, id.id().as_str()),
        TypedRef::Observation(id) => (4, id.id().as_str()),
        TypedRef::DecisionRecord(id) => (5, id.id().as_str()),
        TypedRef::ReflectionOutcome(id) => (6, id.id().as_str()),
        TypedRef::CalibrationRecord(id) => (7, id.id().as_str()),
        TypedRef::CommitmentDevice(id) => (8, id.id().as_str()),
        TypedRef::PathDraft(id) => (9, id.id().as_str()),
        TypedRef::AIProposal(id) => (10, id.id().as_str()),
        TypedRef::PersonalPattern(id) => (11, id.id().as_str()),
        TypedRef::PackOwned { kind, id } => {
            output.push(12);
            push_str(output, kind.as_str());
            push_str(output, id.as_str());
            return;
        }
    };
    output.push(tag);
    push_str(output, id);
}

fn encode_path_change(output: &mut Vec<u8>, value: &PathChange) {
    match value {
        PathChange::SetDirection(value) => {
            output.push(0);
            encode_direction(output, value);
        }
        PathChange::SetOperatingState(value) => {
            output.push(1);
            output.push(*value as u8);
        }
        PathChange::AddCapability(value) => {
            output.push(2);
            push_str(output, value.owning_pack_ref.as_str());
            push_str(output, &value.capability_key);
        }
        PathChange::RemoveCapability(value) => {
            output.push(3);
            push_str(output, value.owning_pack_ref.as_str());
            push_str(output, &value.capability_key);
        }
        PathChange::ReorderCapability(value, position) => {
            output.push(4);
            push_str(output, value.owning_pack_ref.as_str());
            push_str(output, &value.capability_key);
            push_u64(output, u64::from(*position));
        }
        PathChange::AddMethod(value) => {
            output.push(5);
            encode_method_ref(output, value);
        }
        PathChange::RemoveMethod(value) => {
            output.push(6);
            encode_method_ref(output, value);
        }
        PathChange::AddProtocol(value) => {
            output.push(7);
            encode_protocol_ref(output, value);
        }
        PathChange::RemoveProtocol(value) => {
            output.push(8);
            encode_protocol_ref(output, value);
        }
        PathChange::AddIntegration(value) => {
            output.push(9);
            encode_integration_ref(output, value);
        }
        PathChange::RemoveIntegration(value) => {
            output.push(10);
            encode_integration_ref(output, value);
        }
        PathChange::AddExperiment(id) => {
            output.push(11);
            push_str(output, id.id().as_str());
        }
        PathChange::RemoveExperiment(id) => {
            output.push(12);
            push_str(output, id.id().as_str());
        }
        PathChange::SetConstraint(value) => {
            output.push(13);
            push_str(output, &value.key);
            encode_literal(output, &value.value);
            output.push(value.provenance as u8);
            encode_optional_typed_ref(output, value.provenance_ref.as_ref());
        }
        PathChange::RemoveConstraint(key) => {
            output.push(14);
            push_str(output, key);
        }
        PathChange::SetOverride(key, value) => {
            output.push(15);
            push_str(output, key);
            encode_literal(output, value);
        }
        PathChange::RemoveOverride(key) => {
            output.push(16);
            push_str(output, key);
        }
        PathChange::SetExplanationPreference(value) => {
            output.push(17);
            output.push(*value as u8);
        }
    }
}

fn proposal_payload_digest(value: &AIProposalPayload) -> Bytes32 {
    let mut output = Vec::new();
    match value {
        AIProposalPayload::PathDelta(value) => {
            output.push(0);
            output.extend_from_slice(value.sha256().as_bytes());
        }
        AIProposalPayload::ProtocolSelection(value) => {
            output.push(1);
            encode_protocol_ref(&mut output, value);
        }
        AIProposalPayload::CalibrationSuggestion {
            calibration_ref,
            proposed_value,
        } => {
            output.push(2);
            push_str(&mut output, calibration_ref.id().as_str());
            encode_literal(&mut output, proposed_value);
        }
        AIProposalPayload::ReflectionStructure {
            reflection_ref,
            structure,
        } => {
            output.push(3);
            push_str(&mut output, reflection_ref.id().as_str());
            encode_literal(&mut output, structure);
        }
        AIProposalPayload::IntegrationAction(value) => {
            output.push(4);
            encode_integration_ref(&mut output, &value.capability_ref);
            push_str(&mut output, value.operation_id.as_str());
            encode_literal(&mut output, &value.request);
        }
        AIProposalPayload::PackSpecific { kind, payload } => {
            output.push(5);
            push_str(&mut output, kind.as_str());
            encode_literal(&mut output, payload);
        }
    }
    digest(&output)
}

#[cfg(test)]
mod tests {
    use super::digest;

    #[test]
    fn local_sha256_matches_the_standard_vector() {
        assert_eq!(
            digest(b"abc").as_bytes(),
            &[
                0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
                0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
                0xf2, 0x00, 0x15, 0xad,
            ]
        );
    }
}
