use flourish_core::*;
use std::collections::BTreeMap;

const SNAPSHOT_ID: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
const PURSUIT_A: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAA";
const PURSUIT_B: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAB";

fn pack(id: &str) -> CapabilityPack {
    CapabilityPack {
        id: PackId::new(id).unwrap(),
        version: SemVer::new("1.0.0").unwrap(),
        provides: vec![],
        requires: vec![],
        optional_requires: vec![],
        protocols: vec![],
        entity_extensions: vec![],
        publishes: vec![],
        subscribes: vec![],
        surfaces: vec![],
        permissions: vec![],
        content: vec![],
        settings_schema: LiteralValue::Object(BTreeMap::new()),
        enable_semantics: "non-destructive enable".into(),
        disable_semantics: "non-destructive disable".into(),
        export_semantics: "complete export".into(),
        erase_semantics: "explicit erase".into(),
    }
}

fn profile(id: &str) -> DomainProfile {
    DomainProfile::new(
        DomainProfileId::parse(id).unwrap(),
        SemVer::new("1.0.0").unwrap(),
        "Context",
        "Vocabulary and examples only",
        LiteralValue::Text("catalog:test".into()),
        LiteralValue::Object(BTreeMap::new()),
        BTreeMap::new(),
        vec![],
        vec![],
        vec![],
        vec![],
    )
    .unwrap()
}

fn rule(id: &str, need_id: &str, pack_id: &str) -> ComposerRule {
    ComposerRule {
        id: ComposerRuleId::new(id).unwrap(),
        need_id: ComposerNeedId::new(need_id).unwrap(),
        matcher: NeedMatcher::Present,
        pack_id: PackId::new(pack_id).unwrap(),
        reason: format!("confirmed {need_id} selects {pack_id}"),
    }
}

fn need(id: &str) -> ComposerNeed {
    ComposerNeed {
        id: ComposerNeedId::new(id).unwrap(),
        value: LiteralValue::Boolean(true),
        confirmation: NeedConfirmation::Confirmed,
    }
}

fn availability(id: &str, state: CapabilityState) -> CapabilityAvailability {
    CapabilityAvailability {
        pack_id: PackId::new(id).unwrap(),
        state,
    }
}

fn input() -> ComposerInput {
    ComposerInput {
        schema_version: 1,
        manifest_version: 1,
        composer_version: SemVer::new("1.0.0").unwrap(),
        composer_ruleset_version: SemVer::new("1.0.0").unwrap(),
        answers_snapshot_ref: ContextSnapshotId::parse(SNAPSHOT_ID).unwrap(),
        needs: vec![],
        capability_states: vec![],
        pack_choices: vec![],
        domain_profile_refs: vec![],
        user_parameter_overrides: vec![],
        integration_bindings: vec![],
        path_composition_refs: vec![],
        manifest_changelog: vec![],
    }
}

fn ordered_variants<T: Clone>(values: &[T]) -> Vec<Vec<T>> {
    let mut variants = vec![values.to_vec()];
    if values.len() > 1 {
        let mut reversed = values.to_vec();
        reversed.reverse();
        variants.push(reversed);
        for offset in 1..values.len() {
            let mut rotated = values.to_vec();
            rotated.rotate_left(offset);
            variants.push(rotated);
        }
    }
    variants
}

#[test]
fn repeated_runs_and_input_permutations_produce_identical_manifests() {
    let packs = vec![pack("focus"), pack("learning"), pack("planning")];
    let rules = vec![
        rule("rule.focus", "need.focus", "focus"),
        rule("rule.learning", "need.learning", "learning"),
        rule("rule.planning", "need.planning", "planning"),
    ];
    let needs = vec![
        need("need.focus"),
        need("need.learning"),
        need("need.planning"),
    ];
    let states = vec![
        availability("focus", CapabilityState::Available),
        availability("learning", CapabilityState::Available),
        availability("planning", CapabilityState::Available),
    ];
    let catalog = ComposerCatalog {
        packs: packs.clone(),
        domain_profiles: vec![],
        rules: rules.clone(),
    };
    let mut composer_input = input();
    composer_input.needs = needs.clone();
    composer_input.capability_states = states.clone();
    composer_input.path_composition_refs = vec![
        PathCompositionRef::new(PursuitId::parse(PURSUIT_B).unwrap(), 2).unwrap(),
        PathCompositionRef::new(PursuitId::parse(PURSUIT_A).unwrap(), 1).unwrap(),
    ];
    let expected = SystemComposer.compose(&catalog, &composer_input).unwrap();

    for _ in 0..100 {
        assert_eq!(
            SystemComposer.compose(&catalog, &composer_input).unwrap(),
            expected
        );
    }
    for variant in ordered_variants(&packs) {
        let mut permuted = catalog.clone();
        permuted.packs = variant;
        assert_eq!(
            SystemComposer.compose(&permuted, &composer_input).unwrap(),
            expected
        );
    }
    for variant in ordered_variants(&rules) {
        let mut permuted = catalog.clone();
        permuted.rules = variant;
        assert_eq!(
            SystemComposer.compose(&permuted, &composer_input).unwrap(),
            expected
        );
    }
    for variant in ordered_variants(&needs) {
        let mut permuted = composer_input.clone();
        permuted.needs = variant;
        assert_eq!(
            SystemComposer.compose(&catalog, &permuted).unwrap(),
            expected
        );
    }
    for variant in ordered_variants(&states) {
        let mut permuted = composer_input.clone();
        permuted.capability_states = variant;
        assert_eq!(
            SystemComposer.compose(&catalog, &permuted).unwrap(),
            expected
        );
    }

    assert_eq!(
        expected
            .enabled_packs
            .iter()
            .map(|entry| entry.pack_id.as_str())
            .collect::<Vec<_>>(),
        vec!["focus", "learning", "planning"]
    );
    assert_eq!(
        expected
            .path_composition_refs
            .iter()
            .map(|reference| reference.pursuit_ref.to_string())
            .collect::<Vec<_>>(),
        vec![PURSUIT_A, PURSUIT_B]
    );
}

#[test]
fn zero_ai_manual_typed_needs_are_a_complete_composition_path() {
    let catalog = ComposerCatalog {
        packs: vec![pack("learning")],
        domain_profiles: vec![],
        rules: vec![rule("rule.learning", "need.learning", "learning")],
    };
    let mut manual = input();
    manual.needs = vec![need("need.learning")];
    manual.capability_states = vec![availability("learning", CapabilityState::Available)];

    let manifest = SystemComposer.compose(&catalog, &manual).unwrap();

    assert_eq!(manifest.enabled_packs.len(), 1);
    assert!(manifest.path_composition_refs.is_empty());
}

#[test]
fn candidate_needs_never_cross_the_confirmation_boundary() {
    let catalog = ComposerCatalog {
        packs: vec![pack("learning")],
        domain_profiles: vec![],
        rules: vec![rule("rule.learning", "need.learning", "learning")],
    };
    let mut composer_input = input();
    composer_input.needs = vec![ComposerNeed {
        confirmation: NeedConfirmation::Candidate,
        ..need("need.learning")
    }];

    assert_eq!(
        SystemComposer.compose(&catalog, &composer_input),
        Err(ComposerError::UnconfirmedNeed(
            ComposerNeedId::new("need.learning").unwrap()
        ))
    );
}

#[test]
fn capability_degradation_is_explicit_and_fail_closed() {
    let catalog = ComposerCatalog {
        packs: vec![
            pack("available"),
            pack("unavailable"),
            pack("unauthorized"),
            pack("unknown"),
        ],
        domain_profiles: vec![],
        rules: vec![
            rule("rule.available", "need.all", "available"),
            rule("rule.unavailable", "need.all", "unavailable"),
            rule("rule.unauthorized", "need.all", "unauthorized"),
            rule("rule.unknown", "need.all", "unknown"),
        ],
    };
    let mut composer_input = input();
    composer_input.needs = vec![need("need.all")];
    composer_input.capability_states = vec![
        availability("available", CapabilityState::Available),
        availability("unavailable", CapabilityState::Unavailable),
        availability("unauthorized", CapabilityState::Unauthorized),
    ];

    let manifest = SystemComposer.compose(&catalog, &composer_input).unwrap();
    assert_eq!(manifest.enabled_packs[0].pack_id.as_str(), "available");
    assert_eq!(
        manifest
            .omitted_packs
            .iter()
            .map(|entry| entry.pack_id.as_str())
            .collect::<Vec<_>>(),
        vec!["unauthorized", "unavailable", "unknown"]
    );
    assert!(manifest.omitted_packs.iter().all(|entry| entry
        .reason_rule_refs
        .iter()
        .any(|reason| reason.as_str().starts_with("kernel.capability-"))));
}

#[test]
fn hard_dependency_versions_and_user_disable_fail_closed() {
    let mut focus = pack("focus");
    focus.requires = vec![PackDependency {
        pack_id: PackId::new("learning").unwrap(),
        version_requirement: "1.0.0".into(),
        degradation: None,
    }];
    let catalog = ComposerCatalog {
        packs: vec![pack("learning"), focus],
        domain_profiles: vec![],
        rules: vec![rule("rule.focus", "need.focus", "focus")],
    };
    let mut composer_input = input();
    composer_input.needs = vec![need("need.focus")];
    composer_input.capability_states = vec![
        availability("focus", CapabilityState::Available),
        availability("learning", CapabilityState::Available),
    ];
    composer_input.pack_choices = vec![PackChoice {
        pack_id: PackId::new("learning").unwrap(),
        selection: PackSelection::Disable,
    }];

    let manifest = SystemComposer.compose(&catalog, &composer_input).unwrap();
    assert!(manifest.enabled_packs.is_empty());
    assert!(manifest
        .omitted_packs
        .iter()
        .find(|entry| entry.pack_id.as_str() == "focus")
        .unwrap()
        .reason_rule_refs
        .iter()
        .any(|rule_id| rule_id.as_str() == "kernel.required-dependency-blocked"));
}

#[test]
fn context_profile_is_preserved_but_cannot_generate_material() {
    let profile_id = "01ARZ3NDEKTSV4RRFFQ69G5FAC";
    let catalog = ComposerCatalog {
        packs: vec![pack("learning")],
        domain_profiles: vec![profile(profile_id)],
        rules: vec![],
    };
    let mut composer_input = input();
    composer_input.domain_profile_refs = vec![DomainProfileId::parse(profile_id).unwrap()];
    composer_input.capability_states = vec![availability("learning", CapabilityState::Available)];

    let manifest = SystemComposer.compose(&catalog, &composer_input).unwrap();

    assert!(manifest.enabled_packs.is_empty());
    assert!(manifest.protocol_selections.is_empty());
    assert!(manifest.path_composition_refs.is_empty());
    assert_eq!(manifest.domain_profiles.len(), 1);
}

#[test]
fn duplicate_path_references_are_rejected_instead_of_resolved_by_ranking() {
    let catalog = ComposerCatalog {
        packs: vec![],
        domain_profiles: vec![],
        rules: vec![],
    };
    let pursuit = PursuitId::parse(PURSUIT_A).unwrap();
    let mut composer_input = input();
    composer_input.path_composition_refs = vec![
        PathCompositionRef::new(pursuit.clone(), 1).unwrap(),
        PathCompositionRef::new(pursuit.clone(), 2).unwrap(),
    ];

    assert_eq!(
        SystemComposer.compose(&catalog, &composer_input),
        Err(ComposerError::DuplicatePathPursuit(pursuit))
    );

    let mut zero_version = input();
    zero_version.path_composition_refs = vec![PathCompositionRef {
        pursuit_ref: PursuitId::parse(PURSUIT_A).unwrap(),
        path_version: 0,
    }];
    assert_eq!(
        SystemComposer.compose(&catalog, &zero_version),
        Err(ComposerError::InvalidPathVersion(
            PursuitId::parse(PURSUIT_A).unwrap()
        ))
    );
}
