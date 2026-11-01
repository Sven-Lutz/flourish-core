use flourish_core::method_outcome::{
    MethodOutcomeEvaluation, MethodOutcomeEvidence, MethodOutcomeUsefulness, PersonalLearningEvidence,
};
use flourish_core::*;

const ID_PURSUIT: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
const ID_SESSION: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAW";
const ID_DECISION: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAX";
const ID_OBS_A: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAY";
const ID_OBS_B: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAZ";
const ID_PATTERN: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB0";
const ID_CALIBRATION: &str = "01ARZ3NDEKTSV4RRFFQ69G5FB1";

fn time(value: &str) -> Timestamp {
    Timestamp::new(value).unwrap()
}

fn protocol_ref() -> VersionedProtocolRef {
    VersionedProtocolRef {
        protocol_id: ProtocolId::new("reflection.after_action_review").unwrap(),
        protocol_version: SemVer::new("1.0.0").unwrap(),
    }
}

fn subject_ref() -> TypedRef {
    TypedRef::Pursuit(PursuitId::parse(ID_PURSUIT).unwrap())
}

fn observation(id: &str, value: &str) -> Observation {
    Observation::new(
        ObservationId::parse(id).unwrap(),
        subject_ref(),
        ObservationType::new("reflection.review_outcome").unwrap(),
        LiteralValue::Text(value.into()),
        time("2026-09-07T18:30:00.000Z"),
        ObservationProvenance::Manual,
        None,
        None,
        SensitivityClass::ReflectivePrivate,
        None,
    )
    .unwrap()
}

#[test]
fn validated_personal_learning_projects_into_bounded_future_context() {
    let session = Session {
        id: SessionId::parse(ID_SESSION).unwrap(),
        session_type: SessionType::Review,
        pursuit_ref: Some(PursuitId::parse(ID_PURSUIT).unwrap()),
        practice_item_refs: vec![],
        started_at: time("2026-09-07T18:00:00.000Z"),
        ended_at: Some(time("2026-09-07T18:20:00.000Z")),
        interrupted: false,
        commitment_device_ref: None,
        context_snapshot_ref: None,
    };
    let execution = MethodExecution::new(
        session.id.clone(),
        VersionedMethodRef {
            method_id: MethodId::new("method.after_action_review").unwrap(),
            method_version: SemVer::new("1.0.0").unwrap(),
        },
        Some(protocol_ref()),
        MethodExecutionInitiation::AcceptedRecommendation,
        None,
        Some(DecisionRecordId::parse(ID_DECISION).unwrap()),
        time("2026-09-07T18:00:00.000Z"),
    )
    .unwrap();
    let observations = vec![
        observation(ID_OBS_A, "A shorter prompt was useful"),
        observation(ID_OBS_B, "The review was completed again"),
    ];
    let evaluation = MethodOutcomeEvaluation::new(
        Some(DecisionRecordId::parse(ID_DECISION).unwrap()),
        MethodOutcomeUsefulness::Useful,
        observations.iter().map(|value| value.id().clone()).collect(),
        vec!["Two bounded personal observations".into()],
        vec![],
        "Small personal sample; this may not generalize",
        time("2026-09-07T18:40:00.000Z"),
    )
    .unwrap();
    let outcome = MethodOutcomeEvidence {
        execution: &execution,
        session: &session,
        attempts: &[],
        observations: &observations,
        reflection: None,
        evaluation: &evaluation,
        pack_scope_proofs: &[],
    };

    let observation_refs: Vec<ObservationId> =
        observations.iter().map(|value| value.id().clone()).collect();
    let pattern = PersonalPattern::new(
        PersonalPatternId::parse(ID_PATTERN).unwrap(),
        subject_ref(),
        PackNamespacedPatternKind::new("reflection.short_prompt_may_help").unwrap(),
        "A shorter reflection prompt may be easier to complete in this context",
        EpistemicClass::Hypothesis,
        observation_refs.clone(),
        DerivationRef {
            derived_by: OwningRef::CapabilityPack(PackId::new("reflection").unwrap()),
            derivation_version: "reflection.pattern.v1".into(),
            method: "Compare evaluated personal observations".into(),
            source_observation_refs: observation_refs.clone(),
        },
        PatternProcessorProvenance::new(PatternProcessorKind::Deterministic, None).unwrap(),
        time("2026-09-07T18:00:00.000Z"),
        time("2026-09-07T18:30:00.000Z"),
        LiteralValue::Text("Only two observations; causal direction is unknown".into()),
        SensitivityClass::ReflectivePrivate,
        None,
        None,
        time("2026-09-07T18:45:00.000Z"),
    )
    .unwrap();
    let calibration = CalibrationRecord {
        id: CalibrationRecordId::parse(ID_CALIBRATION).unwrap(),
        protocol_ref: (protocol_ref().protocol_id, protocol_ref().protocol_version),
        parameter_key: "prompt_length".into(),
        proposed_value: LiteralValue::Text("short".into()),
        applied_value: None,
        status: CalibrationStatus::Proposed,
        evidentiary_observation_refs: vec![ObservationId::parse(ID_OBS_A).unwrap()],
        applied_at: None,
        manifest_override_ref: None,
    };
    let outcomes = [outcome];
    let patterns = [pattern];
    let calibrations = [calibration];
    let learning = PersonalLearningEvidence {
        outcomes: &outcomes,
        personal_patterns: &patterns,
        calibrations: &calibrations,
    };

    let context = PersonalOutcomeContext::from_validated_learning(
        &learning,
        subject_ref(),
        protocol_ref(),
    )
    .unwrap();

    assert_eq!(context.subject_ref(), &subject_ref());
    assert_eq!(context.protocol_ref(), &protocol_ref());
    assert_eq!(context.items().len(), 2);
    assert!(matches!(
        &context.items()[0],
        PersonalOutcomeContextItem::Pattern {
            epistemic_class: EpistemicClass::Hypothesis,
            supporting_observation_refs,
            uncertainty: LiteralValue::Text(_),
            ..
        } if supporting_observation_refs == &observation_refs
    ));
    let expected_calibration_observations = vec![ObservationId::parse(ID_OBS_A).unwrap()];
    assert!(matches!(
        &context.items()[1],
        PersonalOutcomeContextItem::Calibration {
            status: CalibrationStatus::Proposed,
            supporting_observation_refs,
            ..
        } if supporting_observation_refs == &expected_calibration_observations
    ));
}
