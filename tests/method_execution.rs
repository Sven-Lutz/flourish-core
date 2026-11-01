use flourish_core::method_execution::{
    resolve_method_decision, start_user_requested_method, MethodDecisionResolution,
    MethodDecisionViolation,
};
use flourish_core::method_runtime::{
    BurdenDisclosure, MethodAdaptationDisclosure, MethodEvidenceDisclosure, MethodRecommendation,
    RecommendationAction, RecommendationComposition, ScaffoldFadeCandidate, ScaffoldFadeStep,
    UserInitiationRequirement,
};
use flourish_core::*;

const ID_PURSUIT: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
const ID_SESSION: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAW";
const ID_DECISION: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAX";
const ID_CONTEXT: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAY";
const ID_RECOMMENDATION: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAZ";
const ID_OTHER_RECOMMENDATION: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB0";

fn timestamp() -> Timestamp {
    Timestamp::new("2026-09-07T17:20:00.000Z").unwrap()
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
        None,
        vec![MethodEvidenceRef::EvidenceClaim(
            EvidenceClaimId::new("evidence.reflection.boundaries").unwrap(),
        )],
        MethodEvidenceStatus::MixedContextDependent,
        vec![],
        vec![],
        vec!["Optional, not universally useful".into()],
        vec!["A few minutes".into()],
        vec![],
        vec![],
        vec![],
        vec![protocol_ref()],
        vec![],
        vec![],
        None,
        LiteralValue::Text("catalog.method.v1".into()),
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

fn protocol() -> Protocol {
    let registries = EmptyRegistries;
    Protocol::new_with_method(
        protocol_ref().protocol_id,
        protocol_ref().protocol_version,
        PackId::new("reflection").unwrap(),
        EnforcementClass::Advisory,
        "A short optional review".into(),
        vec![],
        vec![],
        vec![],
        vec![OpaqueRef::new("design.after_action_review").unwrap()],
        RecommendationPolicy {
            conditions: vec![],
            allows_not_now: true,
        },
        vec![ProtocolAlternative {
            key: "skip".into(),
            label: "Skip".into(),
            value: LiteralValue::Null,
        }],
        vec![ObservationType::new("reflection.review_outcome").unwrap()],
        CalibrationPolicy {
            enabled: true,
            observation_types: vec![ObservationType::new("reflection.review_outcome").unwrap()],
            reversal_semantics: "Remove override".into(),
        },
        None,
        "Stop suggesting it".into(),
        Some(method_ref()),
        &registries,
        &registries,
    )
    .unwrap()
}

fn protocol_without_deferral() -> Protocol {
    let registries = EmptyRegistries;
    Protocol::new_with_method(
        protocol_ref().protocol_id,
        protocol_ref().protocol_version,
        PackId::new("reflection").unwrap(),
        EnforcementClass::Advisory,
        "A short optional review".into(),
        vec![],
        vec![],
        vec![],
        vec![OpaqueRef::new("design.after_action_review").unwrap()],
        RecommendationPolicy {
            conditions: vec![],
            allows_not_now: false,
        },
        vec![ProtocolAlternative {
            key: "skip".into(),
            label: "Skip".into(),
            value: LiteralValue::Null,
        }],
        vec![],
        CalibrationPolicy {
            enabled: false,
            observation_types: vec![],
            reversal_semantics: "No personal calibration".into(),
        },
        None,
        "Stop suggesting it".into(),
        Some(method_ref()),
        &registries,
        &registries,
    )
    .unwrap()
}

fn subject_ref() -> TypedRef {
    TypedRef::Pursuit(PursuitId::parse(ID_PURSUIT).unwrap())
}

fn recommendation() -> MethodRecommendation {
    MethodRecommendation {
        id: RecommendationId::parse(ID_RECOMMENDATION).unwrap(),
        recommendation: "Try a short review".into(),
        composition: RecommendationComposition {
            subject_ref: subject_ref(),
            method_ref: method_ref(),
            protocol_ref: protocol_ref(),
            reason: "A bounded review may help".into(),
            alternatives: vec![ProtocolAlternative {
                key: "skip".into(),
                label: "Skip".into(),
                value: LiteralValue::Null,
            }],
            actions: vec![
                RecommendationAction::Accept,
                RecommendationAction::Modify,
                RecommendationAction::Decline,
                RecommendationAction::Defer,
            ],
            burden: BurdenDisclosure::Declared(vec!["A few minutes".into()]),
            boundaries: vec!["Optional".into()],
            risks_and_limitations: vec![],
            evidence: MethodEvidenceDisclosure {
                status: MethodEvidenceStatus::MixedContextDependent,
                method_evidence_refs: vec![MethodEvidenceRef::EvidenceClaim(
                    EvidenceClaimId::new("evidence.reflection.boundaries").unwrap(),
                )],
                protocol_rationale_refs: vec![OpaqueRef::new("design.after_action_review").unwrap()],
            },
            adaptation: MethodAdaptationDisclosure {
                calibration_enabled: true,
                observation_types: vec![ObservationType::new("reflection.review_outcome").unwrap()],
                reversal_semantics: "Remove override".into(),
                disabled_semantics: "Stop suggesting it".into(),
                fade_candidate: ScaffoldFadeCandidate {
                    step: ScaffoldFadeStep::Unchanged,
                    basis_pattern_refs: vec![],
                    reversal_semantics: "Remove override".into(),
                },
            },
            user_initiation: UserInitiationRequirement::Optional,
            context_snapshot_ref: ContextSnapshotId::parse(ID_CONTEXT).unwrap(),
            personal_outcome_context: None,
            source_proposal_ref: None,
        },
    }
}

fn session() -> Session {
    Session {
        id: SessionId::parse(ID_SESSION).unwrap(),
        session_type: SessionType::Review,
        pursuit_ref: Some(PursuitId::parse(ID_PURSUIT).unwrap()),
        practice_item_refs: vec![],
        started_at: timestamp(),
        ended_at: None,
        interrupted: false,
        commitment_device_ref: None,
        context_snapshot_ref: Some(ContextSnapshotId::parse(ID_CONTEXT).unwrap()),
    }
}

fn decision(choice: ProtocolResponseChoice, recommendation_id: &str) -> DecisionRecord {
    DecisionRecord::new(
        DecisionRecordId::parse(ID_DECISION).unwrap(),
        DecisionOrigin::SystemRecommended,
        DecisionKind::ProtocolResponse,
        subject_ref(),
        Some(RecommendationRef::Recommendation(
            RecommendationId::parse(recommendation_id).unwrap(),
        )),
        vec![],
        None,
        DecisionOutcome::ProtocolResponse(choice),
        None,
        ContextSnapshotId::parse(ID_CONTEXT).unwrap(),
        timestamp(),
    )
    .unwrap()
}

#[test]
fn explicit_accept_starts_execution_on_existing_session() {
    let recommendation = recommendation();
    let decision = decision(ProtocolResponseChoice::Accept, ID_RECOMMENDATION);
    let session = session();

    let result = resolve_method_decision(&recommendation, &decision, &session, timestamp()).unwrap();
    let MethodDecisionResolution::Start(execution) = result else {
        panic!("expected Method execution");
    };

    assert_eq!(execution.session_ref, session.id);
    assert_eq!(execution.method_ref, method_ref());
    assert_eq!(execution.protocol_ref, Some(protocol_ref()));
    assert_eq!(execution.initiation, MethodExecutionInitiation::AcceptedRecommendation);
    assert_eq!(execution.source_decision_ref.as_ref(), Some(decision.id()));
    assert!(execution.source_proposal_ref.is_none());
}

#[test]
fn defer_is_complete_and_does_not_start_original_protocol() {
    let result = resolve_method_decision(
        &recommendation(),
        &decision(ProtocolResponseChoice::Defer, ID_RECOMMENDATION),
        &session(),
        timestamp(),
    )
    .unwrap();

    assert_eq!(
        result,
        MethodDecisionResolution::DoNotStart(ProtocolResponseChoice::Defer)
    );
}

#[test]
fn decision_for_another_recommendation_fails_closed() {
    let error = resolve_method_decision(
        &recommendation(),
        &decision(ProtocolResponseChoice::Accept, ID_OTHER_RECOMMENDATION),
        &session(),
        timestamp(),
    )
    .unwrap_err();

    assert_eq!(error, MethodDecisionViolation::RecommendationReferenceMismatch);
}

#[test]
fn context_mismatch_prevents_execution() {
    let mut wrong_session = session();
    wrong_session.context_snapshot_ref = None;
    let error = resolve_method_decision(
        &recommendation(),
        &decision(ProtocolResponseChoice::Accept, ID_RECOMMENDATION),
        &wrong_session,
        timestamp(),
    )
    .unwrap_err();

    assert_eq!(error, MethodDecisionViolation::ContextMismatch);
}

#[test]
fn direct_user_request_has_no_fabricated_recommendation_provenance() {
    let execution = start_user_requested_method(&method(), &protocol(), &session(), timestamp())
        .unwrap();

    assert_eq!(execution.initiation, MethodExecutionInitiation::User);
    assert!(execution.source_proposal_ref.is_none());
    assert!(execution.source_decision_ref.is_none());
    assert_eq!(execution.session_ref, SessionId::parse(ID_SESSION).unwrap());
}

#[test]
fn direct_user_request_cannot_bypass_behavioral_deferral() {
    let error = start_user_requested_method(
        &method(),
        &protocol_without_deferral(),
        &session(),
        timestamp(),
    )
    .unwrap_err();
    assert_eq!(error, MethodDecisionViolation::BehaviorMethodMustAllowDeferral);
}
