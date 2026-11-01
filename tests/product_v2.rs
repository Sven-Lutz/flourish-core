use flourish_core::*;
use std::collections::BTreeMap;

const ID_A: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
const ID_B: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAW";
const ID_C: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAX";

fn timestamp() -> Timestamp {
    Timestamp::new("2026-09-02T10:00:00.000Z").unwrap()
}

fn pursuit_id() -> PursuitId {
    PursuitId::parse(ID_A).unwrap()
}

fn method_ref() -> VersionedMethodRef {
    VersionedMethodRef {
        method_id: MethodId::new("method.retrieval_practice").unwrap(),
        method_version: SemVer::new("1.0.0").unwrap(),
    }
}

fn protocol_ref() -> VersionedProtocolRef {
    VersionedProtocolRef {
        protocol_id: ProtocolId::new("learning.independent_attempt").unwrap(),
        protocol_version: SemVer::new("1.0.0").unwrap(),
    }
}

fn integration_ref() -> VersionedIntegrationCapabilityRef {
    VersionedIntegrationCapabilityRef {
        capability_id: IntegrationCapabilityId::new("integration.anki.reviews").unwrap(),
        capability_version: SemVer::new("1.0.0").unwrap(),
    }
}

fn composition() -> PathComposition {
    PathComposition::new(
        1,
        DirectionSpec::new(
            "Build reliable independent problem-solving ability",
            DirectionStance::Provisional,
            Some(LifeAreaContext::new("learning", "Learning").unwrap()),
        )
        .unwrap(),
        vec![
            PackOwnedCapabilityRef::new(PackId::new("learning").unwrap(), "problem_solving")
                .unwrap(),
        ],
        vec![method_ref()],
        vec![protocol_ref()],
        vec![integration_ref()],
        vec![SessionId::parse(ID_B).unwrap()],
        OperatingState::BuildCapacity,
        vec![PathConstraint::new(
            "weekly_time",
            LiteralValue::Unsigned(180),
            PathConstraintProvenance::UserStated,
            None,
        )
        .unwrap()],
        BTreeMap::from([(
            "learning.assistance".into(),
            LiteralValue::Text("start_unaided".into()),
        )]),
        ExplanationPreference::Standard,
    )
    .unwrap()
}

fn manual_path_decision() -> DecisionRecord {
    DecisionRecord::new(
        DecisionRecordId::parse(ID_B).unwrap(),
        DecisionOrigin::UserInitiated,
        DecisionKind::PathCompositionChange,
        TypedRef::Pursuit(pursuit_id()),
        None,
        vec![],
        None,
        DecisionOutcome::PathCompositionChange(PathCompositionAction::Accept),
        None,
        ContextSnapshotId::parse(ID_C).unwrap(),
        timestamp(),
    )
    .unwrap()
}

fn model_provenance() -> ModelProvenance {
    ModelProvenance::new("local", "test-model", "1", "path-delta.v1").unwrap()
}

fn basis() -> EpistemicBasisRef {
    EpistemicBasisRef::new(
        EpistemicBasisTarget::UserRecord(TypedRef::Pursuit(pursuit_id())),
        EpistemicClass::Known,
        LiteralValue::Text("user statement".into()),
    )
    .unwrap()
}

fn proposal_alternative() -> ProtocolAlternative {
    ProtocolAlternative {
        key: "keep_current_path".into(),
        label: "Keep the current Path".into(),
        value: LiteralValue::Null,
    }
}

#[test]
fn manual_zero_ai_path_is_valid_and_deterministic() {
    let first = composition();
    let second = composition();
    assert_eq!(first.sha256(), second.sha256());

    let accepted = PursuitPathVersion::new(
        pursuit_id(),
        1,
        None,
        first,
        None,
        &manual_path_decision(),
        None,
        None,
        IdempotencyKey::new("accept-manual-path-1").unwrap(),
        timestamp(),
    )
    .unwrap();

    assert_eq!(accepted.path_version(), 1);
    assert_eq!(
        accepted.composition().operating_state(),
        OperatingState::BuildCapacity
    );
    assert_eq!(accepted.composition_sha256(), second.sha256());

    let changed = PursuitPathChanged::from_accepted_version(&accepted);
    assert_eq!(changed.old_path_version, None);
    assert_eq!(changed.new_path_version, 1);
    assert!(matches!(
        changed.to_literal_value(),
        LiteralValue::Object(_)
    ));

    let topic = pursuit_path_changed_topic_definition();
    assert_eq!(topic.topic().as_str(), PURSUIT_PATH_CHANGED_TOPIC);
    assert_eq!(topic.category(), FactCategory::Operational);
    assert_eq!(topic.operational_basis(), None);
    assert_eq!(
        topic.publisher(),
        &OwningRef::KernelService(ServiceId::ManifestResolver)
    );
}

#[test]
fn accepted_path_version_requires_the_immediate_pursuit_version_chain() {
    let invalid = PursuitPathVersion::new(
        pursuit_id(),
        2,
        None,
        composition(),
        None,
        &manual_path_decision(),
        None,
        None,
        IdempotencyKey::new("stale").unwrap(),
        timestamp(),
    );
    assert_eq!(invalid.unwrap_err(), PathViolation::InvalidPathVersionChain);

    let invalid_draft_revision = PursuitPathVersion::new(
        pursuit_id(),
        1,
        None,
        composition(),
        None,
        &manual_path_decision(),
        Some(PathDraftRevisionRef {
            draft_id: PathDraftId::parse(ID_C).unwrap(),
            revision: 0,
        }),
        None,
        IdempotencyKey::new("invalid-draft-revision").unwrap(),
        timestamp(),
    );
    assert_eq!(
        invalid_draft_revision.unwrap_err(),
        PathViolation::InvalidDraftRevision
    );
}

#[test]
fn template_provenance_can_only_enter_an_inactive_draft() {
    let receipt = PathTemplateImportReceipt::new(
        PathTemplateImportId::parse(ID_A).unwrap(),
        "file:///teacher/path-template.json",
        Some("Teacher".into()),
        Some("2026.1".into()),
        Bytes32::new([7; 32]),
        LiteralValue::Text("shared by user".into()),
        TemplateValidationStatus::Valid,
        timestamp(),
    )
    .unwrap();
    let draft = PathDraft::new(
        PathDraftId::parse(ID_B).unwrap(),
        Some(pursuit_id()),
        PathDraftOrigin::Template,
        Some(receipt.id),
        None,
        timestamp(),
        None,
    )
    .unwrap();
    let revision = PathDraftRevision::new(&draft, 1, Some(1), composition(), timestamp()).unwrap();

    assert_eq!(draft.origin, PathDraftOrigin::Template);
    assert_eq!(revision.draft_ref, draft.id);

    let invalid_direct_origin = PathDraft::new(
        PathDraftId::parse(ID_C).unwrap(),
        Some(pursuit_id()),
        PathDraftOrigin::Manual,
        Some(PathTemplateImportId::parse(ID_A).unwrap()),
        None,
        timestamp(),
        None,
    );
    assert_eq!(
        invalid_direct_origin.unwrap_err(),
        PathViolation::DraftOriginMismatch
    );
}

struct CompletePathRegistry;

impl PathReferenceRegistry for CompletePathRegistry {
    fn has_capability(&self, _: &PackOwnedCapabilityRef) -> bool {
        true
    }

    fn has_method(&self, _: &VersionedMethodRef) -> bool {
        true
    }

    fn has_protocol(&self, _: &VersionedProtocolRef) -> bool {
        true
    }

    fn has_integration(&self, _: &VersionedIntegrationCapabilityRef) -> bool {
        false
    }

    fn has_experiment(&self, _: &SessionId) -> bool {
        true
    }
}

#[test]
fn path_reference_validation_fails_closed() {
    assert!(matches!(
        composition().validate_references(&CompletePathRegistry),
        Err(PathReferenceViolation::UnknownIntegration(_))
    ));
}

#[test]
fn ai_durable_write_attempts_and_fact_promotions_reject_mechanically() {
    let delta = PathDelta::new(
        pursuit_id(),
        1,
        vec![PathChange::SetOperatingState(OperatingState::Master)],
    )
    .unwrap();
    let target = ProposalTarget {
        target_ref: TypedRef::Pursuit(pursuit_id()),
        base_target_version: Some(1),
    };
    let direct_write = AIProposal::new(
        AIProposalId::parse(ID_B).unwrap(),
        EpistemicClass::Inferred,
        1,
        AIProposalPayload::PathDelta(delta.clone()),
        target.clone(),
        vec![basis()],
        vec![],
        vec![],
        vec![proposal_alternative()],
        Confidence::Medium,
        false,
        model_provenance(),
        SensitivityClass::Standard,
        timestamp(),
        None,
    );
    assert_eq!(
        direct_write.unwrap_err(),
        AIProposalViolation::DirectDurableWriteAttempt
    );

    let fact_promotion = AIProposal::new(
        AIProposalId::parse(ID_B).unwrap(),
        EpistemicClass::Known,
        1,
        AIProposalPayload::PathDelta(delta),
        target,
        vec![basis()],
        vec![],
        vec![],
        vec![proposal_alternative()],
        Confidence::Medium,
        true,
        model_provenance(),
        SensitivityClass::Standard,
        timestamp(),
        None,
    );
    assert_eq!(
        fact_promotion.unwrap_err(),
        AIProposalViolation::ModelClaimsFact
    );
}

fn valid_ai_path_proposal() -> AIProposal {
    let delta = PathDelta::new(
        pursuit_id(),
        1,
        vec![PathChange::SetOperatingState(OperatingState::Master)],
    )
    .unwrap();
    ai_path_proposal_with(AIProposalPayload::PathDelta(delta), None)
}

fn ai_path_proposal_with(payload: AIProposalPayload, expires_at: Option<Timestamp>) -> AIProposal {
    AIProposal::new(
        AIProposalId::parse(ID_B).unwrap(),
        EpistemicClass::Recommendation,
        1,
        payload,
        ProposalTarget {
            target_ref: TypedRef::Pursuit(pursuit_id()),
            base_target_version: Some(1),
        },
        vec![basis()],
        vec![],
        vec!["transfer evidence is currently limited".into()],
        vec![proposal_alternative()],
        Confidence::Medium,
        true,
        model_provenance(),
        SensitivityClass::Standard,
        timestamp(),
        expires_at,
    )
    .unwrap()
}

#[test]
fn ai_path_change_requires_matching_proposal_decision_before_acceptance() {
    let proposal = valid_ai_path_proposal();
    let decision = DecisionRecord::new(
        DecisionRecordId::parse(ID_C).unwrap(),
        DecisionOrigin::SystemRecommended,
        DecisionKind::AiProposalResponse,
        TypedRef::Pursuit(pursuit_id()),
        Some(RecommendationRef::AiProposal(proposal.id().clone())),
        vec![],
        None,
        DecisionOutcome::AiProposalResponse(ProtocolResponseChoice::Accept),
        None,
        ContextSnapshotId::parse(ID_A).unwrap(),
        timestamp(),
    )
    .unwrap();

    let base = composition();
    let AIProposalPayload::PathDelta(delta) = proposal.payload() else {
        panic!("fixture proposal must carry a Path delta");
    };
    let accepted = PursuitPathVersion::new(
        pursuit_id(),
        2,
        Some(1),
        apply_path_delta(&base, delta).unwrap(),
        Some(&base),
        &decision,
        None,
        Some(&proposal),
        IdempotencyKey::new("accept-ai-delta-1").unwrap(),
        timestamp(),
    )
    .unwrap();
    assert_eq!(accepted.source_proposal_ref(), Some(proposal.id()));
    assert_eq!(
        accepted.composition().operating_state(),
        OperatingState::Master
    );

    let disposition = AIProposalDisposition::new(
        &proposal,
        1,
        AIProposalDispositionKind::Accepted,
        Some(&decision),
        "user_accepted",
        timestamp(),
    )
    .unwrap();
    assert_eq!(disposition.decision_ref, Some(decision.id().clone()));
}

#[test]
fn ai_proposal_references_cannot_use_a_generic_decision_kind() {
    let proposal = valid_ai_path_proposal();
    let bypass = DecisionRecord::new(
        DecisionRecordId::parse(ID_C).unwrap(),
        DecisionOrigin::SystemRecommended,
        DecisionKind::ProtocolResponse,
        TypedRef::Pursuit(pursuit_id()),
        Some(RecommendationRef::AiProposal(proposal.id().clone())),
        vec![],
        None,
        DecisionOutcome::ProtocolResponse(ProtocolResponseChoice::Accept),
        None,
        ContextSnapshotId::parse(ID_A).unwrap(),
        timestamp(),
    );

    assert_eq!(
        bypass.unwrap_err(),
        DecisionViolation::AiProposalReferenceRequiresProposalResponse
    );
}

#[test]
fn expired_ai_proposals_cannot_be_accepted_or_modified() {
    let expiry = Timestamp::new("2026-09-02T11:00:00.000Z").unwrap();
    let recorded_at = Timestamp::new("2026-09-02T12:00:00.000Z").unwrap();
    let delta = PathDelta::new(
        pursuit_id(),
        1,
        vec![PathChange::SetOperatingState(OperatingState::Master)],
    )
    .unwrap();
    let proposal = AIProposal::new(
        AIProposalId::parse(ID_B).unwrap(),
        EpistemicClass::Recommendation,
        1,
        AIProposalPayload::PathDelta(delta),
        ProposalTarget {
            target_ref: TypedRef::Pursuit(pursuit_id()),
            base_target_version: Some(1),
        },
        vec![basis()],
        vec![],
        vec![],
        vec![proposal_alternative()],
        Confidence::Medium,
        true,
        model_provenance(),
        SensitivityClass::Standard,
        timestamp(),
        Some(expiry),
    )
    .unwrap();
    let decision = DecisionRecord::new(
        DecisionRecordId::parse(ID_C).unwrap(),
        DecisionOrigin::SystemRecommended,
        DecisionKind::AiProposalResponse,
        TypedRef::Pursuit(pursuit_id()),
        Some(RecommendationRef::AiProposal(proposal.id().clone())),
        vec![],
        None,
        DecisionOutcome::AiProposalResponse(ProtocolResponseChoice::Accept),
        None,
        ContextSnapshotId::parse(ID_A).unwrap(),
        recorded_at.clone(),
    )
    .unwrap();

    assert!(proposal.is_expired_at(&recorded_at));
    assert_eq!(
        AIProposalDisposition::new(
            &proposal,
            1,
            AIProposalDispositionKind::Accepted,
            Some(&decision),
            "accepted_after_expiry",
            recorded_at,
        )
        .unwrap_err(),
        AIProposalViolation::ProposalExpired
    );

    let invalid_expiration = AIProposal::new(
        AIProposalId::parse(ID_B).unwrap(),
        EpistemicClass::Recommendation,
        1,
        AIProposalPayload::ProtocolSelection(protocol_ref()),
        ProposalTarget {
            target_ref: TypedRef::Pursuit(pursuit_id()),
            base_target_version: Some(1),
        },
        vec![basis()],
        vec![],
        vec![],
        vec![proposal_alternative()],
        Confidence::Low,
        true,
        model_provenance(),
        SensitivityClass::Standard,
        timestamp(),
        Some(timestamp()),
    );
    assert_eq!(
        invalid_expiration.unwrap_err(),
        AIProposalViolation::InvalidExpiration
    );
}

#[test]
fn personal_pattern_preserves_uncertainty_and_rejects_single_event_or_fact_promotion() {
    let observation_a = ObservationId::parse(ID_A).unwrap();
    let observation_b = ObservationId::parse(ID_B).unwrap();
    let processor =
        PatternProcessorProvenance::new(PatternProcessorKind::Deterministic, None).unwrap();
    let derivation = DerivationRef {
        derived_by: OwningRef::CapabilityPack(PackId::new("learning").unwrap()),
        derivation_version: "learning.transfer-window.v1".into(),
        method: "compare two bounded session windows".into(),
        source_observation_refs: vec![observation_a.clone(), observation_b.clone()],
    };
    let pattern = PersonalPattern::new(
        PersonalPatternId::parse(ID_C).unwrap(),
        TypedRef::Pursuit(pursuit_id()),
        PackNamespacedPatternKind::new("learning.transfer_time_pattern").unwrap(),
        "Transfer was lower in the later comparable sessions",
        EpistemicClass::Inferred,
        vec![observation_a.clone(), observation_b],
        derivation.clone(),
        processor.clone(),
        Timestamp::new("2026-08-01T10:00:00.000Z").unwrap(),
        timestamp(),
        LiteralValue::Text("two observations; causal direction unknown".into()),
        SensitivityClass::Standard,
        None,
        None,
        timestamp(),
    )
    .unwrap();
    assert_eq!(pattern.epistemic_class(), EpistemicClass::Inferred);

    let single = PersonalPattern::new(
        PersonalPatternId::parse(ID_C).unwrap(),
        TypedRef::Pursuit(pursuit_id()),
        PackNamespacedPatternKind::new("learning.single").unwrap(),
        "One event",
        EpistemicClass::Inferred,
        vec![observation_a],
        derivation,
        processor,
        timestamp(),
        timestamp(),
        LiteralValue::Text("high".into()),
        SensitivityClass::Standard,
        None,
        None,
        timestamp(),
    );
    assert_eq!(
        single.unwrap_err(),
        PersonalPatternViolation::MissingObservations
    );
}

#[test]
fn domain_profile_rejects_curriculum_and_activation_roles() {
    let forbidden = LiteralValue::Object(BTreeMap::from([(
        "capability_sequence".into(),
        LiteralValue::List(vec![LiteralValue::Text("step-one".into())]),
    )]));
    let profile = DomainProfile::new(
        DomainProfileId::parse(ID_A).unwrap(),
        SemVer::new("1.0.0").unwrap(),
        "Mathematics context",
        "Vocabulary and examples only",
        LiteralValue::Text("user imported".into()),
        forbidden,
        BTreeMap::new(),
        vec![],
        vec![],
        vec![method_ref()],
        vec![],
    );
    assert_eq!(
        profile.unwrap_err(),
        DomainProfileViolation::ForbiddenGeneratorRole
    );

    let disguised_sequence = DomainProfile::new(
        DomainProfileId::parse(ID_A).unwrap(),
        SemVer::new("1.0.0").unwrap(),
        "Software context",
        "Context may not smuggle an ordered curriculum",
        LiteralValue::Text("user imported".into()),
        LiteralValue::Object(BTreeMap::from([(
            "topic_order".into(),
            LiteralValue::List(vec![
                LiteralValue::Text("variables".into()),
                LiteralValue::Text("loops".into()),
                LiteralValue::Text("recursion".into()),
            ]),
        )])),
        BTreeMap::new(),
        vec![],
        vec![],
        vec![method_ref()],
        vec![],
    );
    assert_eq!(
        disguised_sequence.unwrap_err(),
        DomainProfileViolation::ForbiddenGeneratorRole
    );

    let disguised_prose = DomainProfile::new(
        DomainProfileId::parse(ID_A).unwrap(),
        SemVer::new("1.0.0").unwrap(),
        "Software context",
        "Free prose cannot become a hidden curriculum",
        LiteralValue::Text("user imported".into()),
        LiteralValue::Object(BTreeMap::from([(
            "domain_context".into(),
            LiteralValue::Text("Day 1: variables; Day 2: loops; Day 3: recursion".into()),
        )])),
        BTreeMap::new(),
        vec![],
        vec![],
        vec![method_ref()],
        vec![],
    );
    assert_eq!(
        disguised_prose.unwrap_err(),
        DomainProfileViolation::ForbiddenGeneratorRole
    );

    let context_only = DomainProfile::new(
        DomainProfileId::parse(ID_A).unwrap(),
        SemVer::new("1.0.0").unwrap(),
        "Baking context",
        "Craft vocabulary without a fixed Path",
        LiteralValue::Text("teacher supplied".into()),
        LiteralValue::Object(BTreeMap::from([(
            "factual_context".into(),
            LiteralValue::Object(BTreeMap::from([(
                "manual_materials_available".into(),
                LiteralValue::Boolean(true),
            )])),
        )])),
        BTreeMap::from([("Practice item".into(), "Technique".into())]),
        vec![],
        vec![],
        vec![method_ref()],
        vec![],
    )
    .unwrap();
    assert!(matches!(
        context_only.context_priors(),
        LiteralValue::Object(_)
    ));
}

#[test]
fn manifest_path_refs_are_sorted_and_duplicate_pursuits_reject() {
    let mut manifest = SystemManifest {
        schema_version: 1,
        manifest_version: 1,
        composer_version: SemVer::new("1.0.0").unwrap(),
        composer_ruleset_version: SemVer::new("1.0.0").unwrap(),
        answers_snapshot_ref: ContextSnapshotId::parse(ID_A).unwrap(),
        enabled_packs: vec![],
        omitted_packs: vec![],
        domain_profiles: vec![],
        protocol_selections: vec![],
        user_parameter_overrides: vec![],
        integration_bindings: vec![],
        path_composition_refs: vec![],
        manifest_changelog: vec![],
    };
    let pursuit_b = PursuitId::parse(ID_B).unwrap();
    manifest
        .set_path_composition_refs(vec![
            PathCompositionRef::new(pursuit_b.clone(), 2).unwrap(),
            PathCompositionRef::new(pursuit_id(), 1).unwrap(),
        ])
        .unwrap();
    assert_eq!(manifest.path_composition_refs[0].pursuit_ref, pursuit_id());

    assert_eq!(
        manifest
            .set_path_composition_refs(vec![
                PathCompositionRef::new(pursuit_b.clone(), 2).unwrap(),
                PathCompositionRef::new(pursuit_b, 3).unwrap(),
            ])
            .unwrap_err(),
        ManifestPathViolation::DuplicatePursuit
    );

    assert_eq!(
        manifest
            .set_path_composition_refs(vec![PathCompositionRef {
                pursuit_ref: pursuit_id(),
                path_version: 0,
            }])
            .unwrap_err(),
        ManifestPathViolation::InvalidPathVersion
    );
}

#[test]
fn aggregate_validation_rejects_forged_nested_epistemic_and_path_values() {
    let forged_constraint = PathConstraint {
        key: "source_limit".into(),
        value: LiteralValue::Boolean(true),
        provenance: PathConstraintProvenance::PersonalObservation,
        provenance_ref: Some(TypedRef::Pursuit(pursuit_id())),
    };
    let invalid_composition = PathComposition::new(
        1,
        DirectionSpec::new("A user-owned direction", DirectionStance::Stable, None).unwrap(),
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        OperatingState::Pursue,
        vec![forged_constraint],
        BTreeMap::new(),
        ExplanationPreference::Brief,
    );
    assert_eq!(
        invalid_composition.unwrap_err(),
        PathViolation::PersonalConstraintRequiresObservation
    );

    let forged_basis = EpistemicBasisRef {
        reference: EpistemicBasisTarget::UserRecord(TypedRef::Pursuit(pursuit_id())),
        class: EpistemicClass::Recommendation,
        source_or_derivation_ref: LiteralValue::Null,
    };
    let invalid_proposal = AIProposal::new(
        AIProposalId::parse(ID_B).unwrap(),
        EpistemicClass::Recommendation,
        1,
        AIProposalPayload::PathDelta(
            PathDelta::new(
                pursuit_id(),
                1,
                vec![PathChange::SetOperatingState(OperatingState::Maintain)],
            )
            .unwrap(),
        ),
        ProposalTarget {
            target_ref: TypedRef::Pursuit(pursuit_id()),
            base_target_version: Some(1),
        },
        vec![forged_basis],
        vec![],
        vec![],
        vec![proposal_alternative()],
        Confidence::Low,
        true,
        model_provenance(),
        SensitivityClass::Standard,
        timestamp(),
        None,
    );
    assert_eq!(
        invalid_proposal.unwrap_err(),
        AIProposalViolation::RecommendationUsedAsBasis
    );

    let forged_delta = PathDelta {
        pursuit_ref: pursuit_id(),
        base_path_version: 1,
        operations: vec![],
    };
    let invalid_delta_proposal = AIProposal::new(
        AIProposalId::parse(ID_B).unwrap(),
        EpistemicClass::Recommendation,
        1,
        AIProposalPayload::PathDelta(forged_delta),
        ProposalTarget {
            target_ref: TypedRef::Pursuit(pursuit_id()),
            base_target_version: Some(1),
        },
        vec![basis()],
        vec![],
        vec![],
        vec![proposal_alternative()],
        Confidence::Low,
        true,
        model_provenance(),
        SensitivityClass::Standard,
        timestamp(),
        None,
    );
    assert_eq!(
        invalid_delta_proposal.unwrap_err(),
        AIProposalViolation::Path(PathViolation::EmptyDelta)
    );

    let invalid_model_proposal = AIProposal::new(
        AIProposalId::parse(ID_B).unwrap(),
        EpistemicClass::Hypothesis,
        1,
        AIProposalPayload::ReflectionStructure {
            reflection_ref: ReflectionOutcomeId::parse(ID_C).unwrap(),
            structure: LiteralValue::Object(BTreeMap::new()),
        },
        ProposalTarget {
            target_ref: TypedRef::ReflectionOutcome(ReflectionOutcomeId::parse(ID_C).unwrap()),
            base_target_version: None,
        },
        vec![basis()],
        vec![],
        vec![],
        vec![],
        Confidence::Low,
        false,
        ModelProvenance {
            provider: "".into(),
            model: "test-model".into(),
            model_version: "1".into(),
            prompt_template_version: "reflection.v1".into(),
        },
        SensitivityClass::ReflectivePrivate,
        timestamp(),
        None,
    );
    assert_eq!(
        invalid_model_proposal.unwrap_err(),
        AIProposalViolation::IncompleteModelProvenance
    );
}

#[test]
fn integration_actions_require_registered_operations_and_writes_require_safety_contracts() {
    let unsafe_write = OperationDefinition::new(
        IntegrationOperationId::new("calendar.create_event").unwrap(),
        IntegrationOperationMode::Write,
        LiteralValue::Null,
        LiteralValue::Null,
        vec!["calendar.write".into()],
        None,
        None,
        None,
        LiteralValue::Null,
    );
    assert_eq!(
        unsafe_write.unwrap_err(),
        IntegrationViolation::UnsafeWriteContract
    );

    let read = OperationDefinition::new(
        IntegrationOperationId::new("anki.query_due_cards").unwrap(),
        IntegrationOperationMode::Read,
        LiteralValue::Null,
        LiteralValue::Null,
        vec!["anki.read".into()],
        None,
        None,
        None,
        LiteralValue::Null,
    )
    .unwrap();
    let capability = IntegrationCapability::new(
        IntegrationCapabilityId::new("integration.anki.reviews").unwrap(),
        SemVer::new("1.0.0").unwrap(),
        "anki",
        vec![read],
        "Record a manual retrieval session",
        LiteralValue::Text("source deck and review timestamp".into()),
    )
    .unwrap();
    assert!(IntegrationActionProposal::new(
        &capability,
        IntegrationOperationId::new("anki.query_due_cards").unwrap(),
        LiteralValue::Null,
    )
    .is_ok());
    assert_eq!(
        IntegrationActionProposal::new(
            &capability,
            IntegrationOperationId::new("anki.delete_deck").unwrap(),
            LiteralValue::Null,
        )
        .unwrap_err(),
        IntegrationViolation::UnknownOperation
    );

    let unsafe_operation_id = IntegrationOperationId::new("calendar.create_event").unwrap();
    let forged_capability = IntegrationCapability {
        id: IntegrationCapabilityId::new("integration.calendar.events").unwrap(),
        version: SemVer::new("1.0.0").unwrap(),
        provider_family: "calendar".into(),
        operations: vec![OperationDefinition {
            operation_id: unsafe_operation_id.clone(),
            mode: IntegrationOperationMode::Write,
            request_schema: LiteralValue::Null,
            response_schema: LiteralValue::Null,
            required_scopes: vec!["calendar.write".into()],
            target_allowlist_kind: None,
            idempotency_strategy: None,
            precondition_strategy: None,
            failure_schema: LiteralValue::Null,
        }],
        manual_fallback: "Record the commitment manually".into(),
        provenance_schema: LiteralValue::Null,
    };
    assert_eq!(
        IntegrationActionProposal::new(
            &forged_capability,
            unsafe_operation_id,
            LiteralValue::Null,
        )
        .unwrap_err(),
        IntegrationViolation::UnsafeWriteContract
    );
}

#[test]
fn method_definition_is_distinct_from_evidence_and_protocol_policy() {
    let method = MethodDefinition::new(
        MethodId::new("method.after_action_review").unwrap(),
        SemVer::new("1.0.0").unwrap(),
        "After-Action Review",
        "reflection",
        "Learn from a bounded completed action",
        vec!["Identify a next testable adjustment".into()],
        None,
        vec![MethodEvidenceRef::DesignHypothesis(
            DesignHypothesisId::new("hypothesis.aars_support_learning").unwrap(),
        )],
        MethodEvidenceStatus::DesignHypothesis,
        vec![],
        vec!["bounded review".into()],
        vec!["not a causal diagnosis".into()],
        vec!["attention".into()],
        vec!["reflection can be burdensome".into()],
        vec![],
        vec![],
        vec![protocol_ref()],
        vec![MethodAiRole::Reflection],
        vec![],
        None,
        LiteralValue::Text("source-controlled catalog".into()),
    )
    .unwrap();

    assert_eq!(
        method.evidence_status,
        MethodEvidenceStatus::DesignHypothesis
    );
    assert_eq!(method.protocol_variants, vec![protocol_ref()]);
}

// ---------------------------------------------------------------------------
// Accepting an AI Path proposal
//
// An accepted AI proposal must be exactly the change that was proposed and reviewed. Citing its id
// proves none of that, so acceptance re-derives the composition from the proposal's own delta.
// ---------------------------------------------------------------------------

fn ai_acceptance_decision(proposal: &AIProposal) -> DecisionRecord {
    DecisionRecord::new(
        DecisionRecordId::parse(ID_C).unwrap(),
        DecisionOrigin::SystemRecommended,
        DecisionKind::AiProposalResponse,
        TypedRef::Pursuit(pursuit_id()),
        Some(RecommendationRef::AiProposal(proposal.id().clone())),
        vec![],
        None,
        DecisionOutcome::AiProposalResponse(ProtocolResponseChoice::Accept),
        None,
        ContextSnapshotId::parse(ID_A).unwrap(),
        timestamp(),
    )
    .unwrap()
}

fn accept_ai_proposal(
    proposal: &AIProposal,
    composition: PathComposition,
    previous_composition: Option<&PathComposition>,
    previous_path_version: Option<u64>,
) -> Result<PursuitPathVersion, PathViolation> {
    let path_version = previous_path_version.map_or(1, |previous| previous + 1);
    PursuitPathVersion::new(
        pursuit_id(),
        path_version,
        previous_path_version,
        composition,
        previous_composition,
        &ai_acceptance_decision(proposal),
        None,
        Some(proposal),
        IdempotencyKey::new("accept-ai-delta").unwrap(),
        timestamp(),
    )
}

fn path_delta_of(proposal: &AIProposal) -> &PathDelta {
    let AIProposalPayload::PathDelta(delta) = proposal.payload() else {
        panic!("fixture proposal must carry a Path delta");
    };
    delta
}

/// The hole this closes: pairing a valid accepted-proposal decision with a composition that is not
/// what the proposal actually asked for. The id matches, the decision matches, and the change is
/// still not the reviewed one.
#[test]
fn an_accepted_ai_proposal_cannot_carry_an_arbitrary_composition() {
    let proposal = valid_ai_path_proposal();
    let base = composition();

    let error = accept_ai_proposal(&proposal, composition(), Some(&base), Some(1)).unwrap_err();

    assert_eq!(error, PathViolation::AiProposalCompositionMismatch);
}

/// Applying the proposal's own delta to the exact base is what acceptance requires, and produces
/// the reviewed change rather than merely a plausible one.
#[test]
fn an_accepted_ai_proposal_must_recompute_from_its_own_delta() {
    let proposal = valid_ai_path_proposal();
    let base = composition();
    let expected = apply_path_delta(&base, path_delta_of(&proposal)).unwrap();

    let accepted = accept_ai_proposal(&proposal, expected, Some(&base), Some(1)).unwrap();

    assert_eq!(
        accepted.composition().operating_state(),
        OperatingState::Master
    );
    assert_eq!(accepted.source_proposal_ref(), Some(proposal.id()));
}

/// Without the base it was computed from, the delta cannot be checked at all, so acceptance fails
/// closed rather than trusting the caller's composition.
#[test]
fn accepting_an_ai_proposal_without_the_previous_composition_is_rejected() {
    let proposal = valid_ai_path_proposal();
    let base = composition();
    let expected = apply_path_delta(&base, path_delta_of(&proposal)).unwrap();

    let error = accept_ai_proposal(&proposal, expected, None, Some(1)).unwrap_err();

    assert_eq!(error, PathViolation::MissingPreviousComposition);
}

/// A proposal is computed against one exact Path version. If the Path moved on before the user
/// accepted, the proposal is stale — its delta describes a change from a base that no longer is the
/// current one, and accepting it would silently apply the change to something else.
#[test]
fn an_ai_proposal_whose_base_version_has_moved_on_is_stale() {
    let proposal = valid_ai_path_proposal();
    let base = composition();
    let expected = apply_path_delta(&base, path_delta_of(&proposal)).unwrap();

    // The proposal was computed against version 1; the Path is now at version 5.
    let error = accept_ai_proposal(&proposal, expected, Some(&base), Some(5)).unwrap_err();

    assert_eq!(error, PathViolation::AiProposalStaleBase);
}

/// A first Path version has no previous version for a delta to be based on, so a proposal cannot
/// be accepted as the very first accepted composition.
#[test]
fn an_ai_proposal_cannot_create_the_first_path_version() {
    let proposal = valid_ai_path_proposal();
    let base = composition();
    let expected = apply_path_delta(&base, path_delta_of(&proposal)).unwrap();

    let error = accept_ai_proposal(&proposal, expected, Some(&base), None).unwrap_err();

    assert_eq!(error, PathViolation::AiProposalStaleBase);
}

/// An expired proposal is not a reviewable proposal any more, whatever it once said. The window
/// here is realistic: generated, then expired, then accepted.
#[test]
fn an_expired_ai_proposal_cannot_be_accepted() {
    let delta = PathDelta::new(
        pursuit_id(),
        1,
        vec![PathChange::SetOperatingState(OperatingState::Master)],
    )
    .unwrap();
    let expires_at = Timestamp::new("2026-09-02T11:00:00.000Z").unwrap();
    let accepted_at = Timestamp::new("2026-09-02T12:00:00.000Z").unwrap();
    let proposal = ai_path_proposal_with(AIProposalPayload::PathDelta(delta), Some(expires_at));
    let base = composition();
    let expected = apply_path_delta(&base, path_delta_of(&proposal)).unwrap();

    let error = PursuitPathVersion::new(
        pursuit_id(),
        2,
        Some(1),
        expected,
        Some(&base),
        &ai_acceptance_decision(&proposal),
        None,
        Some(&proposal),
        IdempotencyKey::new("accept-expired-ai-delta").unwrap(),
        accepted_at,
    )
    .unwrap_err();

    assert_eq!(error, PathViolation::AiProposalExpired);
}

/// A delta that names an entry the base does not contain describes a change that cannot be made,
/// so it is rejected rather than silently treated as a no-op.
#[test]
fn a_delta_removing_an_absent_entry_is_rejected() {
    let base = composition();
    let delta = PathDelta::new(
        pursuit_id(),
        1,
        vec![PathChange::RemoveMethod(VersionedMethodRef {
            method_id: MethodId::new("method.never_added").unwrap(),
            method_version: SemVer::new("1.0.0").unwrap(),
        })],
    )
    .unwrap();

    let error = apply_path_delta(&base, &delta).unwrap_err();

    assert_eq!(error, PathViolation::DeltaTargetNotFound("method_refs"));
}

/// Adding an entry the base already contains is likewise a change that cannot be made.
#[test]
fn a_delta_adding_a_duplicate_entry_is_rejected() {
    let base = composition();
    let existing = base
        .method_refs()
        .first()
        .expect("fixture has a Method")
        .clone();
    let delta = PathDelta::new(pursuit_id(), 1, vec![PathChange::AddMethod(existing)]).unwrap();

    let error = apply_path_delta(&base, &delta).unwrap_err();

    assert_eq!(error, PathViolation::DuplicateEntry("method_refs"));
}

/// Mirrors the desktop backend's first-Path acceptance call shape exactly
/// (`apps/flourish-desktop/src-tauri/src/product.rs`), argument for argument. That crate needs GTK
/// system libraries and cannot be compiled in every environment, so this test is where a signature
/// change to `PursuitPathVersion::new` gets caught before it reaches a workstation build.
#[test]
fn the_desktop_first_path_acceptance_call_shape_still_compiles() {
    let pursuit_id = pursuit_id();
    let decision = manual_path_decision();
    let draft_id = PathDraftId::parse(ID_C).unwrap();
    let expected_revision: u64 = 1;
    let command_id = "accept-first-path".to_owned();
    let accepted_at = timestamp();

    let accepted = PursuitPathVersion::new(
        pursuit_id.clone(),
        1,
        None,
        composition(),
        None,
        &decision,
        Some(PathDraftRevisionRef {
            draft_id,
            revision: expected_revision,
        }),
        None,
        IdempotencyKey::new(command_id).unwrap(),
        accepted_at,
    )
    .unwrap();

    assert_eq!(accepted.path_version(), 1);
    assert_eq!(accepted.source_proposal_ref(), None);
}

/// Reordering clamps to the end rather than failing or panicking on an out-of-range index, so a
/// proposal that asks for "last" in whatever terms it has is still applicable.
#[test]
fn a_reorder_delta_clamps_an_out_of_range_index() {
    let base = composition();
    let capability = base
        .capability_refs()
        .first()
        .expect("fixture has a capability")
        .clone();
    let delta = PathDelta::new(
        pursuit_id(),
        1,
        vec![PathChange::ReorderCapability(capability.clone(), u32::MAX)],
    )
    .unwrap();

    let applied = apply_path_delta(&base, &delta).unwrap();

    assert_eq!(applied.capability_refs().last(), Some(&capability));
    assert_eq!(
        applied.capability_refs().len(),
        base.capability_refs().len()
    );
}

/// Removing an override that was never set is a change that cannot be made, and is rejected rather
/// than treated as a silent no-op — the same rule the list operations follow.
#[test]
fn a_delta_removing_an_absent_override_is_rejected() {
    let base = composition();
    let delta = PathDelta::new(
        pursuit_id(),
        1,
        vec![PathChange::RemoveOverride("never.set".into())],
    )
    .unwrap();

    let error = apply_path_delta(&base, &delta).unwrap_err();

    assert_eq!(error, PathViolation::DeltaTargetNotFound("user_overrides"));
}

/// Setting a constraint replaces any existing one with the same key rather than accumulating two
/// conflicting values for it.
#[test]
fn a_set_constraint_delta_replaces_the_existing_key() {
    let base = composition();
    let existing = base
        .constraints()
        .first()
        .expect("fixture has a constraint")
        .clone();
    let replacement = PathConstraint {
        key: existing.key.clone(),
        value: LiteralValue::Unsigned(240),
        provenance: PathConstraintProvenance::UserStated,
        provenance_ref: None,
    };
    let delta = PathDelta::new(
        pursuit_id(),
        1,
        vec![PathChange::SetConstraint(replacement)],
    )
    .unwrap();

    let applied = apply_path_delta(&base, &delta).unwrap();

    assert_eq!(applied.constraints().len(), base.constraints().len());
    let updated = applied
        .constraints()
        .iter()
        .find(|constraint| constraint.key == existing.key)
        .expect("constraint survives");
    assert_eq!(updated.value, LiteralValue::Unsigned(240));
}
