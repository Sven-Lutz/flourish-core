use flourish_core::method_runtime::{
    AssembledMethodContext, AttentionArbiter, AttentionDecision, BurdenDisclosure,
    ContextAssembler, DoNothingReason, MethodConsideration, MethodInitiation,
    MethodRuntimeCoordinator, MethodRuntimeError, MethodRuntimePolicy, MethodRuntimeRequest,
    ProtocolAssessment, ProtocolRuntime, RecommendationAction, RecommendationComposer,
    RecommendationComposition, ScaffoldFadeAssessment, ScaffoldFadeCandidate, ScaffoldFadeStep,
    ScaffoldFadeViolation, ScaffoldReliabilityPolicy, UserInitiationRequirement,
};
use flourish_core::*;
use std::cell::Cell;

const ID_PURSUIT: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
const ID_CONTEXT: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAY";
const ID_RECOMMENDATION: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAZ";

fn timestamp() -> Timestamp {
    Timestamp::new("2026-09-07T17:00:00.000Z").unwrap()
}

fn method_ref() -> VersionedMethodRef {
    VersionedMethodRef {
        method_id: MethodId::new("method.after_action_review").unwrap(),
        method_version: SemVer::new("1.0.0").unwrap(),
    }
}

fn protocol_ref() -> VersionedProtocolRef {
    VersionedProtocolRef {
        protocol_id: ProtocolId::new("reflection.after_action_review").unwrap(),
        protocol_version: SemVer::new("1.0.0").unwrap(),
    }
}

fn method() -> MethodDefinition {
    MethodDefinition::new(
        method_ref().method_id,
        method_ref().method_version,
        "After-action review",
        "reflection",
        "Learn from a completed attempt",
        vec!["Identify a useful adjustment".into()],
        Some("Separate observation, interpretation, and next action".into()),
        vec![MethodEvidenceRef::EvidenceClaim(
            EvidenceClaimId::new("evidence.reflection.boundaries").unwrap(),
        )],
        MethodEvidenceStatus::MixedContextDependent,
        vec!["Adults reviewing completed work".into()],
        vec!["After a bounded practice session".into()],
        vec!["One review does not establish a recurring pattern".into()],
        vec!["About five minutes of reflection".into()],
        vec!["May add burden when the user is already overloaded".into()],
        vec![],
        vec![],
        vec![protocol_ref()],
        vec![MethodAiRole::Reflection],
        vec![],
        None,
        LiteralValue::Text("catalog.methods.after_action_review.v1".into()),
    )
    .unwrap()
}

#[derive(Default)]
struct EmptyRegistries;

impl ContextFactRegistry for EmptyRegistries {
    fn resolve_fact(&self, _: &ContextFactRef) -> Option<&ContextFactDefinition> {
        None
    }
}

impl EventTopicRegistry for EmptyRegistries {
    fn resolve_topic(&self, _: &EventTopic) -> Option<&EventTopicDefinition> {
        None
    }
}

fn protocol(enforcement: EnforcementClass) -> Protocol {
    let registries = EmptyRegistries;
    Protocol::new_with_method(
        protocol_ref().protocol_id,
        protocol_ref().protocol_version,
        PackId::new("reflection").unwrap(),
        enforcement,
        "A short review may help turn this completed attempt into a useful adjustment".into(),
        vec![],
        vec![],
        vec![],
        vec![OpaqueRef::new("design.after_action_review").unwrap()],
        RecommendationPolicy {
            conditions: vec![],
            allows_not_now: true,
        },
        vec![ProtocolAlternative {
            key: "skip_review".into(),
            label: "Leave the session as it is".into(),
            value: LiteralValue::Null,
        }],
        vec![ObservationType::new("reflection.review_outcome").unwrap()],
        CalibrationPolicy {
            enabled: true,
            observation_types: vec![ObservationType::new("reflection.review_outcome").unwrap()],
            reversal_semantics: "Remove the personal override".into(),
        },
        None,
        "Stops suggestions and leaves prior observations unchanged".into(),
        Some(method_ref()),
        &registries,
        &registries,
    )
    .unwrap()
}

struct FixedAssembler {
    calls: Cell<u32>,
}

impl ContextAssembler for FixedAssembler {
    fn assemble(&self, _: &TypedRef, _: &Protocol) -> Result<AssembledMethodContext, String> {
        self.calls.set(self.calls.get() + 1);
        Ok(AssembledMethodContext {
            snapshot_ref: ContextSnapshotId::parse(ID_CONTEXT).unwrap(),
            personal_outcome_context: None,
        })
    }
}

struct FixedProtocolRuntime {
    assessment: ProtocolAssessment,
}

impl ProtocolRuntime for FixedProtocolRuntime {
    fn assess(
        &self,
        _: &Protocol,
        _: &[Predicate],
        _: &AssembledMethodContext,
    ) -> Result<ProtocolAssessment, String> {
        Ok(self.assessment.clone())
    }
}

struct CapturingComposer;

impl RecommendationComposer for CapturingComposer {
    fn compose(
        &self,
        composition: RecommendationComposition,
    ) -> Result<flourish_core::method_runtime::MethodRecommendation, String> {
        Ok(flourish_core::method_runtime::MethodRecommendation {
            id: RecommendationId::parse(ID_RECOMMENDATION).unwrap(),
            recommendation: "Take five minutes to review what happened and choose one adjustment"
                .into(),
            composition,
        })
    }
}

struct FixedArbiter {
    decision: AttentionDecision,
    calls: Cell<u32>,
}

impl AttentionArbiter for FixedArbiter {
    fn decide(
        &self,
        _: &flourish_core::method_runtime::MethodRecommendation,
        _: MethodInitiation,
    ) -> AttentionDecision {
        self.calls.set(self.calls.get() + 1);
        self.decision.clone()
    }
}

fn eligible_assessment() -> ProtocolAssessment {
    ProtocolAssessment {
        eligible: true,
        trigger_satisfied: true,
        recommendation_policy_satisfied: true,
        contraindication: None,
    }
}

fn pursuit_ref() -> TypedRef {
    TypedRef::Pursuit(PursuitId::parse(ID_PURSUIT).unwrap())
}

#[test]
fn advisory_method_surfaces_with_evidence_burden_and_full_user_authority() {
    let method = method();
    let protocol = protocol(EnforcementClass::Advisory);
    let assembler = FixedAssembler { calls: Cell::new(0) };
    let runtime = FixedProtocolRuntime { assessment: eligible_assessment() };
    let composer = CapturingComposer;
    let arbiter = FixedArbiter { decision: AttentionDecision::Surface, calls: Cell::new(0) };
    let coordinator = MethodRuntimeCoordinator::new(&assembler, &runtime, &composer, &arbiter);
    let policy = MethodRuntimePolicy {
        user_initiation: UserInitiationRequirement::Optional,
        contraindications: vec![],
    };

    let result = coordinator
        .consider(MethodRuntimeRequest {
            subject_ref: pursuit_ref(),
            method: &method,
            protocol: &protocol,
            initiation: MethodInitiation::UserRequest,
            policy: &policy,
            now: timestamp(),
            source_proposal: None,
            reliability_evidence: &[],
            fade_policy: None,
        })
        .unwrap();

    let MethodConsideration::Surface(recommendation) = result else {
        panic!("expected surfaced recommendation");
    };
    assert_eq!(assembler.calls.get(), 1);
    assert_eq!(arbiter.calls.get(), 1);
    assert_eq!(
        recommendation.composition.actions,
        vec![
            RecommendationAction::Accept,
            RecommendationAction::Modify,
            RecommendationAction::Decline,
            RecommendationAction::Defer,
        ]
    );
    assert_eq!(
        recommendation.composition.burden,
        BurdenDisclosure::Declared(vec!["About five minutes of reflection".into()])
    );
    assert_eq!(recommendation.composition.boundaries, method.boundary_conditions);
    assert_eq!(
        recommendation.composition.evidence.method_evidence_refs,
        method.evidence_refs
    );
    assert!(recommendation.composition.adaptation.calibration_enabled);
    assert!(recommendation.composition.personal_outcome_context.is_none());
    assert!(recommendation.composition.source_proposal_ref.is_none());
}

#[test]
fn required_user_initiation_returns_do_nothing_before_context_work() {
    let method = method();
    let protocol = protocol(EnforcementClass::Advisory);
    let assembler = FixedAssembler { calls: Cell::new(0) };
    let runtime = FixedProtocolRuntime { assessment: eligible_assessment() };
    let composer = CapturingComposer;
    let arbiter = FixedArbiter { decision: AttentionDecision::Surface, calls: Cell::new(0) };
    let coordinator = MethodRuntimeCoordinator::new(&assembler, &runtime, &composer, &arbiter);
    let policy = MethodRuntimePolicy {
        user_initiation: UserInitiationRequirement::Required,
        contraindications: vec![],
    };

    let result = coordinator
        .consider(MethodRuntimeRequest {
            subject_ref: pursuit_ref(),
            method: &method,
            protocol: &protocol,
            initiation: MethodInitiation::ProactiveTrigger,
            policy: &policy,
            now: timestamp(),
            source_proposal: None,
            reliability_evidence: &[],
            fade_policy: None,
        })
        .unwrap();

    assert_eq!(result, MethodConsideration::DoNothing(DoNothingReason::UserInitiationRequired));
    assert_eq!(assembler.calls.get(), 0);
    assert_eq!(arbiter.calls.get(), 0);
}

#[test]
fn proactive_trigger_must_actually_be_satisfied() {
    let method = method();
    let protocol = protocol(EnforcementClass::Advisory);
    let assembler = FixedAssembler { calls: Cell::new(0) };
    let runtime = FixedProtocolRuntime {
        assessment: ProtocolAssessment {
            trigger_satisfied: false,
            ..eligible_assessment()
        },
    };
    let composer = CapturingComposer;
    let arbiter = FixedArbiter { decision: AttentionDecision::Surface, calls: Cell::new(0) };
    let coordinator = MethodRuntimeCoordinator::new(&assembler, &runtime, &composer, &arbiter);
    let policy = MethodRuntimePolicy {
        user_initiation: UserInitiationRequirement::Optional,
        contraindications: vec![],
    };

    let result = coordinator
        .consider(MethodRuntimeRequest {
            subject_ref: pursuit_ref(),
            method: &method,
            protocol: &protocol,
            initiation: MethodInitiation::ProactiveTrigger,
            policy: &policy,
            now: timestamp(),
            source_proposal: None,
            reliability_evidence: &[],
            fade_policy: None,
        })
        .unwrap();

    assert_eq!(result, MethodConsideration::DoNothing(DoNothingReason::TriggerNotSatisfied));
    assert_eq!(arbiter.calls.get(), 0);
}

#[test]
fn contraindication_is_a_successful_do_nothing_result() {
    let method = method();
    let protocol = protocol(EnforcementClass::Advisory);
    let assembler = FixedAssembler { calls: Cell::new(0) };
    let runtime = FixedProtocolRuntime {
        assessment: ProtocolAssessment {
            contraindication: Some("User asked not to reflect right now".into()),
            ..eligible_assessment()
        },
    };
    let composer = CapturingComposer;
    let arbiter = FixedArbiter { decision: AttentionDecision::Surface, calls: Cell::new(0) };
    let coordinator = MethodRuntimeCoordinator::new(&assembler, &runtime, &composer, &arbiter);
    let policy = MethodRuntimePolicy {
        user_initiation: UserInitiationRequirement::Optional,
        contraindications: vec![],
    };

    let result = coordinator
        .consider(MethodRuntimeRequest {
            subject_ref: pursuit_ref(),
            method: &method,
            protocol: &protocol,
            initiation: MethodInitiation::UserRequest,
            policy: &policy,
            now: timestamp(),
            source_proposal: None,
            reliability_evidence: &[],
            fade_policy: None,
        })
        .unwrap();

    assert_eq!(
        result,
        MethodConsideration::DoNothing(DoNothingReason::Contraindicated(
            "User asked not to reflect right now".into()
        ))
    );
    assert_eq!(arbiter.calls.get(), 0);
}

#[test]
fn behavior_method_cannot_use_guardrail_authority() {
    let method = method();
    let protocol = protocol(EnforcementClass::Guardrail);
    let assembler = FixedAssembler { calls: Cell::new(0) };
    let runtime = FixedProtocolRuntime { assessment: eligible_assessment() };
    let composer = CapturingComposer;
    let arbiter = FixedArbiter { decision: AttentionDecision::Surface, calls: Cell::new(0) };
    let coordinator = MethodRuntimeCoordinator::new(&assembler, &runtime, &composer, &arbiter);
    let policy = MethodRuntimePolicy {
        user_initiation: UserInitiationRequirement::Optional,
        contraindications: vec![],
    };

    let error = coordinator
        .consider(MethodRuntimeRequest {
            subject_ref: pursuit_ref(),
            method: &method,
            protocol: &protocol,
            initiation: MethodInitiation::UserRequest,
            policy: &policy,
            now: timestamp(),
            source_proposal: None,
            reliability_evidence: &[],
            fade_policy: None,
        })
        .unwrap_err();

    assert_eq!(error, MethodRuntimeError::BehaviorMethodMustBeAdvisory);
    assert_eq!(assembler.calls.get(), 0);
}

#[test]
fn attention_arbiter_can_suppress_without_fabricating_an_intervention() {
    let method = method();
    let protocol = protocol(EnforcementClass::Advisory);
    let assembler = FixedAssembler { calls: Cell::new(0) };
    let runtime = FixedProtocolRuntime { assessment: eligible_assessment() };
    let composer = CapturingComposer;
    let arbiter = FixedArbiter {
        decision: AttentionDecision::Suppress { reason: "attention budget is already occupied".into() },
        calls: Cell::new(0),
    };
    let coordinator = MethodRuntimeCoordinator::new(&assembler, &runtime, &composer, &arbiter);
    let policy = MethodRuntimePolicy {
        user_initiation: UserInitiationRequirement::Optional,
        contraindications: vec![],
    };

    let result = coordinator
        .consider(MethodRuntimeRequest {
            subject_ref: pursuit_ref(),
            method: &method,
            protocol: &protocol,
            initiation: MethodInitiation::ProactiveTrigger,
            policy: &policy,
            now: timestamp(),
            source_proposal: None,
            reliability_evidence: &[],
            fade_policy: None,
        })
        .unwrap();

    let MethodConsideration::Suppressed { reason, .. } = result else {
        panic!("expected suppression");
    };
    assert_eq!(reason, "attention budget is already occupied");
}

// ---------------------------------------------------------------------------
// Scaffold fade
//
// Reliable personal capability evidence may propose reducing a Method's scaffolding. The kernel
// owns no reliability threshold of its own: a pack-supplied policy decides, the coordinator only
// checks that the decision is attributable to evidence it actually offered, about this exact
// subject and this exact Method. Composing a candidate changes nothing by itself — the user's
// existing Accept/Modify/Decline response to the surrounding recommendation is what applies it.
// ---------------------------------------------------------------------------

const ID_OTHER_PURSUIT: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB0";
const ID_PATTERN: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB1";
const ID_OTHER_PATTERN: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB2";
const ID_UNOFFERED_PATTERN: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB3";
const ID_OBSERVATION_A: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB4";
const ID_OBSERVATION_B: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB5";

fn other_pursuit_ref() -> TypedRef {
    TypedRef::Pursuit(PursuitId::parse(ID_OTHER_PURSUIT).unwrap())
}

fn reliability_pattern_for_method(
    id: &str,
    subject_ref: TypedRef,
    relates_to_method: Option<VersionedMethodRef>,
) -> PersonalPattern {
    let refs = vec![
        ObservationId::parse(ID_OBSERVATION_A).unwrap(),
        ObservationId::parse(ID_OBSERVATION_B).unwrap(),
    ];
    PersonalPattern::new(
        PersonalPatternId::parse(id).unwrap(),
        subject_ref,
        PackNamespacedPatternKind::new("reflection.consistently_completed").unwrap(),
        "Across several reviews, the shorter prompt was reliably completed",
        EpistemicClass::Inferred,
        refs.clone(),
        DerivationRef {
            derived_by: OwningRef::CapabilityPack(PackId::new("reflection").unwrap()),
            derivation_version: "reflection.pattern.v1".into(),
            method: "Compare personal review completion across sessions".into(),
            source_observation_refs: refs,
        },
        PatternProcessorProvenance::new(PatternProcessorKind::Deterministic, None).unwrap(),
        timestamp(),
        timestamp(),
        LiteralValue::Text("Small sample; may not generalize across contexts".into()),
        SensitivityClass::ReflectivePrivate,
        None,
        relates_to_method,
        timestamp(),
    )
    .unwrap()
}

/// Reliability evidence for the exact Method under test.
fn reliability_pattern_for(id: &str, subject_ref: TypedRef) -> PersonalPattern {
    reliability_pattern_for_method(id, subject_ref, Some(method_ref()))
}

fn reliability_pattern(id: &str) -> PersonalPattern {
    reliability_pattern_for(id, pursuit_ref())
}

/// The kernel supplies no built-in reliability threshold; a pack-owned policy decides. This one
/// reduces support once any personal reliability pattern is available, and cites every pattern it
/// was offered as its exact basis.
struct ReduceWhenAnyPatternExists;

impl ScaffoldReliabilityPolicy for ReduceWhenAnyPatternExists {
    fn evaluate(&self, reliability_evidence: &[PersonalPattern]) -> ScaffoldFadeAssessment {
        if reliability_evidence.is_empty() {
            ScaffoldFadeAssessment {
                step: ScaffoldFadeStep::Unchanged,
                basis_pattern_refs: Vec::new(),
            }
        } else {
            ScaffoldFadeAssessment {
                step: ScaffoldFadeStep::Reduced,
                basis_pattern_refs: reliability_evidence
                    .iter()
                    .map(|pattern| pattern.id().clone())
                    .collect(),
            }
        }
    }
}

/// A misbehaving pack policy that claims a fade step without citing any evidence at all.
struct ReduceWithNoBasis;

impl ScaffoldReliabilityPolicy for ReduceWithNoBasis {
    fn evaluate(&self, _: &[PersonalPattern]) -> ScaffoldFadeAssessment {
        ScaffoldFadeAssessment {
            step: ScaffoldFadeStep::Removed,
            basis_pattern_refs: Vec::new(),
        }
    }
}

/// A pack policy that cites one exact pattern id, whatever it was offered. Used to drive the
/// wrong-subject, wrong-Method and never-offered rejections.
struct ReduceCitingExactly {
    pattern_id: PersonalPatternId,
}

impl ScaffoldReliabilityPolicy for ReduceCitingExactly {
    fn evaluate(&self, _: &[PersonalPattern]) -> ScaffoldFadeAssessment {
        ScaffoldFadeAssessment {
            step: ScaffoldFadeStep::Reduced,
            basis_pattern_refs: vec![self.pattern_id.clone()],
        }
    }
}

/// A misbehaving pack policy that pads its basis by naming the same pattern twice, which would
/// otherwise make one piece of evidence look like two.
struct ReduceCitingTheSamePatternTwice {
    pattern_id: PersonalPatternId,
}

impl ScaffoldReliabilityPolicy for ReduceCitingTheSamePatternTwice {
    fn evaluate(&self, _: &[PersonalPattern]) -> ScaffoldFadeAssessment {
        ScaffoldFadeAssessment {
            step: ScaffoldFadeStep::Reduced,
            basis_pattern_refs: vec![self.pattern_id.clone(), self.pattern_id.clone()],
        }
    }
}

/// A pack policy that only relies on a subset of the offered evidence; the coordinator must cite
/// exactly that subset, never every pattern it happened to be handed.
struct ReduceOnFirstPatternOnly;

impl ScaffoldReliabilityPolicy for ReduceOnFirstPatternOnly {
    fn evaluate(&self, reliability_evidence: &[PersonalPattern]) -> ScaffoldFadeAssessment {
        match reliability_evidence.first() {
            None => ScaffoldFadeAssessment {
                step: ScaffoldFadeStep::Unchanged,
                basis_pattern_refs: Vec::new(),
            },
            Some(pattern) => ScaffoldFadeAssessment {
                step: ScaffoldFadeStep::Reduced,
                basis_pattern_refs: vec![pattern.id().clone()],
            },
        }
    }
}

/// Runs one advisory consideration with the given reliability evidence and fade policy. Every fade
/// test differs only in those two inputs, so the surrounding boundaries stay fixed here.
fn consider_with_fade(
    reliability_evidence: &[PersonalPattern],
    fade_policy: &dyn ScaffoldReliabilityPolicy,
) -> Result<MethodConsideration, MethodRuntimeError> {
    let method = method();
    let protocol = protocol(EnforcementClass::Advisory);
    let assembler = FixedAssembler { calls: Cell::new(0) };
    let runtime = FixedProtocolRuntime { assessment: eligible_assessment() };
    let composer = CapturingComposer;
    let arbiter = FixedArbiter { decision: AttentionDecision::Surface, calls: Cell::new(0) };
    let coordinator = MethodRuntimeCoordinator::new(&assembler, &runtime, &composer, &arbiter);
    let policy = MethodRuntimePolicy {
        user_initiation: UserInitiationRequirement::Optional,
        contraindications: vec![],
    };
    coordinator.consider(MethodRuntimeRequest {
        subject_ref: pursuit_ref(),
        method: &method,
        protocol: &protocol,
        initiation: MethodInitiation::UserRequest,
        policy: &policy,
        now: timestamp(),
        source_proposal: None,
        reliability_evidence,
        fade_policy: Some(fade_policy),
    })
}

fn surfaced_fade_candidate(
    reliability_evidence: &[PersonalPattern],
    fade_policy: &dyn ScaffoldReliabilityPolicy,
) -> ScaffoldFadeCandidate {
    match consider_with_fade(reliability_evidence, fade_policy).unwrap() {
        MethodConsideration::Surface(recommendation) => {
            recommendation.composition.adaptation.fade_candidate
        }
        other => panic!("expected surfaced recommendation, got {other:?}"),
    }
}

/// Reliable personal capability evidence, run through a pack-owned (never kernel-hardcoded) policy,
/// can propose reducing scaffolding. The candidate cites its Personal Pattern basis and carries
/// forward the Protocol's own reversal semantics, so it stays reversible and never becomes a second
/// capability/mastery truth.
#[test]
fn scaffold_fade_candidate_reduces_support_when_pack_policy_finds_reliable_evidence() {
    let pattern = reliability_pattern(ID_PATTERN);
    let candidate =
        surfaced_fade_candidate(std::slice::from_ref(&pattern), &ReduceWhenAnyPatternExists);

    assert_eq!(candidate.step, ScaffoldFadeStep::Reduced);
    assert_eq!(candidate.basis_pattern_refs, vec![pattern.id().clone()]);
    assert_eq!(
        candidate.reversal_semantics,
        protocol(EnforcementClass::Advisory)
            .calibration_policy()
            .reversal_semantics
    );
}

/// Without reliable evidence, the same pack policy leaves support unchanged: fading is never
/// automatic or universal, and "unchanged" is a complete result rather than a failure.
#[test]
fn scaffold_fade_candidate_stays_unchanged_without_reliable_evidence() {
    let candidate = surfaced_fade_candidate(&[], &ReduceWhenAnyPatternExists);

    assert_eq!(candidate.step, ScaffoldFadeStep::Unchanged);
    assert!(candidate.basis_pattern_refs.is_empty());
}

/// A request with no fade policy at all never proposes reducing support, and still exposes the
/// Protocol's reversal semantics so the disclosure shape does not depend on whether a pack opted in.
#[test]
fn scaffold_fade_candidate_is_unchanged_when_no_pack_policy_is_supplied() {
    let method = method();
    let protocol = protocol(EnforcementClass::Advisory);
    let assembler = FixedAssembler { calls: Cell::new(0) };
    let runtime = FixedProtocolRuntime { assessment: eligible_assessment() };
    let composer = CapturingComposer;
    let arbiter = FixedArbiter { decision: AttentionDecision::Surface, calls: Cell::new(0) };
    let coordinator = MethodRuntimeCoordinator::new(&assembler, &runtime, &composer, &arbiter);
    let policy = MethodRuntimePolicy {
        user_initiation: UserInitiationRequirement::Optional,
        contraindications: vec![],
    };
    let pattern = reliability_pattern(ID_PATTERN);

    let consideration = coordinator
        .consider(MethodRuntimeRequest {
            subject_ref: pursuit_ref(),
            method: &method,
            protocol: &protocol,
            initiation: MethodInitiation::UserRequest,
            policy: &policy,
            now: timestamp(),
            source_proposal: None,
            reliability_evidence: std::slice::from_ref(&pattern),
            fade_policy: None,
        })
        .unwrap();

    let candidate = match consideration {
        MethodConsideration::Surface(recommendation) => {
            recommendation.composition.adaptation.fade_candidate
        }
        other => panic!("expected surfaced recommendation, got {other:?}"),
    };
    assert_eq!(candidate.step, ScaffoldFadeStep::Unchanged);
    assert!(candidate.basis_pattern_refs.is_empty());
    assert_eq!(
        candidate.reversal_semantics,
        protocol.calibration_policy().reversal_semantics
    );
}

/// A pack policy cannot reduce or remove support without saying what that rests on.
#[test]
fn scaffold_fade_forced_with_empty_basis_is_rejected() {
    let error = consider_with_fade(&[], &ReduceWithNoBasis).unwrap_err();

    assert_eq!(
        error,
        MethodRuntimeError::ScaffoldFadeEvidence(ScaffoldFadeViolation::MissingBasis)
    );
}

/// Citing the same pattern twice would make one piece of evidence look like two.
#[test]
fn scaffold_fade_citing_the_same_pattern_twice_is_rejected() {
    let pattern = reliability_pattern(ID_PATTERN);
    let fade_policy = ReduceCitingTheSamePatternTwice {
        pattern_id: pattern.id().clone(),
    };

    let error = consider_with_fade(std::slice::from_ref(&pattern), &fade_policy).unwrap_err();

    assert_eq!(
        error,
        MethodRuntimeError::ScaffoldFadeEvidence(ScaffoldFadeViolation::DuplicateBasisPattern)
    );
}

/// A basis must name evidence the coordinator actually offered the policy; a policy cannot invent
/// a Personal Pattern id it was never handed.
#[test]
fn scaffold_fade_citing_a_pattern_that_was_never_offered_is_rejected() {
    let offered = reliability_pattern(ID_PATTERN);
    let fade_policy = ReduceCitingExactly {
        pattern_id: PersonalPatternId::parse(ID_UNOFFERED_PATTERN).unwrap(),
    };

    let error = consider_with_fade(std::slice::from_ref(&offered), &fade_policy).unwrap_err();

    assert_eq!(
        error,
        MethodRuntimeError::ScaffoldFadeEvidence(ScaffoldFadeViolation::UnknownBasisPattern)
    );
}

/// A pack policy cannot cite a Personal Pattern belonging to a different subject as the basis for
/// fading this Method/Protocol's scaffolding.
#[test]
fn scaffold_fade_citing_a_wrong_subject_pattern_is_rejected() {
    let wrong_subject_pattern = reliability_pattern_for(ID_OTHER_PATTERN, other_pursuit_ref());
    let fade_policy = ReduceCitingExactly {
        pattern_id: wrong_subject_pattern.id().clone(),
    };

    let error =
        consider_with_fade(std::slice::from_ref(&wrong_subject_pattern), &fade_policy).unwrap_err();

    assert_eq!(
        error,
        MethodRuntimeError::ScaffoldFadeEvidence(ScaffoldFadeViolation::BasisSubjectMismatch)
    );
}

/// Exact subject equality is not enough. A pack policy cannot cite a Personal Pattern that shares
/// the exact considered subject but is reliability evidence for a different, unrelated Method:
/// same-subject alone must never be treated as evidence that support for *this* Method is safe to
/// reduce.
#[test]
fn scaffold_fade_citing_a_same_subject_but_unrelated_method_pattern_is_rejected() {
    let unrelated_method_ref = VersionedMethodRef {
        method_id: MethodId::new("method.unrelated_capability").unwrap(),
        method_version: SemVer::new("1.0.0").unwrap(),
    };
    let unrelated_method_pattern =
        reliability_pattern_for_method(ID_OTHER_PATTERN, pursuit_ref(), Some(unrelated_method_ref));
    let fade_policy = ReduceCitingExactly {
        pattern_id: unrelated_method_pattern.id().clone(),
    };

    let error = consider_with_fade(
        std::slice::from_ref(&unrelated_method_pattern),
        &fade_policy,
    )
    .unwrap_err();

    assert_eq!(
        error,
        MethodRuntimeError::ScaffoldFadeEvidence(ScaffoldFadeViolation::BasisMethodMismatch)
    );
}

/// A pattern that is not Method-scoped at all cannot stand in for evidence about this Method.
#[test]
fn scaffold_fade_citing_a_method_agnostic_pattern_is_rejected() {
    let method_agnostic_pattern = reliability_pattern_for_method(ID_PATTERN, pursuit_ref(), None);
    let fade_policy = ReduceCitingExactly {
        pattern_id: method_agnostic_pattern.id().clone(),
    };

    let error = consider_with_fade(std::slice::from_ref(&method_agnostic_pattern), &fade_policy)
        .unwrap_err();

    assert_eq!(
        error,
        MethodRuntimeError::ScaffoldFadeEvidence(ScaffoldFadeViolation::BasisMethodMismatch)
    );
}

/// When a pack policy relies on only a subset of the offered reliability evidence, the composed
/// candidate must cite exactly that subset — never every pattern the coordinator happened to hand
/// it, including ones the policy never looked at.
#[test]
fn scaffold_fade_candidate_cites_only_the_subset_the_policy_actually_relied_on() {
    let authoritative_pattern = reliability_pattern(ID_PATTERN);
    let unrelated_pattern = reliability_pattern(ID_OTHER_PATTERN);
    let reliability_evidence = [authoritative_pattern.clone(), unrelated_pattern];

    let candidate = surfaced_fade_candidate(&reliability_evidence, &ReduceOnFirstPatternOnly);

    assert_eq!(candidate.step, ScaffoldFadeStep::Reduced);
    assert_eq!(
        candidate.basis_pattern_refs,
        vec![authoritative_pattern.id().clone()]
    );
}
