use flourish_core::method_outcome::{
    validate_method_outcome, validate_personal_learning, MethodOutcomeEvaluation,
    MethodOutcomeEvidence, MethodOutcomeUsefulness, MethodOutcomeViolation,
    PackOwnedScopeProof, PackOwnedScopeProofViolation, PersonalLearningEvidence,
};
use flourish_core::*;
use std::collections::BTreeMap;

const ID_PURSUIT: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
const ID_SESSION: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAW";
const ID_DECISION: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAX";
const ID_ATTEMPT: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAY";
const ID_PRACTICE: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAZ";
const ID_OBS_A: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB0";
const ID_OBS_B: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB1";
const ID_OBS_C: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB2";
const ID_REFLECTION: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB3";
const ID_PATTERN: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB4";
const ID_CALIBRATION: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB5";
const ID_OTHER_PURSUIT: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB6";
const ID_OTHER_SESSION: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB7";
const ID_OTHER_DECISION: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB8";
const ID_OBS_D: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB9";

fn timestamp() -> Timestamp {
    Timestamp::new("2026-09-07T18:00:00.000Z").unwrap()
}

fn later_timestamp() -> Timestamp {
    Timestamp::new("2026-09-07T18:30:00.000Z").unwrap()
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

fn other_protocol_ref() -> VersionedProtocolRef {
    VersionedProtocolRef {
        protocol_id: ProtocolId::new("reflection.other_review").unwrap(),
        protocol_version: SemVer::new("1.0.0").unwrap(),
    }
}

fn pursuit_ref() -> TypedRef {
    TypedRef::Pursuit(PursuitId::parse(ID_PURSUIT).unwrap())
}

fn other_pursuit_ref() -> TypedRef {
    TypedRef::Pursuit(PursuitId::parse(ID_OTHER_PURSUIT).unwrap())
}

fn session() -> Session {
    Session {
        id: SessionId::parse(ID_SESSION).unwrap(),
        session_type: SessionType::Review,
        pursuit_ref: Some(PursuitId::parse(ID_PURSUIT).unwrap()),
        practice_item_refs: vec![PracticeItemId::parse(ID_PRACTICE).unwrap()],
        started_at: timestamp(),
        ended_at: Some(later_timestamp()),
        interrupted: false,
        commitment_device_ref: None,
        context_snapshot_ref: None,
    }
}

fn other_session() -> Session {
    Session {
        id: SessionId::parse(ID_OTHER_SESSION).unwrap(),
        session_type: SessionType::Review,
        pursuit_ref: Some(PursuitId::parse(ID_OTHER_PURSUIT).unwrap()),
        practice_item_refs: vec![],
        started_at: timestamp(),
        ended_at: Some(later_timestamp()),
        interrupted: false,
        commitment_device_ref: None,
        context_snapshot_ref: None,
    }
}

fn attempt() -> Attempt {
    Attempt {
        id: AttemptId::parse(ID_ATTEMPT).unwrap(),
        practice_item_ref: PracticeItemId::parse(ID_PRACTICE).unwrap(),
        session_ref: Some(SessionId::parse(ID_SESSION).unwrap()),
        attempted_at: timestamp(),
        input_ref: None,
        outcome: AttemptOutcome::Qualitative,
        feedback_source: FeedbackSource::SelfAssessment,
        quality_dims: None,
    }
}

fn observation(id: &str, subject_ref: TypedRef, value: &str) -> Observation {
    Observation::new(
        ObservationId::parse(id).unwrap(),
        subject_ref,
        ObservationType::new("reflection.review_outcome").unwrap(),
        LiteralValue::Text(value.into()),
        later_timestamp(),
        ObservationProvenance::Manual,
        None,
        None,
        SensitivityClass::ReflectivePrivate,
        None,
    )
    .unwrap()
}

fn accepted_execution() -> MethodExecution {
    MethodExecution::new(
        SessionId::parse(ID_SESSION).unwrap(),
        method_ref(),
        Some(protocol_ref()),
        MethodExecutionInitiation::AcceptedRecommendation,
        None,
        Some(DecisionRecordId::parse(ID_DECISION).unwrap()),
        timestamp(),
    )
    .unwrap()
}

fn other_accepted_execution() -> MethodExecution {
    MethodExecution::new(
        SessionId::parse(ID_OTHER_SESSION).unwrap(),
        method_ref(),
        Some(other_protocol_ref()),
        MethodExecutionInitiation::AcceptedRecommendation,
        None,
        Some(DecisionRecordId::parse(ID_OTHER_DECISION).unwrap()),
        timestamp(),
    )
    .unwrap()
}

fn user_execution() -> MethodExecution {
    MethodExecution::new(
        SessionId::parse(ID_SESSION).unwrap(),
        method_ref(),
        Some(protocol_ref()),
        MethodExecutionInitiation::User,
        None,
        None,
        timestamp(),
    )
    .unwrap()
}

fn reflection() -> ReflectionOutcome {
    ReflectionOutcome {
        id: ReflectionOutcomeId::parse(ID_REFLECTION).unwrap(),
        session_ref: Some(SessionId::parse(ID_SESSION).unwrap()),
        journal_entry_ref: None,
        medium: ReflectionMedium::None,
        completed: true,
        structured_fields: BTreeMap::from([(
            "next_adjustment".into(),
            LiteralValue::Text("Use a smaller first step".into()),
        )]),
        pack_private_content_ref: None,
        recorded_at: later_timestamp(),
    }
}

fn evaluation(ids: &[&str]) -> MethodOutcomeEvaluation {
    evaluation_for(ID_DECISION, ids)
}

fn evaluation_for(decision_id: &str, ids: &[&str]) -> MethodOutcomeEvaluation {
    MethodOutcomeEvaluation::new(
        Some(DecisionRecordId::parse(decision_id).unwrap()),
        MethodOutcomeUsefulness::Useful,
        ids.iter()
            .map(|id| ObservationId::parse(*id).unwrap())
            .collect(),
        vec!["Observed after the bounded review session".into()],
        vec![],
        "Only one execution has been evaluated; generalization is uncertain",
        later_timestamp(),
    )
    .unwrap()
}

fn personal_pattern(observation_ids: &[&str]) -> PersonalPattern {
    personal_pattern_for(pursuit_ref(), observation_ids)
}

fn personal_pattern_for(subject_ref: TypedRef, observation_ids: &[&str]) -> PersonalPattern {
    let refs: Vec<ObservationId> = observation_ids
        .iter()
        .map(|id| ObservationId::parse(*id).unwrap())
        .collect();
    PersonalPattern::new(
        PersonalPatternId::parse(ID_PATTERN).unwrap(),
        subject_ref,
        PackNamespacedPatternKind::new("reflection.smaller_start_helped").unwrap(),
        "In the observed reviews, a smaller first step was followed by a useful adjustment",
        EpistemicClass::Hypothesis,
        refs.clone(),
        DerivationRef {
            derived_by: OwningRef::CapabilityPack(PackId::new("reflection").unwrap()),
            derivation_version: "reflection.pattern.v1".into(),
            method: "Compare evaluated personal review observations".into(),
            source_observation_refs: refs,
        },
        PatternProcessorProvenance::new(PatternProcessorKind::Deterministic, None).unwrap(),
        timestamp(),
        later_timestamp(),
        LiteralValue::Text("Small personal sample; causal direction is unknown".into()),
        SensitivityClass::ReflectivePrivate,
        None,
        None,
        later_timestamp(),
    )
    .unwrap()
}

fn calibration(observation_ids: &[&str]) -> CalibrationRecord {
    CalibrationRecord {
        id: CalibrationRecordId::parse(ID_CALIBRATION).unwrap(),
        protocol_ref: (protocol_ref().protocol_id, protocol_ref().protocol_version),
        parameter_key: "reflection_prompt_length".into(),
        proposed_value: LiteralValue::Text("short".into()),
        applied_value: None,
        status: CalibrationStatus::Proposed,
        evidentiary_observation_refs: observation_ids
            .iter()
            .map(|id| ObservationId::parse(*id).unwrap())
            .collect(),
        applied_at: None,
        manifest_override_ref: None,
    }
}

#[test]
fn accepted_recommendation_requires_observed_outcome_not_acceptance_alone() {
    let error = MethodOutcomeEvaluation::new(
        Some(DecisionRecordId::parse(ID_DECISION).unwrap()),
        MethodOutcomeUsefulness::Useful,
        vec![],
        vec![],
        vec![],
        "Acceptance is not evidence of usefulness",
        later_timestamp(),
    )
    .unwrap_err();

    assert_eq!(error, MethodOutcomeViolation::EvaluationRequiresObservation);
}

#[test]
fn session_attempt_observation_and_optional_reflection_form_a_valid_outcome_chain() {
    let execution = accepted_execution();
    let session = session();
    let attempt = attempt();
    let observations = vec![
        observation(ID_OBS_A, TypedRef::Attempt(attempt.id.clone()), "Useful adjustment found"),
        observation(ID_OBS_B, pursuit_ref(), "Next attempt felt easier to start"),
    ];
    let reflection = reflection();
    let evaluation = evaluation(&[ID_OBS_A, ID_OBS_B]);

    validate_method_outcome(&MethodOutcomeEvidence {
        execution: &execution,
        session: &session,
        attempts: &[attempt],
        observations: &observations,
        reflection: Some(&reflection),
        evaluation: &evaluation,
        pack_scope_proofs: &[],
    })
    .unwrap();
}

#[test]
fn direct_user_execution_cannot_fabricate_a_recommendation_decision_in_outcome() {
    let execution = user_execution();
    let session = session();
    let observations = vec![observation(ID_OBS_A, pursuit_ref(), "Observed result")];
    let evaluation = evaluation(&[ID_OBS_A]);

    let error = validate_method_outcome(&MethodOutcomeEvidence {
        execution: &execution,
        session: &session,
        attempts: &[],
        observations: &observations,
        reflection: None,
        evaluation: &evaluation,
        pack_scope_proofs: &[],
    })
    .unwrap_err();

    assert_eq!(error, MethodOutcomeViolation::UnexpectedEvaluationDecision);
}

#[test]
fn personal_pattern_and_calibration_can_only_use_explicitly_evaluated_observations() {
    let execution = accepted_execution();
    let session = session();
    let attempt = attempt();
    let observations = vec![
        observation(ID_OBS_A, TypedRef::Attempt(attempt.id.clone()), "Useful adjustment found"),
        observation(ID_OBS_B, pursuit_ref(), "Next attempt felt easier to start"),
    ];
    let evaluation = evaluation(&[ID_OBS_A, ID_OBS_B]);
    let outcome = MethodOutcomeEvidence {
        execution: &execution,
        session: &session,
        attempts: &[attempt],
        observations: &observations,
        reflection: None,
        evaluation: &evaluation,
        pack_scope_proofs: &[],
    };
    let pattern = personal_pattern(&[ID_OBS_A, ID_OBS_B]);
    let calibration = calibration(&[ID_OBS_A]);

    validate_personal_learning(&PersonalLearningEvidence {
        outcomes: &[outcome],
        personal_patterns: &[pattern],
        calibrations: &[calibration],
    })
    .unwrap();
}

#[test]
fn pattern_cannot_promote_an_unevaluated_observation_into_personal_truth() {
    let execution = accepted_execution();
    let session = session();
    let attempt = attempt();
    let observations = vec![
        observation(ID_OBS_A, TypedRef::Attempt(attempt.id.clone()), "Useful adjustment found"),
        observation(ID_OBS_B, pursuit_ref(), "Evaluated personal signal"),
        observation(ID_OBS_C, pursuit_ref(), "Recorded but not outcome-evaluated"),
    ];
    let evaluation = evaluation(&[ID_OBS_A, ID_OBS_B]);
    let outcome = MethodOutcomeEvidence {
        execution: &execution,
        session: &session,
        attempts: &[attempt],
        observations: &observations,
        reflection: None,
        evaluation: &evaluation,
        pack_scope_proofs: &[],
    };
    let pattern = personal_pattern(&[ID_OBS_A, ID_OBS_C]);

    let error = validate_personal_learning(&PersonalLearningEvidence {
        outcomes: &[outcome],
        personal_patterns: &[pattern],
        calibrations: &[],
    })
    .unwrap_err();

    assert_eq!(
        error,
        MethodOutcomeViolation::PatternObservationNotEvaluated(
            ObservationId::parse(ID_OBS_C).unwrap()
        )
    );
}

#[test]
fn pattern_cannot_mix_evaluated_observations_from_an_unrelated_subject() {
    let execution_a = accepted_execution();
    let session_a = session();
    let observations_a = vec![
        observation(ID_OBS_A, pursuit_ref(), "Outcome for pursuit A"),
        observation(ID_OBS_B, pursuit_ref(), "Second outcome for pursuit A"),
    ];
    let evaluation_a = evaluation(&[ID_OBS_A, ID_OBS_B]);
    let outcome_a = MethodOutcomeEvidence {
        execution: &execution_a,
        session: &session_a,
        attempts: &[],
        observations: &observations_a,
        reflection: None,
        evaluation: &evaluation_a,
        pack_scope_proofs: &[],
    };

    let execution_b = other_accepted_execution();
    let session_b = other_session();
    let observations_b = vec![
        observation(ID_OBS_C, other_pursuit_ref(), "Outcome for pursuit B"),
        observation(ID_OBS_D, other_pursuit_ref(), "Second outcome for pursuit B"),
    ];
    let evaluation_b = evaluation_for(ID_OTHER_DECISION, &[ID_OBS_C, ID_OBS_D]);
    let outcome_b = MethodOutcomeEvidence {
        execution: &execution_b,
        session: &session_b,
        attempts: &[],
        observations: &observations_b,
        reflection: None,
        evaluation: &evaluation_b,
        pack_scope_proofs: &[],
    };

    let pattern = personal_pattern_for(other_pursuit_ref(), &[ID_OBS_A, ID_OBS_C]);
    let error = validate_personal_learning(&PersonalLearningEvidence {
        outcomes: &[outcome_a, outcome_b],
        personal_patterns: &[pattern],
        calibrations: &[],
    })
    .unwrap_err();

    assert_eq!(
        error,
        MethodOutcomeViolation::PatternObservationSubjectMismatch(
            ObservationId::parse(ID_OBS_A).unwrap()
        )
    );
}

#[test]
fn calibration_must_target_the_protocol_that_generated_the_validated_outcome() {
    let execution = accepted_execution();
    let session = session();
    let observations = vec![observation(ID_OBS_A, pursuit_ref(), "Observed result")];
    let evaluation = evaluation(&[ID_OBS_A]);
    let outcome = MethodOutcomeEvidence {
        execution: &execution,
        session: &session,
        attempts: &[],
        observations: &observations,
        reflection: None,
        evaluation: &evaluation,
        pack_scope_proofs: &[],
    };
    let mut calibration = calibration(&[ID_OBS_A]);
    calibration.protocol_ref = (
        ProtocolId::new("other.protocol").unwrap(),
        SemVer::new("1.0.0").unwrap(),
    );

    let error = validate_personal_learning(&PersonalLearningEvidence {
        outcomes: &[outcome],
        personal_patterns: &[],
        calibrations: &[calibration],
    })
    .unwrap_err();

    assert_eq!(error, MethodOutcomeViolation::CalibrationProtocolMismatch);
}

#[test]
fn calibration_cannot_borrow_evidence_from_another_protocol_outcome() {
    let execution_a = accepted_execution();
    let session_a = session();
    let observations_a = vec![observation(ID_OBS_A, pursuit_ref(), "Protocol A result")];
    let evaluation_a = evaluation(&[ID_OBS_A]);
    let outcome_a = MethodOutcomeEvidence {
        execution: &execution_a,
        session: &session_a,
        attempts: &[],
        observations: &observations_a,
        reflection: None,
        evaluation: &evaluation_a,
        pack_scope_proofs: &[],
    };

    let execution_b = other_accepted_execution();
    let session_b = other_session();
    let observations_b = vec![observation(ID_OBS_C, other_pursuit_ref(), "Protocol B result")];
    let evaluation_b = evaluation_for(ID_OTHER_DECISION, &[ID_OBS_C]);
    let outcome_b = MethodOutcomeEvidence {
        execution: &execution_b,
        session: &session_b,
        attempts: &[],
        observations: &observations_b,
        reflection: None,
        evaluation: &evaluation_b,
        pack_scope_proofs: &[],
    };

    let mut calibration = calibration(&[ID_OBS_A]);
    calibration.protocol_ref = (
        other_protocol_ref().protocol_id,
        other_protocol_ref().protocol_version,
    );

    let error = validate_personal_learning(&PersonalLearningEvidence {
        outcomes: &[outcome_a, outcome_b],
        personal_patterns: &[],
        calibrations: &[calibration],
    })
    .unwrap_err();

    assert_eq!(
        error,
        MethodOutcomeViolation::CalibrationObservationProtocolMismatch(
            ObservationId::parse(ID_OBS_A).unwrap()
        )
    );
}

// ---------------------------------------------------------------------------
// Pack-owned scope
//
// A `TypedRef::PackOwned` carries no pack or execution lineage, so the kernel cannot resolve its
// scope structurally the way it resolves a Session, Attempt, Pursuit or PracticeItem subject. It
// fails closed and admits such a subject only through an exact-match `PackOwnedScopeProof`: the
// pack module does its own validation and says so, per subject, attributably.
// ---------------------------------------------------------------------------

const ID_PACK_OWNED_A: &str = "01ARZ3NDEKTSV4RRFFQ69G5FBB";
const ID_PACK_OWNED_B: &str = "01ARZ3NDEKTSV4RRFFQ69G5FBC";
const ID_PACK_OWNED_OBS: &str = "01ARZ3NDEKTSV4RRFFQ69G5FBA";

fn pack_owned_subject(id: &str) -> TypedRef {
    TypedRef::PackOwned {
        kind: PackEntityKind::new("learning.capability").unwrap(),
        id: Id::new(id).unwrap(),
    }
}

fn pack_proof(subject_id: &str, session_id: &str) -> PackOwnedScopeProof {
    PackOwnedScopeProof::new(
        pack_owned_subject(subject_id),
        SessionId::parse(session_id).unwrap(),
        "learning-pack-validator",
    )
    .unwrap()
}

fn pack_owned_outcome<'a>(
    session: &'a Session,
    execution: &'a MethodExecution,
    observations: &'a [Observation],
    evaluation: &'a MethodOutcomeEvaluation,
    pack_scope_proofs: &'a [PackOwnedScopeProof],
) -> MethodOutcomeEvidence<'a> {
    MethodOutcomeEvidence {
        execution,
        session,
        attempts: &[],
        observations,
        reflection: None,
        evaluation,
        pack_scope_proofs,
    }
}

/// Without a proof, a pack-owned Observation is out of scope. Pack ownership never implies
/// authorization on its own.
#[test]
fn an_unproven_pack_owned_observation_is_out_of_scope() {
    let session = session();
    let execution = accepted_execution();
    let observations = [
        observation(ID_OBS_A, pursuit_ref(), "reviewed"),
        observation(
            ID_PACK_OWNED_OBS,
            pack_owned_subject(ID_PACK_OWNED_A),
            "claimed",
        ),
    ];
    let evaluation = evaluation(&[ID_OBS_A]);

    let error = validate_method_outcome(&pack_owned_outcome(
        &session,
        &execution,
        &observations,
        &evaluation,
        &[],
    ))
    .unwrap_err();

    assert_eq!(
        error,
        MethodOutcomeViolation::ObservationOutOfScope(
            ObservationId::parse(ID_PACK_OWNED_OBS).unwrap()
        )
    );
}

/// An exact-match proof for this subject and this Session admits the Observation.
#[test]
fn a_proven_pack_owned_observation_is_in_scope() {
    let session = session();
    let execution = accepted_execution();
    let observations = [observation(
        ID_PACK_OWNED_OBS,
        pack_owned_subject(ID_PACK_OWNED_A),
        "claimed",
    )];
    let evaluation = evaluation(&[ID_PACK_OWNED_OBS]);
    let proofs = [pack_proof(ID_PACK_OWNED_A, ID_SESSION)];

    validate_method_outcome(&pack_owned_outcome(
        &session,
        &execution,
        &observations,
        &evaluation,
        &proofs,
    ))
    .unwrap();
}

/// A proof for a sibling pack entity authorizes nothing: the subject must match exactly, so one
/// legitimate capability claim cannot carry an unrelated one in beside it.
#[test]
fn a_proof_for_another_pack_owned_subject_does_not_authorize_this_one() {
    let session = session();
    let execution = accepted_execution();
    let observations = [observation(
        ID_PACK_OWNED_OBS,
        pack_owned_subject(ID_PACK_OWNED_A),
        "claimed",
    )];
    let evaluation = evaluation(&[ID_PACK_OWNED_OBS]);
    let proofs = [pack_proof(ID_PACK_OWNED_B, ID_SESSION)];

    let error = validate_method_outcome(&pack_owned_outcome(
        &session,
        &execution,
        &observations,
        &evaluation,
        &proofs,
    ))
    .unwrap_err();

    assert_eq!(
        error,
        MethodOutcomeViolation::ObservationOutOfScope(
            ObservationId::parse(ID_PACK_OWNED_OBS).unwrap()
        )
    );
}

/// A proof issued for a different Session does not travel: scope is per execution, so a pack
/// cannot validate a subject once and reuse that attestation everywhere.
#[test]
fn a_proof_from_another_session_does_not_authorize_this_execution() {
    let session = session();
    let execution = accepted_execution();
    let observations = [observation(
        ID_PACK_OWNED_OBS,
        pack_owned_subject(ID_PACK_OWNED_A),
        "claimed",
    )];
    let evaluation = evaluation(&[ID_PACK_OWNED_OBS]);
    let proofs = [pack_proof(ID_PACK_OWNED_A, ID_OTHER_SESSION)];

    let error = validate_method_outcome(&pack_owned_outcome(
        &session,
        &execution,
        &observations,
        &evaluation,
        &proofs,
    ))
    .unwrap_err();

    assert_eq!(
        error,
        MethodOutcomeViolation::ObservationOutOfScope(
            ObservationId::parse(ID_PACK_OWNED_OBS).unwrap()
        )
    );
}

/// A proof cannot be constructed for a subject that is not pack-owned, so it can never be used to
/// widen scope for a subject the kernel already resolves structurally.
#[test]
fn a_scope_proof_cannot_name_a_non_pack_owned_subject() {
    let error = PackOwnedScopeProof::new(
        pursuit_ref(),
        SessionId::parse(ID_SESSION).unwrap(),
        "learning-pack-validator",
    )
    .unwrap_err();

    assert_eq!(error, PackOwnedScopeProofViolation::NotPackOwned);
}

/// An anonymous proof is worthless for audit: the point is naming who checked.
#[test]
fn a_scope_proof_requires_a_validator_attribution() {
    let error = PackOwnedScopeProof::new(
        pack_owned_subject(ID_PACK_OWNED_A),
        SessionId::parse(ID_SESSION).unwrap(),
        "   ",
    )
    .unwrap_err();

    assert_eq!(
        error,
        PackOwnedScopeProofViolation::MissingValidatorAttribution
    );
}

/// A proof admits an Observation into scope; it does not by itself make personal truth. A
/// PersonalPattern about a pack-owned subject additionally requires that the outcome explicitly
/// evaluated an Observation on that subject, so the subject is not yet in *pattern* scope while
/// the pack's evidence sits unevaluated — the attestation and the evaluation are separate gates.
#[test]
fn a_pack_owned_subject_is_not_in_pattern_scope_while_its_evidence_is_unevaluated() {
    let session = session();
    let execution = accepted_execution();
    let observations = [
        observation(ID_OBS_A, pursuit_ref(), "reviewed"),
        observation(
            ID_PACK_OWNED_OBS,
            pack_owned_subject(ID_PACK_OWNED_A),
            "claimed",
        ),
    ];
    // The pack-owned Observation is proven in scope, but only the Pursuit one is evaluated.
    let evaluation = evaluation(&[ID_OBS_A]);
    let proofs = [pack_proof(ID_PACK_OWNED_A, ID_SESSION)];
    let outcome = pack_owned_outcome(&session, &execution, &observations, &evaluation, &proofs);
    validate_method_outcome(&outcome).unwrap();

    let pattern = personal_pattern_for(
        pack_owned_subject(ID_PACK_OWNED_A),
        &[ID_OBS_A, ID_PACK_OWNED_OBS],
    );
    let error = validate_personal_learning(&PersonalLearningEvidence {
        outcomes: std::slice::from_ref(&outcome),
        personal_patterns: &[pattern],
        calibrations: &[],
    })
    .unwrap_err();

    assert_eq!(error, MethodOutcomeViolation::PatternSubjectOutOfScope);
}

/// With the proof in place and the Observation actually evaluated, a pack-owned PersonalPattern is
/// valid — the path exists rather than being closed outright, which is what a capability pack needs.
#[test]
fn a_proven_and_evaluated_pack_owned_observation_can_carry_a_personal_pattern() {
    let session = session();
    let execution = accepted_execution();
    let observations = [
        observation(
            ID_PACK_OWNED_OBS,
            pack_owned_subject(ID_PACK_OWNED_A),
            "claimed",
        ),
        observation(ID_OBS_A, pursuit_ref(), "reviewed"),
    ];
    let evaluation = evaluation(&[ID_PACK_OWNED_OBS, ID_OBS_A]);
    let proofs = [pack_proof(ID_PACK_OWNED_A, ID_SESSION)];
    let outcome = pack_owned_outcome(&session, &execution, &observations, &evaluation, &proofs);
    validate_method_outcome(&outcome).unwrap();

    let pattern = personal_pattern_for(
        pack_owned_subject(ID_PACK_OWNED_A),
        &[ID_PACK_OWNED_OBS, ID_OBS_A],
    );
    validate_personal_learning(&PersonalLearningEvidence {
        outcomes: std::slice::from_ref(&outcome),
        personal_patterns: &[pattern],
        calibrations: &[],
    })
    .unwrap();
}
