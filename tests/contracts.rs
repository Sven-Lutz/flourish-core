use flourish_core::*;
use std::collections::BTreeMap;

const ID_A: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
const ID_B: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAW";
const ID_C: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAX";

#[derive(Default)]
struct Registries {
    facts: BTreeMap<ContextFactRef, ContextFactDefinition>,
    topics: BTreeMap<EventTopic, EventTopicDefinition>,
}

impl Registries {
    fn with_fact(mut self, definition: ContextFactDefinition) -> Self {
        self.facts.insert(definition.fact_id().clone(), definition);
        self
    }

    fn with_topic(mut self, definition: EventTopicDefinition) -> Self {
        self.topics.insert(definition.topic().clone(), definition);
        self
    }
}

impl ContextFactRegistry for Registries {
    fn resolve_fact(&self, fact_id: &ContextFactRef) -> Option<&ContextFactDefinition> {
        self.facts.get(fact_id)
    }
}

impl EventTopicRegistry for Registries {
    fn resolve_topic(&self, topic: &EventTopic) -> Option<&EventTopicDefinition> {
        self.topics.get(topic)
    }
}

#[derive(Default)]
struct DeclaredAccess {
    facts: Vec<ContextFactRef>,
    topics: Vec<EventTopic>,
}

impl PredicateAccessDeclaration for DeclaredAccess {
    fn allows_fact(&self, fact_id: &ContextFactRef) -> bool {
        self.facts.contains(fact_id)
    }

    fn allows_topic(&self, event_topic: &EventTopic) -> bool {
        self.topics.contains(event_topic)
    }
}

fn owner() -> OwningRef {
    OwningRef::KernelService(ServiceId::ContextAssembler)
}

fn fact_definition(
    name: &str,
    category: FactCategory,
    operational_basis: Option<OperationalBasis>,
) -> ContextFactDefinition {
    ContextFactDefinition::new(
        ContextFactRef::new(name).unwrap(),
        category,
        operational_basis,
        owner(),
        ValueSchema::Boolean,
        vec![],
    )
    .unwrap()
}

fn topic_definition(
    name: &str,
    category: FactCategory,
    operational_basis: Option<OperationalBasis>,
) -> EventTopicDefinition {
    EventTopicDefinition::new(
        EventTopic::new(name).unwrap(),
        1,
        OwningRef::KernelService(ServiceId::EventBus),
        category,
        operational_basis,
    )
    .unwrap()
}

fn fact_leaves(name: &str) -> [Predicate; 2] {
    let fact_id = ContextFactRef::new(name).unwrap();
    [
        Predicate::Fact {
            fact_id: fact_id.clone(),
        },
        Predicate::Compare {
            fact_id,
            op: CompareOp::Eq,
            literal: LiteralValue::Boolean(true),
        },
    ]
}

fn topic_leaves(name: &str) -> [Predicate; 2] {
    let event_topic = EventTopic::new(name).unwrap();
    [
        Predicate::EventOccurred {
            event_topic: event_topic.clone(),
            within: Some(DurationMillis::new(60_000)),
        },
        Predicate::TimeElapsedSince {
            event_topic,
            duration: DurationMillis::new(1_000),
        },
    ]
}

fn user_pack_decision(kind: &str) -> DecisionRecord {
    DecisionRecord::new(
        DecisionRecordId::parse(ID_A).unwrap(),
        DecisionOrigin::UserInitiated,
        DecisionKind::pack_specific(kind).unwrap(),
        TypedRef::Pursuit(PursuitId::parse(ID_B).unwrap()),
        None,
        vec![],
        None,
        DecisionOutcome::PackSpecific(LiteralValue::Boolean(true)),
        None,
        ContextSnapshotId::parse(ID_C).unwrap(),
        Timestamp::new("2026-08-30T12:00:00.000Z").unwrap(),
    )
    .unwrap()
}

#[test]
fn operational_basis_is_closed_to_operational_registrations() {
    let fact = ContextFactDefinition::new(
        ContextFactRef::new("system.schema_valid").unwrap(),
        FactCategory::SystemIntegrity,
        Some(OperationalBasis::TechnicalAvailability),
        owner(),
        ValueSchema::Boolean,
        vec![],
    );
    assert_eq!(
        fact.unwrap_err(),
        RegistryDefinitionViolation::OperationalBasisOnNonOperationalCategory
    );

    let topic = EventTopicDefinition::new(
        EventTopic::new("system.schema_validated").unwrap(),
        1,
        OwningRef::KernelService(ServiceId::EventBus),
        FactCategory::Behavioral,
        Some(OperationalBasis::DirectActionCompletion),
    );
    assert_eq!(
        topic.unwrap_err(),
        RegistryDefinitionViolation::OperationalBasisOnNonOperationalCategory
    );
}

#[test]
fn invariant_accepts_system_integrity_fact_and_event_leaf_families() {
    let registry = Registries::default()
        .with_fact(fact_definition(
            "system.schema_valid",
            FactCategory::SystemIntegrity,
            None,
        ))
        .with_topic(topic_definition(
            "system.schema_validated",
            FactCategory::SystemIntegrity,
            None,
        ));

    for predicate in fact_leaves("system.schema_valid")
        .into_iter()
        .chain(topic_leaves("system.schema_validated"))
    {
        assert_eq!(
            check_invariant_eligibility(&predicate, &registry, &registry),
            Ok(())
        );
    }
}

#[test]
fn invariant_rejects_every_non_system_category_for_both_registry_families() {
    for (index, category) in [
        FactCategory::Operational,
        FactCategory::Behavioral,
        FactCategory::Motivational,
        FactCategory::SelfReport,
        FactCategory::WellbeingDerived,
    ]
    .into_iter()
    .enumerate()
    {
        let fact_name = format!("test.fact.{index}");
        let topic_name = format!("test.topic.{index}");
        let basis =
            (category == FactCategory::Operational).then_some(OperationalBasis::DirectActionCount);
        let registry = Registries::default()
            .with_fact(fact_definition(&fact_name, category, basis))
            .with_topic(topic_definition(&topic_name, category, basis));

        for predicate in fact_leaves(&fact_name)
            .into_iter()
            .chain(topic_leaves(&topic_name))
        {
            assert!(check_invariant_eligibility(&predicate, &registry, &registry).is_err());
        }
    }
}

#[test]
fn condition_bound_accepts_system_integrity_fact_unconditionally() {
    let registry = Registries::default().with_fact(fact_definition(
        "system.permission_valid",
        FactCategory::SystemIntegrity,
        None,
    ));
    for predicate in fact_leaves("system.permission_valid") {
        assert_eq!(
            check_condition_bound_eligibility(&predicate, &registry, &registry),
            Ok(())
        );
    }
}

#[test]
fn condition_bound_accepts_system_integrity_event_unconditionally() {
    let registry = Registries::default().with_topic(topic_definition(
        "system.permission_validated",
        FactCategory::SystemIntegrity,
        None,
    ));
    for predicate in topic_leaves("system.permission_validated") {
        assert_eq!(
            check_condition_bound_eligibility(&predicate, &registry, &registry),
            Ok(())
        );
    }
}

fn assert_operational_basis_accepts_every_leaf(basis: OperationalBasis) {
    let fact_name = format!("operational.fact.{basis:?}");
    let topic_name = format!("operational.topic.{basis:?}");
    let registry = Registries::default()
        .with_fact(fact_definition(
            &fact_name,
            FactCategory::Operational,
            Some(basis),
        ))
        .with_topic(topic_definition(
            &topic_name,
            FactCategory::Operational,
            Some(basis),
        ));

    // The upstream review NIT is resolved deliberately: every OperationalBasis
    // variant exercises both ContextFact leaves and both EventTopic leaves.
    for predicate in fact_leaves(&fact_name)
        .into_iter()
        .chain(topic_leaves(&topic_name))
    {
        assert_eq!(
            check_condition_bound_eligibility(&predicate, &registry, &registry),
            Ok(()),
            "{basis:?} should qualify {predicate:?}"
        );
    }
}

#[test]
fn condition_bound_accepts_time_boundary_for_both_leaf_families() {
    assert_operational_basis_accepts_every_leaf(OperationalBasis::TimeBoundary);
}

#[test]
fn condition_bound_accepts_direct_action_count_for_both_leaf_families() {
    assert_operational_basis_accepts_every_leaf(OperationalBasis::DirectActionCount);
}

#[test]
fn condition_bound_accepts_direct_action_completion_for_both_leaf_families() {
    assert_operational_basis_accepts_every_leaf(OperationalBasis::DirectActionCompletion);
}

#[test]
fn condition_bound_accepts_technical_availability_for_both_leaf_families() {
    assert_operational_basis_accepts_every_leaf(OperationalBasis::TechnicalAvailability);
}

#[test]
fn condition_bound_rejects_operational_fact_and_event_without_basis() {
    let registry = Registries::default()
        .with_fact(fact_definition(
            "operational.due_work_count",
            FactCategory::Operational,
            None,
        ))
        .with_topic(topic_definition(
            "operational.manifest_changed",
            FactCategory::Operational,
            None,
        ));

    for predicate in fact_leaves("operational.due_work_count") {
        assert!(matches!(
            check_condition_bound_eligibility(&predicate, &registry, &registry),
            Err(ConditionBoundViolation::MissingFactOperationalBasis(_))
        ));
    }
    for predicate in topic_leaves("operational.manifest_changed") {
        assert!(matches!(
            check_condition_bound_eligibility(&predicate, &registry, &registry),
            Err(ConditionBoundViolation::MissingTopicOperationalBasis(_))
        ));
    }
}

macro_rules! prohibited_category_tests {
    ($fact_test:ident, $topic_test:ident, $category:expr, $stem:literal) => {
        #[test]
        fn $fact_test() {
            let name = concat!($stem, ".fact");
            let registry = Registries::default().with_fact(fact_definition(name, $category, None));
            for predicate in fact_leaves(name) {
                assert!(matches!(
                    check_condition_bound_eligibility(&predicate, &registry, &registry),
                    Err(ConditionBoundViolation::IneligibleFactCategory { .. })
                ));
            }
        }

        #[test]
        fn $topic_test() {
            let name = concat!($stem, ".topic");
            let registry =
                Registries::default().with_topic(topic_definition(name, $category, None));
            for predicate in topic_leaves(name) {
                assert!(matches!(
                    check_condition_bound_eligibility(&predicate, &registry, &registry),
                    Err(ConditionBoundViolation::IneligibleTopicCategory { .. })
                ));
            }
        }
    };
}

prohibited_category_tests!(
    condition_bound_rejects_behavioral_fact_leaves,
    condition_bound_rejects_behavioral_topic_leaves,
    FactCategory::Behavioral,
    "behavioral"
);
prohibited_category_tests!(
    condition_bound_rejects_motivational_fact_leaves,
    condition_bound_rejects_motivational_topic_leaves,
    FactCategory::Motivational,
    "motivational"
);
prohibited_category_tests!(
    condition_bound_rejects_self_report_fact_leaves,
    condition_bound_rejects_self_report_topic_leaves,
    FactCategory::SelfReport,
    "self_report"
);
prohibited_category_tests!(
    condition_bound_rejects_wellbeing_derived_fact_leaves,
    condition_bound_rejects_wellbeing_derived_topic_leaves,
    FactCategory::WellbeingDerived,
    "wellbeing_derived"
);

#[test]
fn blocker_regression_rejects_motivational_engagement_score_event() {
    let registry = Registries::default().with_topic(topic_definition(
        "motivation.engagement_score_low",
        FactCategory::Motivational,
        None,
    ));
    let predicate = Predicate::EventOccurred {
        event_topic: EventTopic::new("motivation.engagement_score_low").unwrap(),
        within: None,
    };

    assert!(matches!(
        check_condition_bound_eligibility(&predicate, &registry, &registry),
        Err(ConditionBoundViolation::IneligibleTopicCategory {
            category: FactCategory::Motivational,
            ..
        })
    ));
}

#[test]
fn condition_bound_rejects_unresolved_fact_and_topic() {
    let registry = Registries::default();
    assert!(matches!(
        check_condition_bound_eligibility(
            &Predicate::fact(ContextFactRef::new("missing.fact").unwrap()),
            &registry,
            &registry,
        ),
        Err(ConditionBoundViolation::UnregisteredFact(_))
    ));
    assert!(matches!(
        check_condition_bound_eligibility(
            &Predicate::event_occurred(EventTopic::new("missing.topic").unwrap()),
            &registry,
            &registry,
        ),
        Err(ConditionBoundViolation::UnregisteredTopic(_))
    ));
}

#[test]
fn invariant_rejects_unresolved_fact_and_topic() {
    let registry = Registries::default();
    assert!(matches!(
        check_invariant_eligibility(
            &Predicate::fact(ContextFactRef::new("missing.fact").unwrap()),
            &registry,
            &registry,
        ),
        Err(InvariantViolation::UnregisteredFact(_))
    ));
    assert!(matches!(
        check_invariant_eligibility(
            &Predicate::event_occurred(EventTopic::new("missing.topic").unwrap()),
            &registry,
            &registry,
        ),
        Err(InvariantViolation::UnregisteredTopic(_))
    ));
}

#[test]
fn condition_and_invariant_rules_provably_diverge() {
    let registry = Registries::default().with_fact(fact_definition(
        "focus.sessions_logged",
        FactCategory::Operational,
        Some(OperationalBasis::DirectActionCount),
    ));
    let predicate = Predicate::fact(ContextFactRef::new("focus.sessions_logged").unwrap());

    assert_eq!(
        check_condition_bound_eligibility(&predicate, &registry, &registry),
        Ok(())
    );
    assert!(matches!(
        check_invariant_eligibility(&predicate, &registry, &registry),
        Err(InvariantViolation::IneligibleFact {
            category: FactCategory::Operational,
            ..
        })
    ));
}

#[test]
fn both_eligibility_rules_walk_compound_predicates() {
    let registry = Registries::default()
        .with_fact(fact_definition(
            "system.valid",
            FactCategory::SystemIntegrity,
            None,
        ))
        .with_topic(topic_definition(
            "motivation.hidden",
            FactCategory::Motivational,
            None,
        ));
    let predicate = Predicate::And(vec![
        Predicate::fact(ContextFactRef::new("system.valid").unwrap()),
        Predicate::Not(Box::new(Predicate::event_occurred(
            EventTopic::new("motivation.hidden").unwrap(),
        ))),
    ]);

    assert!(check_condition_bound_eligibility(&predicate, &registry, &registry).is_err());
    assert!(check_invariant_eligibility(&predicate, &registry, &registry).is_err());
}

#[test]
fn predicate_access_checks_both_leaf_families_and_compound_trees() {
    let allowed_fact = ContextFactRef::new("system.allowed").unwrap();
    let allowed_topic = EventTopic::new("system.allowed_event").unwrap();
    let access = DeclaredAccess {
        facts: vec![allowed_fact.clone()],
        topics: vec![allowed_topic.clone()],
    };
    let allowed = Predicate::And(vec![
        Predicate::Fact {
            fact_id: allowed_fact.clone(),
        },
        Predicate::Compare {
            fact_id: allowed_fact,
            op: CompareOp::Eq,
            literal: LiteralValue::Boolean(true),
        },
        Predicate::EventOccurred {
            event_topic: allowed_topic.clone(),
            within: None,
        },
        Predicate::TimeElapsedSince {
            event_topic: allowed_topic,
            duration: DurationMillis::new(1_000),
        },
    ]);
    assert_eq!(check_predicate_access(&allowed, &access), Ok(()));

    for predicate in fact_leaves("system.undeclared") {
        assert!(matches!(
            check_predicate_access(&predicate, &access),
            Err(PredicateAccessViolation::UndeclaredFact(_))
        ));
    }
    for predicate in topic_leaves("system.unsubscribed") {
        assert!(matches!(
            check_predicate_access(&predicate, &access),
            Err(PredicateAccessViolation::UnsubscribedTopic(_))
        ));
    }
}

#[test]
fn predicate_access_and_category_eligibility_are_independent_gates() {
    let motivational_topic = EventTopic::new("motivation.visible_but_ineligible").unwrap();
    let system_fact = ContextFactRef::new("system.eligible_but_undeclared").unwrap();
    let registry = Registries::default()
        .with_topic(topic_definition(
            motivational_topic.as_str(),
            FactCategory::Motivational,
            None,
        ))
        .with_fact(fact_definition(
            system_fact.as_str(),
            FactCategory::SystemIntegrity,
            None,
        ));
    let access = DeclaredAccess {
        facts: vec![],
        topics: vec![motivational_topic.clone()],
    };

    let declared_but_ineligible = Predicate::event_occurred(motivational_topic);
    assert_eq!(
        check_predicate_access(&declared_but_ineligible, &access),
        Ok(())
    );
    assert!(
        check_condition_bound_eligibility(&declared_but_ineligible, &registry, &registry).is_err()
    );

    let eligible_but_undeclared = Predicate::fact(system_fact);
    assert!(check_predicate_access(&eligible_but_undeclared, &access).is_err());
    assert_eq!(
        check_invariant_eligibility(&eligible_but_undeclared, &registry, &registry),
        Ok(())
    );
}

#[test]
fn commitment_device_constructor_cannot_bypass_condition_validation() {
    let registry = Registries::default().with_topic(topic_definition(
        "motivation.engagement_score_low",
        FactCategory::Motivational,
        None,
    ));
    let creation = user_pack_decision("focus.commitment_device_start");
    let condition =
        Predicate::event_occurred(EventTopic::new("motivation.engagement_score_low").unwrap());

    let result = CommitmentDevice::new(
        CommitmentDeviceId::parse(ID_B).unwrap(),
        ScopeRef::Global,
        CommitmentBoundBy::Condition,
        None,
        Some(condition),
        "focus.in_app_block".into(),
        "restore the previous in-app state".into(),
        CommitmentDeviceStatus::Armed,
        &creation,
        &registry,
        &registry,
    );

    assert!(matches!(
        result,
        Err(CommitmentDeviceViolation::ConditionBound(
            ConditionBoundViolation::IneligibleTopicCategory {
                category: FactCategory::Motivational,
                ..
            }
        ))
    ));
}

#[test]
fn commitment_device_constructor_accepts_qualified_operational_condition() {
    let registry = Registries::default().with_fact(fact_definition(
        "focus.sessions_logged",
        FactCategory::Operational,
        Some(OperationalBasis::DirectActionCount),
    ));
    let creation = user_pack_decision("focus.commitment_device_start");
    let device = CommitmentDevice::new(
        CommitmentDeviceId::parse(ID_B).unwrap(),
        ScopeRef::Global,
        CommitmentBoundBy::Condition,
        None,
        Some(Predicate::fact(
            ContextFactRef::new("focus.sessions_logged").unwrap(),
        )),
        "focus.in_app_block".into(),
        "restore the previous in-app state".into(),
        CommitmentDeviceStatus::Armed,
        &creation,
        &registry,
        &registry,
    )
    .unwrap();

    assert!(device.reversible());
    assert!(device.condition_bound().is_some());
}

#[test]
fn decision_record_enforces_origin_recommendation_and_completion_rules() {
    let user_completion = DecisionRecord::new(
        DecisionRecordId::parse(ID_A).unwrap(),
        DecisionOrigin::UserInitiated,
        DecisionKind::PursuitCompletion,
        TypedRef::Pursuit(PursuitId::parse(ID_B).unwrap()),
        None,
        vec![],
        None,
        DecisionOutcome::PursuitCompletion,
        None,
        ContextSnapshotId::parse(ID_C).unwrap(),
        Timestamp::new("2026-08-30T12:00:00.000Z").unwrap(),
    )
    .unwrap();
    assert_eq!(
        user_completion.decision_kind(),
        &DecisionKind::PursuitCompletion
    );

    let invalid = DecisionRecord::new(
        DecisionRecordId::parse(ID_A).unwrap(),
        DecisionOrigin::SystemRecommended,
        DecisionKind::PursuitCompletion,
        TypedRef::Pursuit(PursuitId::parse(ID_B).unwrap()),
        Some(RecommendationRef::Recommendation(
            RecommendationId::parse(ID_C).unwrap(),
        )),
        vec![],
        None,
        DecisionOutcome::PursuitCompletion,
        None,
        ContextSnapshotId::parse(ID_C).unwrap(),
        Timestamp::new("2026-08-30T12:00:00.000Z").unwrap(),
    );
    assert_eq!(
        invalid.unwrap_err(),
        DecisionViolation::PursuitCompletionMustBeUserInitiated
    );

    let wrong_subject = DecisionRecord::new(
        DecisionRecordId::parse(ID_A).unwrap(),
        DecisionOrigin::UserInitiated,
        DecisionKind::PursuitCompletion,
        TypedRef::Session(SessionId::parse(ID_B).unwrap()),
        None,
        vec![],
        None,
        DecisionOutcome::PursuitCompletion,
        None,
        ContextSnapshotId::parse(ID_C).unwrap(),
        Timestamp::new("2026-08-30T12:00:00.000Z").unwrap(),
    );
    assert_eq!(
        wrong_subject.unwrap_err(),
        DecisionViolation::PursuitCompletionRequiresPursuitSubject
    );
}

#[test]
fn pursuit_completion_and_goal_adjustment_are_distinct_transition_paths() {
    let completion = DecisionRecord::new(
        DecisionRecordId::parse(ID_A).unwrap(),
        DecisionOrigin::UserInitiated,
        DecisionKind::PursuitCompletion,
        TypedRef::Pursuit(PursuitId::parse(ID_B).unwrap()),
        None,
        vec![],
        None,
        DecisionOutcome::PursuitCompletion,
        None,
        ContextSnapshotId::parse(ID_C).unwrap(),
        Timestamp::new("2026-08-30T12:00:00.000Z").unwrap(),
    )
    .unwrap();
    let pause = DecisionRecord::new(
        DecisionRecordId::parse(ID_C).unwrap(),
        DecisionOrigin::UserInitiated,
        DecisionKind::PursuitStatusChange,
        TypedRef::Pursuit(PursuitId::parse(ID_B).unwrap()),
        None,
        vec![],
        None,
        DecisionOutcome::PursuitStatusChange(PursuitAction::Pause),
        None,
        ContextSnapshotId::parse(ID_C).unwrap(),
        Timestamp::new("2026-08-30T12:00:00.000Z").unwrap(),
    )
    .unwrap();

    assert_eq!(
        check_pursuit_status_transition(PursuitStatus::Completed, &completion),
        Ok(())
    );
    assert_eq!(
        check_pursuit_status_transition(PursuitStatus::Paused, &pause),
        Ok(())
    );
    assert!(check_pursuit_status_transition(PursuitStatus::Completed, &pause).is_err());
}

#[test]
fn pursuit_status_is_mutated_only_by_a_matching_decision_backed_transition() {
    let pursuit_id = PursuitId::parse(ID_B).unwrap();
    let created_at = Timestamp::new("2026-08-30T10:00:00.000Z").unwrap();
    let mut pursuit = Pursuit::new(
        pursuit_id.clone(),
        PursuitKind::Outcome,
        "Ship the contract".into(),
        None,
        None,
        None,
        created_at.clone(),
        created_at,
    );
    let completion = DecisionRecord::new(
        DecisionRecordId::parse(ID_A).unwrap(),
        DecisionOrigin::UserInitiated,
        DecisionKind::PursuitCompletion,
        TypedRef::Pursuit(pursuit_id),
        None,
        vec![],
        None,
        DecisionOutcome::PursuitCompletion,
        None,
        ContextSnapshotId::parse(ID_C).unwrap(),
        Timestamp::new("2026-08-30T12:00:00.000Z").unwrap(),
    )
    .unwrap();

    assert_eq!(pursuit.status(), PursuitStatus::Active);
    pursuit
        .transition_status(
            PursuitStatus::Completed,
            Timestamp::new("2026-08-30T12:00:00.000Z").unwrap(),
            &completion,
        )
        .unwrap();
    assert_eq!(pursuit.status(), PursuitStatus::Completed);
    assert_eq!(pursuit.status_history().len(), 1);
    assert_eq!(
        pursuit.status_history()[0].decision_record_ref,
        *completion.id()
    );

    let other_completion = DecisionRecord::new(
        DecisionRecordId::parse(ID_C).unwrap(),
        DecisionOrigin::UserInitiated,
        DecisionKind::PursuitCompletion,
        TypedRef::Pursuit(PursuitId::parse(ID_A).unwrap()),
        None,
        vec![],
        None,
        DecisionOutcome::PursuitCompletion,
        None,
        ContextSnapshotId::parse(ID_C).unwrap(),
        Timestamp::new("2026-08-30T13:00:00.000Z").unwrap(),
    )
    .unwrap();
    assert_eq!(
        pursuit.transition_status(
            PursuitStatus::Completed,
            Timestamp::new("2026-08-30T13:00:00.000Z").unwrap(),
            &other_completion,
        ),
        Err(PursuitTransitionViolation::SubjectMismatch)
    );
}

#[test]
fn observation_derivation_is_required_if_and_only_if_provenance_is_derived() {
    let common = || {
        (
            ObservationId::parse(ID_A).unwrap(),
            TypedRef::Pursuit(PursuitId::parse(ID_B).unwrap()),
            ObservationType::new("practice.quality").unwrap(),
            LiteralValue::Integer(1),
            Timestamp::new("2026-08-30T12:00:00.000Z").unwrap(),
        )
    };
    let (id, subject, kind, value, time) = common();
    let missing = Observation::new(
        id,
        subject,
        kind,
        value,
        time,
        ObservationProvenance::Derived,
        None,
        None,
        SensitivityClass::Standard,
        None,
    );
    assert_eq!(
        missing.unwrap_err(),
        ObservationViolation::MissingDerivation
    );

    let (id, subject, kind, value, time) = common();
    let unexpected = Observation::new(
        id,
        subject,
        kind,
        value,
        time,
        ObservationProvenance::Manual,
        None,
        Some(DerivationRef {
            derived_by: owner(),
            derivation_version: "v1".into(),
            method: "copy".into(),
            source_observation_refs: vec![],
        }),
        SensitivityClass::Standard,
        None,
    );
    assert_eq!(
        unexpected.unwrap_err(),
        ObservationViolation::UnexpectedDerivation
    );
}

#[test]
fn journal_access_grant_requires_finite_scope_and_explicit_user_decision() {
    let decision = user_pack_decision("journal.access_grant_created");
    let empty = JournalAccessGrant::new(
        JournalAccessGrantId::parse(ID_B).unwrap(),
        PackId::new("learning").unwrap(),
        JournalAccessScope::JournalEntries(vec![]),
        "read one entry".into(),
        Timestamp::new("2026-08-30T12:00:00.000Z").unwrap(),
        &decision,
        None,
    );
    assert_eq!(empty.unwrap_err(), JournalAccessGrantViolation::EmptyScope);
}

#[test]
fn kernel_service_inventory_is_closed_and_complete() {
    assert_eq!(KERNEL_SERVICES.len(), 14);
    assert!(KERNEL_SERVICES.contains(&ServiceId::PermissionService));
}

#[test]
fn ulid_timestamp_and_semver_conventions_are_validated() {
    assert!(Id::new(ID_A).is_ok());
    assert!(Id::new("not-an-id").is_err());
    assert!(Timestamp::new("2026-08-30T12:00:00.000Z").is_ok());
    assert!(Timestamp::new("2026-08-30 12:00:00Z").is_err());
    assert!(SemVer::new("1.2.3").is_ok());
    assert!(SemVer::new("v1").is_err());
}
