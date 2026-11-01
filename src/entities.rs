//! Canonical Flourish kernel entities and their construction invariants.

use crate::{
    check_condition_bound_eligibility, AlternativeId, AttemptId, CalibrationRecordId,
    CommitmentDeviceId, ConditionBoundViolation, ContentRegistryId, ContextFactRegistry,
    ContextSnapshotId, DecisionRecordId, DomainProfileId, EventTopicRegistry, Id, LiteralValue,
    ObservationId, ObservationType, OpaqueRef, OwningRef, PackId, PracticeItemId, Predicate,
    ProtocolId, ReflectionOutcomeId, ScopeRef, SemVer, SessionId, StrategyId, Timestamp, TypedRef,
};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Kind of user Pursuit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PursuitKind {
    /// Desired future outcome.
    Outcome,
    /// Finite project.
    Project,
    /// Capability-development goal.
    CapabilityGoal,
    /// Practice goal.
    PracticeGoal,
    /// Venture.
    Venture,
}

/// Lifecycle state of a Pursuit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PursuitStatus {
    /// Currently pursued.
    Active,
    /// Temporarily paused.
    Paused,
    /// Explicitly completed by the user.
    Completed,
    /// Stopped without completion.
    Stopped,
    /// Replaced by another Pursuit.
    Replaced,
}

/// Auditable status-history entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PursuitStatusHistoryEntry {
    /// Status entered by this transition.
    pub status: PursuitStatus,
    /// Time of the transition.
    pub at: Timestamp,
    /// Decision that authorized the transition.
    pub decision_record_ref: DecisionRecordId,
}

/// User-intended outcome, project, development goal, practice goal, or venture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pursuit {
    /// Stable identifier.
    pub id: crate::PursuitId,
    /// Pursuit kind.
    pub kind: PursuitKind,
    /// User-facing title.
    pub title: String,
    /// Optional detail.
    pub description: Option<String>,
    /// Optional declarative domain profile.
    pub domain_profile_ref: Option<DomainProfileId>,
    /// Optional parent Pursuit.
    pub parent_ref: Option<crate::PursuitId>,
    /// Current lifecycle state, mutated only through a decision-backed transition.
    status: PursuitStatus,
    /// Auditable transition history, appended only through the transition API.
    status_history: Vec<PursuitStatusHistoryEntry>,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last update time.
    pub updated_at: Timestamp,
}

impl Pursuit {
    /// Creates an active Pursuit with no fabricated transition history.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: crate::PursuitId,
        kind: PursuitKind,
        title: String,
        description: Option<String>,
        domain_profile_ref: Option<DomainProfileId>,
        parent_ref: Option<crate::PursuitId>,
        created_at: Timestamp,
        updated_at: Timestamp,
    ) -> Self {
        Self {
            id,
            kind,
            title,
            description,
            domain_profile_ref,
            parent_ref,
            status: PursuitStatus::Active,
            status_history: Vec::new(),
            created_at,
            updated_at,
        }
    }

    /// Current lifecycle state.
    pub const fn status(&self) -> PursuitStatus {
        self.status
    }

    /// Decision-backed status history.
    pub fn status_history(&self) -> &[PursuitStatusHistoryEntry] {
        &self.status_history
    }

    /// Applies an auditable transition authorized by a decision for this Pursuit.
    pub fn transition_status(
        &mut self,
        target: PursuitStatus,
        at: Timestamp,
        decision: &DecisionRecord,
    ) -> Result<(), PursuitTransitionViolation> {
        check_pursuit_status_transition(target, decision)?;
        if !matches!(decision.subject_ref(), TypedRef::Pursuit(id) if id == &self.id) {
            return Err(PursuitTransitionViolation::SubjectMismatch);
        }
        self.status = target;
        self.updated_at = at.clone();
        self.status_history.push(PursuitStatusHistoryEntry {
            status: target,
            at,
            decision_record_ref: decision.id().clone(),
        });
        Ok(())
    }
}

/// Repeatable target of practice, retrieval, testing, or improvement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeItem {
    /// Stable identifier.
    pub id: PracticeItemId,
    /// Optional Pursuit association.
    pub pursuit_ref: Option<crate::PursuitId>,
    /// Pack that defines the item-type vocabulary.
    pub owning_pack_ref: PackId,
    /// Pack-declared item type.
    pub item_type: String,
    /// User-facing title.
    pub title: String,
    /// Optional kernel content reference.
    pub content_ref: Option<ContentRegistryId>,
    /// Optional Learning Strategy; absence means the item is not scheduled.
    pub strategy_ref: Option<StrategyId>,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last update time.
    pub updated_at: Timestamp,
}

/// Kind of bounded execution episode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    /// Study session.
    Study,
    /// Focus session.
    Focus,
    /// Practice session.
    Practice,
    /// Training session.
    Training,
    /// Experiment session.
    Experiment,
    /// Review session.
    Review,
}

/// Bounded execution episode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// Stable identifier.
    pub id: SessionId,
    /// Session kind.
    pub session_type: SessionType,
    /// Optional Pursuit association.
    pub pursuit_ref: Option<crate::PursuitId>,
    /// Practice Items worked during the session.
    pub practice_item_refs: Vec<PracticeItemId>,
    /// Start time.
    pub started_at: Timestamp,
    /// Optional end time for an active session.
    pub ended_at: Option<Timestamp>,
    /// Whether interruption occurred.
    pub interrupted: bool,
    /// Optional user-created Commitment Device.
    pub commitment_device_ref: Option<CommitmentDeviceId>,
    /// Context captured for auditability.
    pub context_snapshot_ref: Option<ContextSnapshotId>,
}

/// Outcome of a discrete Attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptOutcome {
    /// Awaiting feedback.
    Pending,
    /// Correct.
    Correct,
    /// Partially correct.
    Partial,
    /// Incorrect.
    Incorrect,
    /// Qualitative feedback only.
    Qualitative,
}

/// Source of Attempt feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackSource {
    /// Learner self-assessment.
    SelfAssessment,
    /// AI feedback.
    Ai,
    /// Human coach feedback.
    Coach,
    /// Automated test suite.
    TestSuite,
    /// External result.
    Result,
}

/// Discrete attempt against a Practice Item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    /// Stable identifier.
    pub id: AttemptId,
    /// Attempted Practice Item.
    pub practice_item_ref: PracticeItemId,
    /// Optional containing Session.
    pub session_ref: Option<SessionId>,
    /// Attempt time.
    pub attempted_at: Timestamp,
    /// Optional pack-owned input pointer.
    pub input_ref: Option<OpaqueRef>,
    /// Attempt result.
    pub outcome: AttemptOutcome,
    /// Feedback source.
    pub feedback_source: FeedbackSource,
    /// Optional multidimensional qualitative measures.
    pub quality_dims: Option<BTreeMap<String, LiteralValue>>,
}

/// Independent sensitivity policy category.
///
/// This type intentionally does not implement `Ord` or `PartialOrd`: these are
/// non-ranked policy categories rather than a sensitivity ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SensitivityClass {
    /// Ordinary product and operational data.
    Standard,
    /// Journal or reflective private content.
    ReflectivePrivate,
    /// Regulation, wellbeing, or linguistic-analysis data.
    WellbeingSensitive,
    /// Wearable, training, sleep, or biometric observations.
    BiometricHealth,
}

/// Provenance category of an Observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationProvenance {
    /// User-entered value.
    Manual,
    /// OCR-confirmed value.
    Ocr,
    /// Imported integration value.
    Integration,
    /// Value derived from typed inputs.
    Derived,
}

/// Reconstructable derivation provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivationRef {
    /// Kernel service or pack that computed the value.
    pub derived_by: OwningRef,
    /// Version of the derivation logic pinned at write time.
    pub derivation_version: String,
    /// Human-readable computation summary.
    pub method: String,
    /// Source Observations, when applicable.
    pub source_observation_refs: Vec<ObservationId>,
}

/// Invalid Observation construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationViolation {
    /// Derived provenance requires derivation details.
    MissingDerivation,
    /// Non-derived provenance must not carry derivation details.
    UnexpectedDerivation,
}

impl Display for ObservationViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDerivation => {
                formatter.write_str("DERIVED observation requires derivation_ref")
            }
            Self::UnexpectedDerivation => formatter
                .write_str("MANUAL/OCR/INTEGRATION observation must not carry derivation_ref"),
        }
    }
}

impl Error for ObservationViolation {}

/// Immutable typed fact with provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    id: ObservationId,
    subject_ref: TypedRef,
    observation_type: ObservationType,
    value: LiteralValue,
    recorded_at: Timestamp,
    provenance: ObservationProvenance,
    provenance_ref: Option<OpaqueRef>,
    derivation_ref: Option<DerivationRef>,
    sensitivity_class: SensitivityClass,
    supersedes: Option<ObservationId>,
}

impl Observation {
    /// Constructs an immutable Observation while enforcing derivation provenance.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ObservationId,
        subject_ref: TypedRef,
        observation_type: ObservationType,
        value: LiteralValue,
        recorded_at: Timestamp,
        provenance: ObservationProvenance,
        provenance_ref: Option<OpaqueRef>,
        derivation_ref: Option<DerivationRef>,
        sensitivity_class: SensitivityClass,
        supersedes: Option<ObservationId>,
    ) -> Result<Self, ObservationViolation> {
        match (provenance, derivation_ref.is_some()) {
            (ObservationProvenance::Derived, false) => {
                return Err(ObservationViolation::MissingDerivation)
            }
            (ObservationProvenance::Derived, true) => {}
            (_, true) => return Err(ObservationViolation::UnexpectedDerivation),
            (_, false) => {}
        }
        Ok(Self {
            id,
            subject_ref,
            observation_type,
            value,
            recorded_at,
            provenance,
            provenance_ref,
            derivation_ref,
            sensitivity_class,
            supersedes,
        })
    }

    /// Stable identifier.
    pub fn id(&self) -> &ObservationId {
        &self.id
    }

    /// Subject of this typed fact.
    pub fn subject_ref(&self) -> &TypedRef {
        &self.subject_ref
    }

    /// Stable observation type.
    pub fn observation_type(&self) -> &ObservationType {
        &self.observation_type
    }

    /// Typed observation value.
    pub fn value(&self) -> &LiteralValue {
        &self.value
    }

    /// Recording time.
    pub fn recorded_at(&self) -> &Timestamp {
        &self.recorded_at
    }

    /// Provenance category.
    pub const fn provenance(&self) -> ObservationProvenance {
        self.provenance
    }

    /// Optional source attachment/integration/OCR pointer.
    pub fn provenance_ref(&self) -> Option<&OpaqueRef> {
        self.provenance_ref.as_ref()
    }

    /// Required details for a derived Observation.
    pub fn derivation_ref(&self) -> Option<&DerivationRef> {
        self.derivation_ref.as_ref()
    }

    /// Fixed sensitivity category.
    pub const fn sensitivity_class(&self) -> SensitivityClass {
        self.sensitivity_class
    }

    /// Corrected Observation, if any.
    pub fn supersedes(&self) -> Option<&ObservationId> {
        self.supersedes.as_ref()
    }
}

/// Source of a Decision Record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionOrigin {
    /// Response to a system recommendation.
    SystemRecommended,
    /// Explicit user action without a recommendation dependency.
    UserInitiated,
}

/// Closed kernel decision kinds plus pack-namespaced extensions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionKind {
    /// Response to a presented Protocol recommendation.
    ProtocolResponse,
    /// Goal-adjustment taxonomy for a Pursuit.
    PursuitStatusChange,
    /// Explicit user-only completion of a Pursuit.
    PursuitCompletion,
    /// Response to an immutable typed AI proposal.
    AiProposalResponse,
    /// Explicit manual acceptance or rejection of a Path draft/delta.
    PathCompositionChange,
    /// Pack-declared, namespaced decision kind.
    PackSpecific(PackDecisionKind),
}

impl DecisionKind {
    /// Creates a pack-specific kind and rejects unnamespaced or kernel-reserved values.
    pub fn pack_specific(value: impl Into<String>) -> Result<Self, DecisionViolation> {
        let value = value.into();
        if !value.contains('.') || value.starts_with('.') || value.ends_with('.') {
            return Err(DecisionViolation::PackKindNotNamespaced);
        }
        if matches!(
            value.as_str(),
            "protocol_response"
                | "pursuit_status_change"
                | "pursuit_completion"
                | "ai_proposal_response"
                | "path_composition_change"
        ) {
            return Err(DecisionViolation::PackKindUsesReservedValue);
        }
        Ok(Self::PackSpecific(PackDecisionKind(value)))
    }

    /// Returns the stable vocabulary string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::ProtocolResponse => "protocol_response",
            Self::PursuitStatusChange => "pursuit_status_change",
            Self::PursuitCompletion => "pursuit_completion",
            Self::AiProposalResponse => "ai_proposal_response",
            Self::PathCompositionChange => "path_composition_change",
            Self::PackSpecific(value) => value.as_str(),
        }
    }
}

/// Validated namespaced decision-kind identifier owned by a Capability Pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackDecisionKind(String);

impl PackDecisionKind {
    /// Returns the validated namespaced vocabulary string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Reference to the recommendation that prompted a decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecommendationRef {
    /// Immutable Protocol Version reference.
    ProtocolVersion {
        protocol_id: ProtocolId,
        protocol_version: SemVer,
    },
    /// Materialized Recommendation entity reference.
    Recommendation(crate::RecommendationId),
    /// Immutable AI Proposal reference.
    AiProposal(crate::AIProposalId),
}

/// Response to a presented recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolResponseChoice {
    /// Accept the recommendation.
    Accept,
    /// Select an alternative.
    Alternative,
    /// Modify it.
    Modify,
    /// Decline it.
    Decline,
    /// Defer it.
    Defer,
}

/// Canonical six-value Goal Adjustment taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PursuitAction {
    /// Continue without a status transition.
    Continue,
    /// Modify without a status transition.
    Modify,
    /// Reduce without a status transition.
    Reduce,
    /// Pause the Pursuit.
    Pause,
    /// Stop the Pursuit.
    Stop,
    /// Replace the Pursuit.
    Replace,
}

/// Explicit disposition of a manually composed Path draft or delta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathCompositionAction {
    /// Accept the reviewed composition.
    Accept,
    /// Accept after explicit user modification.
    Modify,
    /// Decline the candidate composition.
    Decline,
    /// Defer the decision without activating the candidate.
    Defer,
}

/// Outcome shape discriminated by Decision Kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionOutcome {
    /// Protocol response choice.
    ProtocolResponse(ProtocolResponseChoice),
    /// Goal Adjustment action.
    PursuitStatusChange(PursuitAction),
    /// Separate user-only completion action.
    PursuitCompletion,
    /// User response to an AI proposal.
    AiProposalResponse(ProtocolResponseChoice),
    /// Manual Path composition decision.
    PathCompositionChange(PathCompositionAction),
    /// Pack-defined typed payload.
    PackSpecific(LiteralValue),
}

/// Alternative shown for a system recommendation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alternative {
    /// Stable identifier.
    pub id: AlternativeId,
    /// User-readable label.
    pub label: String,
    /// Typed alternative payload.
    pub payload: LiteralValue,
}

/// Invalid Decision Record combination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionViolation {
    /// System-recommended decisions require a recommendation reference.
    MissingRecommendation,
    /// User-initiated decisions cannot fabricate a recommendation reference.
    UnexpectedRecommendation,
    /// The outcome variant does not match the decision kind.
    OutcomeKindMismatch,
    /// Protocol responses are definitionally system-recommended.
    ProtocolResponseMustBeSystemRecommended,
    /// Pursuit completion is always an explicit user action.
    PursuitCompletionMustBeUserInitiated,
    /// Pursuit completion must target the Pursuit being completed.
    PursuitCompletionRequiresPursuitSubject,
    /// AI proposal responses are definitionally system-recommended.
    AiProposalResponseMustBeSystemRecommended,
    /// AI proposal responses must cite the proposal they answer.
    AiProposalResponseRequiresProposalReference,
    /// AI Proposal references belong only to the dedicated response kind.
    AiProposalReferenceRequiresProposalResponse,
    /// Manual Path composition decisions are user-initiated.
    PathCompositionChangeMustBeUserInitiated,
    /// Path composition decisions must target a Pursuit.
    PathCompositionChangeRequiresPursuitSubject,
    /// Pack-specific kinds require a namespace.
    PackKindNotNamespaced,
    /// Pack-specific kinds cannot reuse a reserved kernel value.
    PackKindUsesReservedValue,
}

impl Display for DecisionViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingRecommendation => {
                "SYSTEM_RECOMMENDED decision requires recommendation_ref"
            }
            Self::UnexpectedRecommendation => {
                "USER_INITIATED decision must not carry recommendation_ref"
            }
            Self::OutcomeKindMismatch => "decision outcome does not match decision_kind",
            Self::ProtocolResponseMustBeSystemRecommended => {
                "protocol_response must be SYSTEM_RECOMMENDED"
            }
            Self::PursuitCompletionMustBeUserInitiated => {
                "pursuit_completion must be USER_INITIATED"
            }
            Self::PursuitCompletionRequiresPursuitSubject => {
                "pursuit_completion subject_ref must be a Pursuit"
            }
            Self::AiProposalResponseMustBeSystemRecommended => {
                "ai_proposal_response must be SYSTEM_RECOMMENDED"
            }
            Self::AiProposalResponseRequiresProposalReference => {
                "ai_proposal_response must reference the AI Proposal being answered"
            }
            Self::AiProposalReferenceRequiresProposalResponse => {
                "AI Proposal references require decision_kind = ai_proposal_response"
            }
            Self::PathCompositionChangeMustBeUserInitiated => {
                "path_composition_change must be USER_INITIATED"
            }
            Self::PathCompositionChangeRequiresPursuitSubject => {
                "path_composition_change subject_ref must be a Pursuit"
            }
            Self::PackKindNotNamespaced => "pack decision kind must be namespaced",
            Self::PackKindUsesReservedValue => {
                "pack decision kind must not reuse a kernel-reserved value"
            }
        })
    }
}

impl Error for DecisionViolation {}

/// Auditable user/system decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionRecord {
    id: DecisionRecordId,
    origin: DecisionOrigin,
    decision_kind: DecisionKind,
    subject_ref: TypedRef,
    recommendation_ref: Option<RecommendationRef>,
    alternatives_presented: Vec<Alternative>,
    chosen_alternative_ref: Option<AlternativeId>,
    outcome: DecisionOutcome,
    reason: Option<String>,
    context_snapshot_ref: ContextSnapshotId,
    decided_at: Timestamp,
}

impl DecisionRecord {
    /// Constructs a Decision Record while enforcing discriminator and origin coupling.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: DecisionRecordId,
        origin: DecisionOrigin,
        decision_kind: DecisionKind,
        subject_ref: TypedRef,
        recommendation_ref: Option<RecommendationRef>,
        alternatives_presented: Vec<Alternative>,
        chosen_alternative_ref: Option<AlternativeId>,
        outcome: DecisionOutcome,
        reason: Option<String>,
        context_snapshot_ref: ContextSnapshotId,
        decided_at: Timestamp,
    ) -> Result<Self, DecisionViolation> {
        match (origin, recommendation_ref.is_some()) {
            (DecisionOrigin::SystemRecommended, false) => {
                return Err(DecisionViolation::MissingRecommendation)
            }
            (DecisionOrigin::UserInitiated, true) => {
                return Err(DecisionViolation::UnexpectedRecommendation)
            }
            _ => {}
        }
        let matching_outcome = matches!(
            (&decision_kind, &outcome),
            (
                DecisionKind::ProtocolResponse,
                DecisionOutcome::ProtocolResponse(_)
            ) | (
                DecisionKind::PursuitStatusChange,
                DecisionOutcome::PursuitStatusChange(_)
            ) | (
                DecisionKind::PursuitCompletion,
                DecisionOutcome::PursuitCompletion
            ) | (
                DecisionKind::AiProposalResponse,
                DecisionOutcome::AiProposalResponse(_)
            ) | (
                DecisionKind::PathCompositionChange,
                DecisionOutcome::PathCompositionChange(_)
            ) | (
                DecisionKind::PackSpecific(_),
                DecisionOutcome::PackSpecific(_)
            )
        );
        if !matching_outcome {
            return Err(DecisionViolation::OutcomeKindMismatch);
        }
        if decision_kind == DecisionKind::ProtocolResponse
            && origin != DecisionOrigin::SystemRecommended
        {
            return Err(DecisionViolation::ProtocolResponseMustBeSystemRecommended);
        }
        if decision_kind == DecisionKind::PursuitCompletion
            && origin != DecisionOrigin::UserInitiated
        {
            return Err(DecisionViolation::PursuitCompletionMustBeUserInitiated);
        }
        if decision_kind == DecisionKind::PursuitCompletion
            && !matches!(&subject_ref, TypedRef::Pursuit(_))
        {
            return Err(DecisionViolation::PursuitCompletionRequiresPursuitSubject);
        }
        if decision_kind == DecisionKind::AiProposalResponse
            && origin != DecisionOrigin::SystemRecommended
        {
            return Err(DecisionViolation::AiProposalResponseMustBeSystemRecommended);
        }
        if decision_kind == DecisionKind::AiProposalResponse
            && !matches!(&recommendation_ref, Some(RecommendationRef::AiProposal(_)))
        {
            return Err(DecisionViolation::AiProposalResponseRequiresProposalReference);
        }
        if decision_kind != DecisionKind::AiProposalResponse
            && matches!(&recommendation_ref, Some(RecommendationRef::AiProposal(_)))
        {
            return Err(DecisionViolation::AiProposalReferenceRequiresProposalResponse);
        }
        if decision_kind == DecisionKind::PathCompositionChange
            && origin != DecisionOrigin::UserInitiated
        {
            return Err(DecisionViolation::PathCompositionChangeMustBeUserInitiated);
        }
        if decision_kind == DecisionKind::PathCompositionChange
            && !matches!(&subject_ref, TypedRef::Pursuit(_))
        {
            return Err(DecisionViolation::PathCompositionChangeRequiresPursuitSubject);
        }
        Ok(Self {
            id,
            origin,
            decision_kind,
            subject_ref,
            recommendation_ref,
            alternatives_presented,
            chosen_alternative_ref,
            outcome,
            reason,
            context_snapshot_ref,
            decided_at,
        })
    }

    /// Stable identifier.
    pub fn id(&self) -> &DecisionRecordId {
        &self.id
    }

    /// Decision origin.
    pub const fn origin(&self) -> DecisionOrigin {
        self.origin
    }

    /// Decision kind discriminator.
    pub fn decision_kind(&self) -> &DecisionKind {
        &self.decision_kind
    }

    /// Decision subject.
    pub fn subject_ref(&self) -> &TypedRef {
        &self.subject_ref
    }

    /// Recommendation prompting the decision, when system-recommended.
    pub fn recommendation_ref(&self) -> Option<&RecommendationRef> {
        self.recommendation_ref.as_ref()
    }

    /// Alternatives actually presented.
    pub fn alternatives_presented(&self) -> &[Alternative] {
        &self.alternatives_presented
    }

    /// Chosen alternative, when one was selected.
    pub fn chosen_alternative_ref(&self) -> Option<&AlternativeId> {
        self.chosen_alternative_ref.as_ref()
    }

    /// Typed decision outcome.
    pub fn outcome(&self) -> &DecisionOutcome {
        &self.outcome
    }

    /// Optional user reason.
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    /// Context snapshot that informed the decision.
    pub fn context_snapshot_ref(&self) -> &ContextSnapshotId {
        &self.context_snapshot_ref
    }

    /// Decision time.
    pub fn decided_at(&self) -> &Timestamp {
        &self.decided_at
    }

    fn is_user_pack_decision(&self, suffix: &str) -> bool {
        self.origin == DecisionOrigin::UserInitiated
            && self.recommendation_ref.is_none()
            && matches!(
                &self.decision_kind,
                DecisionKind::PackSpecific(kind) if kind.as_str().ends_with(suffix)
            )
    }

    fn is_exact_user_pack_decision(&self, kind: &str) -> bool {
        self.origin == DecisionOrigin::UserInitiated
            && self.recommendation_ref.is_none()
            && matches!(
                &self.decision_kind,
                DecisionKind::PackSpecific(candidate) if candidate.as_str() == kind
            )
    }
}

/// Failure to match a Pursuit status transition with its authorizing decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PursuitTransitionViolation {
    /// The decision kind/outcome does not authorize the requested state.
    DecisionDoesNotAuthorize,
    /// The decision targets a different entity or Pursuit.
    SubjectMismatch,
}

impl Display for PursuitTransitionViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::DecisionDoesNotAuthorize => {
                "decision does not authorize the requested Pursuit status transition"
            }
            Self::SubjectMismatch => "decision does not target the Pursuit being transitioned",
        })
    }
}

impl Error for PursuitTransitionViolation {}

/// Checks the required decision kind/outcome for a Pursuit status transition.
pub fn check_pursuit_status_transition(
    target: PursuitStatus,
    decision: &DecisionRecord,
) -> Result<(), PursuitTransitionViolation> {
    let valid = match (target, decision.decision_kind(), decision.outcome()) {
        (
            PursuitStatus::Paused,
            DecisionKind::PursuitStatusChange,
            DecisionOutcome::PursuitStatusChange(PursuitAction::Pause),
        )
        | (
            PursuitStatus::Stopped,
            DecisionKind::PursuitStatusChange,
            DecisionOutcome::PursuitStatusChange(PursuitAction::Stop),
        )
        | (
            PursuitStatus::Replaced,
            DecisionKind::PursuitStatusChange,
            DecisionOutcome::PursuitStatusChange(PursuitAction::Replace),
        )
        | (
            PursuitStatus::Completed,
            DecisionKind::PursuitCompletion,
            DecisionOutcome::PursuitCompletion,
        ) => true,
        (PursuitStatus::Active, _, _) => false,
        _ => false,
    };
    valid
        .then_some(())
        .ok_or(PursuitTransitionViolation::DecisionDoesNotAuthorize)
}

/// Reflection medium, including an explicit no-content path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReflectionMedium {
    /// Paper reflection.
    Paper,
    /// Digital reflection.
    Digital,
    /// No reflection content recorded.
    None,
}

/// Structured outcome of reflection without requiring stored prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflectionOutcome {
    /// Stable identifier.
    pub id: ReflectionOutcomeId,
    /// Optional Session association.
    pub session_ref: Option<SessionId>,
    /// Optional Journal-owned entry pointer.
    pub journal_entry_ref: Option<OpaqueRef>,
    /// Chosen medium.
    pub medium: ReflectionMedium,
    /// Whether the reflection was completed.
    pub completed: bool,
    /// Optional structured fields.
    pub structured_fields: BTreeMap<String, LiteralValue>,
    /// Optional pack-private content pointer.
    pub pack_private_content_ref: Option<OpaqueRef>,
    /// Recording time.
    pub recorded_at: Timestamp,
}

/// Lifecycle status of a personal Calibration Record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalibrationStatus {
    /// Proposed but not decided.
    Proposed,
    /// Applied through a manifest override.
    Applied,
    /// Explicitly rejected.
    Rejected,
}

/// Personal calibration; structurally incapable of referencing Evidence Claims.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalibrationRecord {
    /// Stable identifier.
    pub id: CalibrationRecordId,
    /// Immutable Protocol Version being calibrated.
    pub protocol_ref: (ProtocolId, SemVer),
    /// Protocol parameter key.
    pub parameter_key: String,
    /// Proposed typed value.
    pub proposed_value: LiteralValue,
    /// Applied typed value, when accepted.
    pub applied_value: Option<LiteralValue>,
    /// Lifecycle status.
    pub status: CalibrationStatus,
    /// Personal Observation basis only.
    pub evidentiary_observation_refs: Vec<ObservationId>,
    /// Application time.
    pub applied_at: Option<Timestamp>,
    /// Back-reference to the manifest override.
    pub manifest_override_ref: Option<OpaqueRef>,
}

/// Finite, explicit Journal content scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalAccessScope {
    /// Explicit Journal entries.
    JournalEntries(Vec<Id>),
    /// Explicit Reflection Outcomes.
    ReflectionOutcomes(Vec<ReflectionOutcomeId>),
}

/// Invalid Journal Access Grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalAccessGrantViolation {
    /// A grant cannot have an empty explicit scope.
    EmptyScope,
    /// Grant creation must cite the required explicit user decision.
    InvalidGrantDecision,
    /// Revocation must cite the required explicit user decision.
    InvalidRevocationDecision,
}

impl Display for JournalAccessGrantViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EmptyScope => "Journal access scope must contain at least one explicit id",
            Self::InvalidGrantDecision => {
                "Journal access grant requires a USER_INITIATED journal.access_grant_created decision"
            }
            Self::InvalidRevocationDecision => {
                "Journal access revocation requires a USER_INITIATED journal.access_grant_revoked decision"
            }
        })
    }
}

impl Error for JournalAccessGrantViolation {}

/// Explicit, scoped cross-pack access to Journal content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalAccessGrant {
    id: crate::JournalAccessGrantId,
    grantee_pack_ref: PackId,
    scope: JournalAccessScope,
    purpose: String,
    granted_at: Timestamp,
    granted_via: DecisionRecordId,
    expires_at: Option<Timestamp>,
    revoked_at: Option<Timestamp>,
    revoked_via: Option<DecisionRecordId>,
}

impl JournalAccessGrant {
    /// Creates a finite grant from an explicit user decision.
    pub fn new(
        id: crate::JournalAccessGrantId,
        grantee_pack_ref: PackId,
        scope: JournalAccessScope,
        purpose: String,
        granted_at: Timestamp,
        granted_via: &DecisionRecord,
        expires_at: Option<Timestamp>,
    ) -> Result<Self, JournalAccessGrantViolation> {
        let is_empty = match &scope {
            JournalAccessScope::JournalEntries(ids) => ids.is_empty(),
            JournalAccessScope::ReflectionOutcomes(ids) => ids.is_empty(),
        };
        if is_empty {
            return Err(JournalAccessGrantViolation::EmptyScope);
        }
        if !granted_via.is_exact_user_pack_decision("journal.access_grant_created") {
            return Err(JournalAccessGrantViolation::InvalidGrantDecision);
        }
        Ok(Self {
            id,
            grantee_pack_ref,
            scope,
            purpose,
            granted_at,
            granted_via: granted_via.id().clone(),
            expires_at,
            revoked_at: None,
            revoked_via: None,
        })
    }

    /// Records forward-only revocation; previously produced artifacts are not erased here.
    pub fn revoke(
        &mut self,
        revoked_at: Timestamp,
        revoked_via: &DecisionRecord,
    ) -> Result<(), JournalAccessGrantViolation> {
        if !revoked_via.is_exact_user_pack_decision("journal.access_grant_revoked") {
            return Err(JournalAccessGrantViolation::InvalidRevocationDecision);
        }
        self.revoked_at = Some(revoked_at);
        self.revoked_via = Some(revoked_via.id().clone());
        Ok(())
    }

    /// Stable identifier.
    pub fn id(&self) -> &crate::JournalAccessGrantId {
        &self.id
    }

    /// Grantee Capability Pack.
    pub fn grantee_pack_ref(&self) -> &PackId {
        &self.grantee_pack_ref
    }

    /// Finite grant scope.
    pub fn scope(&self) -> &JournalAccessScope {
        &self.scope
    }

    /// User-readable purpose disclosure.
    pub fn purpose(&self) -> &str {
        &self.purpose
    }

    /// Grant time.
    pub fn granted_at(&self) -> &Timestamp {
        &self.granted_at
    }

    /// Explicit decision authorizing the grant.
    pub fn granted_via(&self) -> &DecisionRecordId {
        &self.granted_via
    }

    /// Optional expiry.
    pub fn expires_at(&self) -> Option<&Timestamp> {
        self.expires_at.as_ref()
    }

    /// Forward-only revocation time.
    pub fn revoked_at(&self) -> Option<&Timestamp> {
        self.revoked_at.as_ref()
    }

    /// Explicit decision authorizing revocation.
    pub fn revoked_via(&self) -> Option<&DecisionRecordId> {
        self.revoked_via.as_ref()
    }
}

/// Primary declared bound of a Commitment Device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitmentBoundBy {
    /// Scope alone bounds the commitment.
    Scope,
    /// A time interval is required.
    Time,
    /// A declarative condition is required.
    Condition,
}

/// Explicit time interval for a Commitment Device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeBound {
    /// Inclusive start.
    pub starts_at: Timestamp,
    /// End boundary.
    pub ends_at: Timestamp,
}

/// Commitment Device lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitmentDeviceStatus {
    /// Created but not active.
    Armed,
    /// Active.
    Active,
    /// Completed normally.
    Completed,
    /// Explicitly aborted by the user.
    Aborted,
}

/// Invalid Commitment Device construction or transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitmentDeviceViolation {
    /// `bound_by = TIME` requires a time bound.
    MissingTimeBound,
    /// `bound_by = CONDITION` requires a condition.
    MissingConditionBound,
    /// Time bounds must increase.
    InvalidTimeRange,
    /// Mechanism identifiers cannot be empty.
    EmptyMechanism,
    /// Abort semantics must be transparent.
    EmptyAbortSemantics,
    /// Creation must cite an explicit user commitment decision.
    InvalidCreationDecision,
    /// Abort must cite an explicit user abort decision.
    InvalidAbortDecision,
    /// Condition references are prohibited or unresolved.
    ConditionBound(ConditionBoundViolation),
}

impl Display for CommitmentDeviceViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTimeBound => formatter.write_str("TIME commitment requires time_bound"),
            Self::MissingConditionBound => {
                formatter.write_str("CONDITION commitment requires condition_bound")
            }
            Self::InvalidTimeRange => {
                formatter.write_str("CommitmentDevice time bound must end after it starts")
            }
            Self::EmptyMechanism => formatter.write_str("mechanism must not be empty"),
            Self::EmptyAbortSemantics => formatter.write_str("abort_semantics must not be empty"),
            Self::InvalidCreationDecision => formatter.write_str(
                "CommitmentDevice requires a USER_INITIATED commitment_device_start decision",
            ),
            Self::InvalidAbortDecision => formatter.write_str(
                "CommitmentDevice abort requires a USER_INITIATED commitment_device_abort decision",
            ),
            Self::ConditionBound(violation) => Display::fmt(violation, formatter),
        }
    }
}

impl Error for CommitmentDeviceViolation {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ConditionBound(violation) => Some(violation),
            _ => None,
        }
    }
}

impl From<ConditionBoundViolation> for CommitmentDeviceViolation {
    fn from(value: ConditionBoundViolation) -> Self {
        Self::ConditionBound(value)
    }
}

/// Explicit, transparent, reversible user-created constraint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitmentDevice {
    id: CommitmentDeviceId,
    scope_ref: ScopeRef,
    bound_by: CommitmentBoundBy,
    time_bound: Option<TimeBound>,
    condition_bound: Option<Predicate>,
    mechanism: String,
    abort_semantics: String,
    status: CommitmentDeviceStatus,
    created_via: DecisionRecordId,
    aborted_via: Option<DecisionRecordId>,
}

impl CommitmentDevice {
    /// Constructs a Commitment Device and always routes any condition through
    /// the independent `check_condition_bound_eligibility` rule.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: CommitmentDeviceId,
        scope_ref: ScopeRef,
        bound_by: CommitmentBoundBy,
        time_bound: Option<TimeBound>,
        condition_bound: Option<Predicate>,
        mechanism: String,
        abort_semantics: String,
        status: CommitmentDeviceStatus,
        created_via: &DecisionRecord,
        fact_registry: &dyn ContextFactRegistry,
        topic_registry: &dyn EventTopicRegistry,
    ) -> Result<Self, CommitmentDeviceViolation> {
        if bound_by == CommitmentBoundBy::Time && time_bound.is_none() {
            return Err(CommitmentDeviceViolation::MissingTimeBound);
        }
        if bound_by == CommitmentBoundBy::Condition && condition_bound.is_none() {
            return Err(CommitmentDeviceViolation::MissingConditionBound);
        }
        if let Some(time_bound) = &time_bound {
            if time_bound.starts_at >= time_bound.ends_at {
                return Err(CommitmentDeviceViolation::InvalidTimeRange);
            }
        }
        if mechanism.trim().is_empty() {
            return Err(CommitmentDeviceViolation::EmptyMechanism);
        }
        if abort_semantics.trim().is_empty() {
            return Err(CommitmentDeviceViolation::EmptyAbortSemantics);
        }
        if !created_via.is_user_pack_decision(".commitment_device_start") {
            return Err(CommitmentDeviceViolation::InvalidCreationDecision);
        }
        if let Some(predicate) = &condition_bound {
            check_condition_bound_eligibility(predicate, fact_registry, topic_registry)?;
        }
        Ok(Self {
            id,
            scope_ref,
            bound_by,
            time_bound,
            condition_bound,
            mechanism,
            abort_semantics,
            status,
            created_via: created_via.id().clone(),
            aborted_via: None,
        })
    }

    /// Marks the device aborted through an explicit user decision.
    pub fn abort(&mut self, aborted_via: &DecisionRecord) -> Result<(), CommitmentDeviceViolation> {
        if !aborted_via.is_user_pack_decision(".commitment_device_abort") {
            return Err(CommitmentDeviceViolation::InvalidAbortDecision);
        }
        self.status = CommitmentDeviceStatus::Aborted;
        self.aborted_via = Some(aborted_via.id().clone());
        Ok(())
    }

    /// Stable identifier.
    pub fn id(&self) -> &CommitmentDeviceId {
        &self.id
    }

    /// Explicit scope.
    pub fn scope_ref(&self) -> &ScopeRef {
        &self.scope_ref
    }

    /// Primary bound kind.
    pub const fn bound_by(&self) -> CommitmentBoundBy {
        self.bound_by
    }

    /// Optional time bound.
    pub fn time_bound(&self) -> Option<&TimeBound> {
        self.time_bound.as_ref()
    }

    /// Validated declarative condition.
    pub fn condition_bound(&self) -> Option<&Predicate> {
        self.condition_bound.as_ref()
    }

    /// Pack-declared in-app/UI mechanism identifier.
    pub fn mechanism(&self) -> &str {
        &self.mechanism
    }

    /// Commitment Devices are always reversible/abortable.
    pub const fn reversible(&self) -> bool {
        true
    }

    /// Transparent abort behavior.
    pub fn abort_semantics(&self) -> &str {
        &self.abort_semantics
    }

    /// Current status.
    pub const fn status(&self) -> CommitmentDeviceStatus {
        self.status
    }

    /// Explicit decision that created the device.
    pub fn created_via(&self) -> &DecisionRecordId {
        &self.created_via
    }

    /// Explicit decision that aborted the device, when applicable.
    pub fn aborted_via(&self) -> Option<&DecisionRecordId> {
        self.aborted_via.as_ref()
    }
}
