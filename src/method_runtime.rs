//! Governed Method recommendation consideration over existing Flourish kernel services.
//!
//! This module deliberately does **not** create a `MethodRun` aggregate, scheduler, writer, or new
//! kernel service. It coordinates the already-authoritative Context Assembler, Protocol Runtime,
//! Recommendation Composer and Attention Arbiter boundaries. The result is either a reviewable
//! recommendation, a suppressed candidate, or an explicit `DO NOTHING` result. No durable user
//! state is changed here.

use crate::{
    AIProposal, AIProposalId, AIProposalPayload, ContextSnapshotId, EnforcementClass,
    MethodDefinition, MethodEvidenceRef, MethodEvidenceStatus, PersonalOutcomeContext,
    PersonalPattern, PersonalPatternId, Predicate, Protocol, ProtocolAlternative, RecommendationId,
    Timestamp, TypedRef, VersionedMethodRef, VersionedProtocolRef,
};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// How consideration of a Method entered the coordinator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodInitiation {
    UserRequest,
    ProactiveTrigger,
}

/// Whether the Method may be considered without an explicit user request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserInitiationRequirement {
    Required,
    Optional,
}

/// Runtime policy supplied by the owning capability. This is not durable Method truth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodRuntimePolicy {
    pub user_initiation: UserInitiationRequirement,
    pub contraindications: Vec<Predicate>,
}

/// Source-independent request to consider one exact Method/Protocol pair.
pub struct MethodRuntimeRequest<'a> {
    pub subject_ref: TypedRef,
    pub method: &'a MethodDefinition,
    pub protocol: &'a Protocol,
    pub initiation: MethodInitiation,
    pub policy: &'a MethodRuntimePolicy,
    /// Instant used only to reject expired immutable AI proposals.
    pub now: Timestamp,
    /// Optional AI proposal that selected this exact Protocol for this exact subject.
    /// Presence never grants authority by itself; it is revalidated below.
    pub source_proposal: Option<&'a AIProposal>,
    /// Personal capability-reliability evidence available for a scaffold-fade candidate. Empty when
    /// none exists, which is an ordinary state and never implies unreliable capability.
    pub reliability_evidence: &'a [PersonalPattern],
    /// Optional pack-supplied fade policy. `None` means this request never proposes reducing
    /// support. The kernel never hard-codes a universal reliability threshold; only a pack policy
    /// may decide that personal evidence justifies fading scaffolding.
    pub fade_policy: Option<&'a dyn ScaffoldReliabilityPolicy>,
}

/// Locally assembled context. The snapshot reference is the durable audit link. Personal outcome
/// context, when present, has already crossed the validated personal-learning bridge; it remains
/// personal evidence and cannot become Scientific Evidence through this type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssembledMethodContext {
    pub snapshot_ref: ContextSnapshotId,
    pub personal_outcome_context: Option<PersonalOutcomeContext>,
}

/// Adapter to the existing Context Assembler service.
pub trait ContextAssembler {
    fn assemble(
        &self,
        subject_ref: &TypedRef,
        protocol: &Protocol,
    ) -> Result<AssembledMethodContext, String>;
}

/// Result of evaluating the Protocol against assembled context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolAssessment {
    pub eligible: bool,
    pub trigger_satisfied: bool,
    pub recommendation_policy_satisfied: bool,
    pub contraindication: Option<String>,
}

/// Adapter to the existing Protocol Runtime service.
pub trait ProtocolRuntime {
    fn assess(
        &self,
        protocol: &Protocol,
        contraindications: &[Predicate],
        context: &AssembledMethodContext,
    ) -> Result<ProtocolAssessment, String>;
}

/// User-facing burden disclosure from the governed Method definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BurdenDisclosure {
    Declared(Vec<String>),
    NotSpecified,
}

/// Evidence provenance carried into the recommendation without changing evidence authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodEvidenceDisclosure {
    pub status: MethodEvidenceStatus,
    pub method_evidence_refs: Vec<MethodEvidenceRef>,
    pub protocol_rationale_refs: Vec<crate::OpaqueRef>,
}

/// A single typed reduction of Method/Protocol support. `Unchanged` is a complete, valid result:
/// "do nothing" is always available and never implies unreliable capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaffoldFadeStep {
    Unchanged,
    Reduced,
    Removed,
}

/// Evidence- and provenance-aware candidate for reducing scaffolding. It is only ever a proposal:
/// composing it mutates nothing, it cites its personal-pattern basis for audit, it stays reversible
/// through the Protocol's own `reversal_semantics`, and it can never write capability/mastery truth
/// (it reads [`PersonalPattern`], which is personal evidence, never Scientific Evidence).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaffoldFadeCandidate {
    pub step: ScaffoldFadeStep,
    pub basis_pattern_refs: Vec<PersonalPatternId>,
    pub reversal_semantics: String,
}

/// A pack policy's exact, attributable fade decision: the step plus the precise subset of the
/// offered evidence it actually relied on. The coordinator never assumes every pattern it was
/// handed supports the step; it only accepts the ids the policy names here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaffoldFadeAssessment {
    pub step: ScaffoldFadeStep,
    pub basis_pattern_refs: Vec<PersonalPatternId>,
}

/// Pack-supplied policy deciding whether personal capability-reliability evidence justifies a
/// scaffold-fade candidate. The kernel intentionally has no built-in implementation and no
/// universal numeric threshold; only a pack/protocol policy may decide this.
pub trait ScaffoldReliabilityPolicy {
    fn evaluate(&self, reliability_evidence: &[PersonalPattern]) -> ScaffoldFadeAssessment;
}

/// A pack policy's fade decision could not be trusted as composed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaffoldFadeViolation {
    /// `Reduced`/`Removed` was returned with no cited evidence basis at all.
    MissingBasis,
    /// The same Personal Pattern id was cited more than once as basis.
    DuplicateBasisPattern,
    /// A cited basis id does not name any pattern the coordinator actually offered the policy.
    UnknownBasisPattern,
    /// A cited basis pattern belongs to a different subject than the one being considered.
    BasisSubjectMismatch,
    /// A cited basis pattern is not typed reliability evidence for the exact Method/Protocol whose
    /// support is being faded (same subject is not enough: it must be evidence about this Method).
    BasisMethodMismatch,
}

impl Display for ScaffoldFadeViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingBasis => {
                "a scaffold-fade policy reducing or removing support must cite a non-empty evidence basis"
            }
            Self::DuplicateBasisPattern => {
                "a scaffold-fade evidence basis cannot cite the same Personal Pattern twice"
            }
            Self::UnknownBasisPattern => {
                "a scaffold-fade evidence basis cited a Personal Pattern that was never offered as reliability evidence"
            }
            Self::BasisSubjectMismatch => {
                "a scaffold-fade evidence basis cited a Personal Pattern belonging to a different subject"
            }
            Self::BasisMethodMismatch => {
                "a scaffold-fade evidence basis cited a Personal Pattern that is not reliability evidence for this exact Method"
            }
        })
    }
}

impl Error for ScaffoldFadeViolation {}

/// Existing Protocol calibration/disable semantics exposed without applying any calibration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodAdaptationDisclosure {
    pub calibration_enabled: bool,
    pub observation_types: Vec<crate::ObservationType>,
    pub reversal_semantics: String,
    pub disabled_semantics: String,
    /// Typed, overrideable candidate for reducing support. The user's existing Accept/Modify/
    /// Decline response to the surrounding recommendation is what actually applies or overrides it;
    /// composing the candidate never changes Method/Protocol/capability truth by itself.
    pub fade_candidate: ScaffoldFadeCandidate,
}

/// Actions kept visible for behavior-facing recommendations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationAction {
    Accept,
    Modify,
    Decline,
    Defer,
}

/// Complete typed input delegated to the Recommendation Composer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecommendationComposition {
    pub subject_ref: TypedRef,
    pub method_ref: VersionedMethodRef,
    pub protocol_ref: VersionedProtocolRef,
    pub reason: String,
    pub alternatives: Vec<ProtocolAlternative>,
    pub actions: Vec<RecommendationAction>,
    pub burden: BurdenDisclosure,
    pub boundaries: Vec<String>,
    pub risks_and_limitations: Vec<String>,
    pub evidence: MethodEvidenceDisclosure,
    pub adaptation: MethodAdaptationDisclosure,
    pub user_initiation: UserInitiationRequirement,
    pub context_snapshot_ref: ContextSnapshotId,
    /// Read-only personal evidence available when composing/explaining this recommendation.
    /// Its provenance and epistemic labels remain intact; the recommendation cannot rewrite it.
    pub personal_outcome_context: Option<PersonalOutcomeContext>,
    pub source_proposal_ref: Option<AIProposalId>,
}

/// Materialized recommendation returned by the Recommendation Composer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodRecommendation {
    pub id: RecommendationId,
    pub recommendation: String,
    pub composition: RecommendationComposition,
}

/// Adapter to the existing Recommendation Composer service.
pub trait RecommendationComposer {
    fn compose(
        &self,
        composition: RecommendationComposition,
    ) -> Result<MethodRecommendation, String>;
}

/// Sole proactive-attention authority. Suppression preserves the candidate for history/overflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttentionDecision {
    Surface,
    Suppress { reason: String },
}

pub trait AttentionArbiter {
    fn decide(
        &self,
        recommendation: &MethodRecommendation,
        initiation: MethodInitiation,
    ) -> AttentionDecision;
}

/// `DO NOTHING` is a valid successful system result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoNothingReason {
    UserInitiationRequired,
    Ineligible,
    TriggerNotSatisfied,
    RecommendationPolicyNotSatisfied,
    Contraindicated(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodConsideration {
    Surface(MethodRecommendation),
    Suppressed {
        recommendation: MethodRecommendation,
        reason: String,
    },
    DoNothing(DoNothingReason),
}

/// Why an AI proposal cannot lend provenance to this exact Method/Protocol consideration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodProposalAuthorizationViolation {
    WrongPayloadKind,
    ProtocolMismatch,
    TargetMismatch,
    Expired,
}

impl Display for MethodProposalAuthorizationViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WrongPayloadKind => {
                "AI proposal is not a Protocol selection and cannot authorize this Method recommendation"
            }
            Self::ProtocolMismatch => {
                "AI proposal selected a different Protocol than the one being considered"
            }
            Self::TargetMismatch => {
                "AI proposal targets a different subject than the one being considered"
            }
            Self::Expired => "AI proposal has expired and cannot authorize this recommendation",
        })
    }
}

impl Error for MethodProposalAuthorizationViolation {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodRuntimeError {
    MethodProtocolMismatch,
    ProtocolVariantNotRegistered,
    BehaviorMethodMustBeAdvisory,
    BehaviorMethodMustAllowDeferral,
    BehaviorMethodRequiresAlternative,
    PersonalContextSubjectMismatch,
    PersonalContextProtocolMismatch,
    ProposalAuthorization(MethodProposalAuthorizationViolation),
    ScaffoldFadeEvidence(ScaffoldFadeViolation),
    ContextAssembly(String),
    ProtocolEvaluation(String),
    RecommendationComposition(String),
}

impl Display for MethodRuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MethodProtocolMismatch => {
                formatter.write_str("Protocol method_ref does not name the requested Method")
            }
            Self::ProtocolVariantNotRegistered => {
                formatter.write_str("Method does not register the requested Protocol as a variant")
            }
            Self::BehaviorMethodMustBeAdvisory => {
                formatter.write_str("behavior-facing Method Protocol must be ADVISORY")
            }
            Self::BehaviorMethodMustAllowDeferral => {
                formatter.write_str("behavior-facing Method Protocol must allow not-now/defer")
            }
            Self::BehaviorMethodRequiresAlternative => {
                formatter.write_str("behavior-facing Method Protocol requires a meaningful alternative")
            }
            Self::PersonalContextSubjectMismatch => formatter.write_str(
                "validated personal outcome context targets a different recommendation subject",
            ),
            Self::PersonalContextProtocolMismatch => formatter.write_str(
                "validated personal outcome context targets a different Protocol version",
            ),
            Self::ProposalAuthorization(violation) => Display::fmt(violation, formatter),
            Self::ScaffoldFadeEvidence(violation) => Display::fmt(violation, formatter),
            Self::ContextAssembly(message) => write!(formatter, "context assembly failed: {message}"),
            Self::ProtocolEvaluation(message) => {
                write!(formatter, "Protocol evaluation failed: {message}")
            }
            Self::RecommendationComposition(message) => {
                write!(formatter, "recommendation composition failed: {message}")
            }
        }
    }
}

impl Error for MethodRuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ProposalAuthorization(violation) => Some(violation),
            Self::ScaffoldFadeEvidence(violation) => Some(violation),
            _ => None,
        }
    }
}

/// Additive coordinator. It is intentionally absent from `ServiceId`.
pub struct MethodRuntimeCoordinator<'a, C, P, R, A> {
    context_assembler: &'a C,
    protocol_runtime: &'a P,
    recommendation_composer: &'a R,
    attention_arbiter: &'a A,
}

impl<'a, C, P, R, A> MethodRuntimeCoordinator<'a, C, P, R, A>
where
    C: ContextAssembler,
    P: ProtocolRuntime,
    R: RecommendationComposer,
    A: AttentionArbiter,
{
    pub const fn new(
        context_assembler: &'a C,
        protocol_runtime: &'a P,
        recommendation_composer: &'a R,
        attention_arbiter: &'a A,
    ) -> Self {
        Self {
            context_assembler,
            protocol_runtime,
            recommendation_composer,
            attention_arbiter,
        }
    }

    /// Runs eligibility, recommendation composition and attention arbitration without any writer.
    pub fn consider(
        &self,
        request: MethodRuntimeRequest<'_>,
    ) -> Result<MethodConsideration, MethodRuntimeError> {
        validate_method_protocol(request.method, request.protocol)?;

        if request.policy.user_initiation == UserInitiationRequirement::Required
            && request.initiation == MethodInitiation::ProactiveTrigger
        {
            return Ok(MethodConsideration::DoNothing(
                DoNothingReason::UserInitiationRequired,
            ));
        }

        let context = self
            .context_assembler
            .assemble(&request.subject_ref, request.protocol)
            .map_err(MethodRuntimeError::ContextAssembly)?;
        validate_personal_context(&request, &context)?;

        let assessment = self
            .protocol_runtime
            .assess(
                request.protocol,
                &request.policy.contraindications,
                &context,
            )
            .map_err(MethodRuntimeError::ProtocolEvaluation)?;

        if !assessment.eligible {
            return Ok(MethodConsideration::DoNothing(DoNothingReason::Ineligible));
        }
        if let Some(reason) = assessment.contraindication {
            return Ok(MethodConsideration::DoNothing(
                DoNothingReason::Contraindicated(reason),
            ));
        }
        if request.initiation == MethodInitiation::ProactiveTrigger && !assessment.trigger_satisfied
        {
            return Ok(MethodConsideration::DoNothing(
                DoNothingReason::TriggerNotSatisfied,
            ));
        }
        if !assessment.recommendation_policy_satisfied {
            return Ok(MethodConsideration::DoNothing(
                DoNothingReason::RecommendationPolicyNotSatisfied,
            ));
        }

        let composition = recommendation_composition(&request, &context)?;
        let recommendation = self
            .recommendation_composer
            .compose(composition)
            .map_err(MethodRuntimeError::RecommendationComposition)?;

        match self
            .attention_arbiter
            .decide(&recommendation, request.initiation)
        {
            AttentionDecision::Surface => Ok(MethodConsideration::Surface(recommendation)),
            AttentionDecision::Suppress { reason } => Ok(MethodConsideration::Suppressed {
                recommendation,
                reason,
            }),
        }
    }
}

fn validate_method_protocol(
    method: &MethodDefinition,
    protocol: &Protocol,
) -> Result<(), MethodRuntimeError> {
    let method_ref = VersionedMethodRef {
        method_id: method.id.clone(),
        method_version: method.version.clone(),
    };
    if protocol.method_ref() != Some(&method_ref) {
        return Err(MethodRuntimeError::MethodProtocolMismatch);
    }

    let protocol_ref = VersionedProtocolRef {
        protocol_id: protocol.id().clone(),
        protocol_version: protocol.version().clone(),
    };
    if !method.protocol_variants.contains(&protocol_ref) {
        return Err(MethodRuntimeError::ProtocolVariantNotRegistered);
    }
    if protocol.enforcement_class() != EnforcementClass::Advisory {
        return Err(MethodRuntimeError::BehaviorMethodMustBeAdvisory);
    }
    if !protocol.recommendation_policy().allows_not_now {
        return Err(MethodRuntimeError::BehaviorMethodMustAllowDeferral);
    }
    if protocol.alternatives().is_empty() {
        return Err(MethodRuntimeError::BehaviorMethodRequiresAlternative);
    }
    Ok(())
}

fn validate_personal_context(
    request: &MethodRuntimeRequest<'_>,
    context: &AssembledMethodContext,
) -> Result<(), MethodRuntimeError> {
    let Some(personal) = context.personal_outcome_context.as_ref() else {
        return Ok(());
    };
    if personal.subject_ref() != &request.subject_ref {
        return Err(MethodRuntimeError::PersonalContextSubjectMismatch);
    }
    let expected_protocol = VersionedProtocolRef {
        protocol_id: request.protocol.id().clone(),
        protocol_version: request.protocol.version().clone(),
    };
    if personal.protocol_ref() != &expected_protocol {
        return Err(MethodRuntimeError::PersonalContextProtocolMismatch);
    }
    Ok(())
}

fn authorize_source_proposal(
    proposal: &AIProposal,
    subject_ref: &TypedRef,
    protocol_ref: &VersionedProtocolRef,
    now: &Timestamp,
) -> Result<AIProposalId, MethodProposalAuthorizationViolation> {
    let AIProposalPayload::ProtocolSelection(selected_protocol_ref) = proposal.payload() else {
        return Err(MethodProposalAuthorizationViolation::WrongPayloadKind);
    };
    if selected_protocol_ref != protocol_ref {
        return Err(MethodProposalAuthorizationViolation::ProtocolMismatch);
    }
    if &proposal.target().target_ref != subject_ref {
        return Err(MethodProposalAuthorizationViolation::TargetMismatch);
    }
    if proposal.is_expired_at(now) {
        return Err(MethodProposalAuthorizationViolation::Expired);
    }
    Ok(proposal.id().clone())
}

/// Composes the fade candidate a pack policy proposes, and refuses to carry a decision it cannot
/// attribute. The kernel supplies no reliability threshold of its own: without a policy, or when
/// the policy returns `Unchanged`, support stays as it is. When a policy does propose reducing
/// support, every id it cites must name a pattern the coordinator actually offered it, belonging to
/// the exact subject under consideration, and typed as reliability evidence for this exact Method —
/// sharing a subject is not evidence that support for *this* capability is safe to reduce.
fn scaffold_fade_candidate(
    request: &MethodRuntimeRequest<'_>,
) -> Result<ScaffoldFadeCandidate, ScaffoldFadeViolation> {
    let reversal_semantics = request
        .protocol
        .calibration_policy()
        .reversal_semantics
        .clone();
    let unchanged = |reversal_semantics| ScaffoldFadeCandidate {
        step: ScaffoldFadeStep::Unchanged,
        basis_pattern_refs: Vec::new(),
        reversal_semantics,
    };
    let Some(policy) = request.fade_policy else {
        return Ok(unchanged(reversal_semantics));
    };
    let assessment = policy.evaluate(request.reliability_evidence);
    if assessment.step == ScaffoldFadeStep::Unchanged {
        return Ok(unchanged(reversal_semantics));
    }
    if assessment.basis_pattern_refs.is_empty() {
        return Err(ScaffoldFadeViolation::MissingBasis);
    }
    let unique_basis: BTreeSet<_> = assessment.basis_pattern_refs.iter().collect();
    if unique_basis.len() != assessment.basis_pattern_refs.len() {
        return Err(ScaffoldFadeViolation::DuplicateBasisPattern);
    }
    let target_method_ref = VersionedMethodRef {
        method_id: request.method.id.clone(),
        method_version: request.method.version.clone(),
    };
    for pattern_id in &assessment.basis_pattern_refs {
        let pattern = request
            .reliability_evidence
            .iter()
            .find(|candidate| candidate.id() == pattern_id)
            .ok_or(ScaffoldFadeViolation::UnknownBasisPattern)?;
        if pattern.subject_ref() != &request.subject_ref {
            return Err(ScaffoldFadeViolation::BasisSubjectMismatch);
        }
        if pattern.relates_to_method() != Some(&target_method_ref) {
            return Err(ScaffoldFadeViolation::BasisMethodMismatch);
        }
    }
    Ok(ScaffoldFadeCandidate {
        step: assessment.step,
        basis_pattern_refs: assessment.basis_pattern_refs,
        reversal_semantics,
    })
}

fn recommendation_composition(
    request: &MethodRuntimeRequest<'_>,
    context: &AssembledMethodContext,
) -> Result<RecommendationComposition, MethodRuntimeError> {
    let protocol_ref = VersionedProtocolRef {
        protocol_id: request.protocol.id().clone(),
        protocol_version: request.protocol.version().clone(),
    };
    let source_proposal_ref = request
        .source_proposal
        .map(|proposal| {
            authorize_source_proposal(proposal, &request.subject_ref, &protocol_ref, &request.now)
        })
        .transpose()
        .map_err(MethodRuntimeError::ProposalAuthorization)?;
    let fade_candidate =
        scaffold_fade_candidate(request).map_err(MethodRuntimeError::ScaffoldFadeEvidence)?;

    Ok(RecommendationComposition {
        subject_ref: request.subject_ref.clone(),
        method_ref: VersionedMethodRef {
            method_id: request.method.id.clone(),
            method_version: request.method.version.clone(),
        },
        protocol_ref,
        reason: request.protocol.purpose().to_owned(),
        alternatives: request.protocol.alternatives().to_vec(),
        actions: vec![
            RecommendationAction::Accept,
            RecommendationAction::Modify,
            RecommendationAction::Decline,
            RecommendationAction::Defer,
        ],
        burden: if request.method.costs.is_empty() {
            BurdenDisclosure::NotSpecified
        } else {
            BurdenDisclosure::Declared(request.method.costs.clone())
        },
        boundaries: request.method.boundary_conditions.clone(),
        risks_and_limitations: request.method.risks_and_limitations.clone(),
        evidence: MethodEvidenceDisclosure {
            status: request.method.evidence_status,
            method_evidence_refs: request.method.evidence_refs.clone(),
            protocol_rationale_refs: request.protocol.rationale_refs().to_vec(),
        },
        adaptation: MethodAdaptationDisclosure {
            calibration_enabled: request.protocol.calibration_policy().enabled,
            observation_types: request
                .protocol
                .calibration_policy()
                .observation_types
                .clone(),
            reversal_semantics: request
                .protocol
                .calibration_policy()
                .reversal_semantics
                .clone(),
            disabled_semantics: request.protocol.disabled_semantics().to_owned(),
            fade_candidate,
        },
        user_initiation: request.policy.user_initiation,
        context_snapshot_ref: context.snapshot_ref.clone(),
        personal_outcome_context: context.personal_outcome_context.clone(),
        source_proposal_ref,
    })
}
