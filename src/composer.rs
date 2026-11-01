//! Pure, deterministic typed-needs composition and manifest projection.
//!
//! This is the approved reusable part of the pre-V2 Composer. It consumes only
//! typed, user-confirmed needs and local catalog state. AI, free-text
//! interpretation, template import, and Path activation remain outside this
//! no-I/O boundary.

use crate::{
    CapabilityPack, ComposerNeedId, ComposerRuleId, ContextSnapshotId, DomainProfile,
    DomainProfileId, EnabledPack, IntegrationBindingRef, LiteralValue, ManifestChange, OmittedPack,
    PackId, PathCompositionRef, PursuitId, SelectedDomainProfile, SemVer, SystemManifest,
    UserParameterOverride,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

const RULE_EXPLICIT_ENABLE: &str = "kernel.explicit-enable";
const RULE_REQUIRED_DEPENDENCY: &str = "kernel.required-dependency";
const RULE_USER_DISABLED: &str = "kernel.user-disabled";
const RULE_CAPABILITY_UNAVAILABLE: &str = "kernel.capability-unavailable";
const RULE_CAPABILITY_UNAUTHORIZED: &str = "kernel.capability-unauthorized";
const RULE_CAPABILITY_UNKNOWN: &str = "kernel.capability-unknown";
const RULE_DEPENDENCY_BLOCKED: &str = "kernel.required-dependency-blocked";
const RULE_NOT_SELECTED: &str = "kernel.not-selected";

/// Local capability probe state consumed by deterministic composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityState {
    /// Capability is locally usable.
    Available,
    /// Required capability, hardware, or configuration is unavailable.
    Unavailable,
    /// The user has not authorized the capability.
    Unauthorized,
    /// No confirmed probe result exists.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompositionReason {
    rule_id: ComposerRuleId,
    explanation: String,
}

/// Confirmation boundary for a typed need.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedConfirmation {
    /// Candidate interpretation that has not been accepted by the user.
    Candidate,
    /// Typed need explicitly confirmed by the user.
    Confirmed,
}

/// Typed composition input. Free text is intentionally outside this contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposerNeed {
    /// Stable schema-owned need identifier.
    pub id: ComposerNeedId,
    /// Schema-owned typed value.
    pub value: LiteralValue,
    /// Explicit user-confirmation state.
    pub confirmation: NeedConfirmation,
}

/// Declarative match supported by a composition rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NeedMatcher {
    /// Match whenever the confirmed need is present.
    Present,
    /// Match only an exact typed value.
    Equals(LiteralValue),
}

/// Deterministic rule mapping a confirmed need to a Capability Pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposerRule {
    /// Stable rule identifier recorded in the manifest.
    pub id: ComposerRuleId,
    /// Typed need read by the rule.
    pub need_id: ComposerNeedId,
    /// Pure match operation.
    pub matcher: NeedMatcher,
    /// Pack selected when the matcher succeeds.
    pub pack_id: PackId,
    /// User-readable reason recorded with the rule identifier.
    pub reason: String,
}

/// Explicit user choice for one Capability Pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackSelection {
    /// Let confirmed typed needs select the pack.
    Automatic,
    /// Explicitly enable the pack, subject to availability and dependencies.
    Enable,
    /// Explicitly disable the pack non-destructively.
    Disable,
}

/// User selection associated with a known pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackChoice {
    /// Pack receiving the choice.
    pub pack_id: PackId,
    /// Explicit selection state.
    pub selection: PackSelection,
}

/// Probe result for a known Capability Pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityAvailability {
    /// Pack whose local capability was probed.
    pub pack_id: PackId,
    /// Explicit probe state.
    pub state: CapabilityState,
}

/// Versioned, declarative catalog consumed by the deterministic Composer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposerCatalog {
    /// In-tree Capability Pack versions.
    pub packs: Vec<CapabilityPack>,
    /// Context-only Domain Profile versions. Profiles never select Path material.
    pub domain_profiles: Vec<DomainProfile>,
    /// Rules in the pinned ruleset.
    pub rules: Vec<ComposerRule>,
}

/// Complete typed input to one deterministic manifest projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposerInput {
    /// Manifest schema version to emit.
    pub schema_version: u64,
    /// Per-user manifest version to emit.
    pub manifest_version: u64,
    /// Composer implementation version.
    pub composer_version: SemVer,
    /// Pinned deterministic ruleset version.
    pub composer_ruleset_version: SemVer,
    /// Durable reference to the confirmed answer snapshot.
    pub answers_snapshot_ref: ContextSnapshotId,
    /// Typed needs; every entry must be user-confirmed.
    pub needs: Vec<ComposerNeed>,
    /// Local capability states. Missing entries resolve as `Unknown`.
    pub capability_states: Vec<CapabilityAvailability>,
    /// Explicit pack choices. Missing entries resolve as `Automatic`.
    pub pack_choices: Vec<PackChoice>,
    /// User-selected context-only Domain Profiles.
    pub domain_profile_refs: Vec<DomainProfileId>,
    /// User parameter overrides preserved in the manifest.
    pub user_parameter_overrides: Vec<UserParameterOverride>,
    /// Integration references preserved without secrets.
    pub integration_bindings: Vec<IntegrationBindingRef>,
    /// Current accepted Path version references preserved in the manifest projection.
    pub path_composition_refs: Vec<PathCompositionRef>,
    /// Existing auditable change history supplied by the persistence boundary.
    pub manifest_changelog: Vec<ManifestChange>,
}

/// Invalid catalog or composition input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposerError {
    /// A catalog contains the same pack identifier more than once.
    DuplicatePack(PackId),
    /// A catalog contains the same profile identifier more than once.
    DuplicateDomainProfile(DomainProfileId),
    /// A ruleset contains the same rule identifier more than once.
    DuplicateRule(ComposerRuleId),
    /// A rule has no user-readable explanation.
    EmptyRuleReason(ComposerRuleId),
    /// Input contains the same typed need more than once.
    DuplicateNeed(ComposerNeedId),
    /// Input contains more than one capability state for a pack.
    DuplicateCapabilityState(PackId),
    /// Input contains more than one explicit choice for a pack.
    DuplicatePackChoice(PackId),
    /// A candidate need crossed the composition boundary without confirmation.
    UnconfirmedNeed(ComposerNeedId),
    /// An input or catalog reference points to an unknown pack.
    UnknownPack(PackId),
    /// A selected profile is not in the catalog.
    UnknownDomainProfile(DomainProfileId),
    /// More than one accepted Path version was supplied for a Pursuit.
    DuplicatePathPursuit(PursuitId),
    /// A Path manifest reference used the reserved zero version.
    InvalidPathVersion(PursuitId),
}

impl Display for ComposerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicatePack(id) => write!(formatter, "duplicate Capability Pack {id}"),
            Self::DuplicateDomainProfile(id) => write!(formatter, "duplicate Domain Profile {id}"),
            Self::DuplicateRule(id) => write!(formatter, "duplicate Composer rule {id}"),
            Self::EmptyRuleReason(id) => {
                write!(formatter, "Composer rule {id} has an empty reason")
            }
            Self::DuplicateNeed(id) => write!(formatter, "duplicate Composer need {id}"),
            Self::DuplicateCapabilityState(id) => {
                write!(formatter, "duplicate capability state for pack {id}")
            }
            Self::DuplicatePackChoice(id) => write!(formatter, "duplicate choice for pack {id}"),
            Self::UnconfirmedNeed(id) => {
                write!(formatter, "Composer need {id} is not user-confirmed")
            }
            Self::UnknownPack(id) => write!(formatter, "unknown Capability Pack {id}"),
            Self::UnknownDomainProfile(id) => write!(formatter, "unknown Domain Profile {id}"),
            Self::DuplicatePathPursuit(id) => {
                write!(
                    formatter,
                    "duplicate Path version reference for Pursuit {id}"
                )
            }
            Self::InvalidPathVersion(id) => {
                write!(
                    formatter,
                    "Path reference for Pursuit {id} has version zero"
                )
            }
        }
    }
}

impl Error for ComposerError {}

/// Stateless deterministic Composer.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemComposer;

impl SystemComposer {
    /// Resolves confirmed inputs into a canonically ordered System Manifest.
    pub fn compose(
        &self,
        catalog: &ComposerCatalog,
        input: &ComposerInput,
    ) -> Result<SystemManifest, ComposerError> {
        let packs = unique_map(
            &catalog.packs,
            |pack| pack.id.clone(),
            ComposerError::DuplicatePack,
        )?;
        let profiles = unique_map(
            &catalog.domain_profiles,
            |profile| profile.id.clone(),
            ComposerError::DuplicateDomainProfile,
        )?;
        validate_catalog_references(&packs)?;
        validate_rules(&catalog.rules, &packs)?;
        let needs = confirmed_needs(&input.needs)?;
        let states = capability_states(&input.capability_states, &packs)?;
        let choices = pack_choices(&input.pack_choices, &packs)?;
        let selected_profiles = selected_profiles(&input.domain_profile_refs, &profiles)?;
        let path_composition_refs = path_references(&input.path_composition_refs)?;

        let mut selected: BTreeMap<PackId, Vec<CompositionReason>> = BTreeMap::new();
        let mut rules = catalog.rules.iter().collect::<Vec<_>>();
        rules.sort_by(|left, right| left.id.cmp(&right.id));
        for rule in rules {
            if needs
                .get(&rule.need_id)
                .is_some_and(|value| rule.matcher.matches(value))
            {
                selected
                    .entry(rule.pack_id.clone())
                    .or_default()
                    .push(CompositionReason {
                        rule_id: rule.id.clone(),
                        explanation: rule.reason.clone(),
                    });
            }
        }

        for (pack_id, selection) in &choices {
            if *selection == PackSelection::Enable {
                selected.entry(pack_id.clone()).or_default().push(reason(
                    RULE_EXPLICIT_ENABLE,
                    format!("{pack_id} was explicitly enabled by the user"),
                ));
            }
        }

        // Context-only profiles are preserved as selected references. They never
        // create, order, select, or activate packs, Protocols, or Path material.
        add_required_dependencies(&packs, &choices, &mut selected)?;

        let candidate_ids = selected
            .keys()
            .filter(|pack_id| {
                choices.get(*pack_id) != Some(&PackSelection::Disable)
                    && state_for(pack_id, &states) == CapabilityState::Available
            })
            .cloned()
            .collect::<BTreeSet<_>>();
        let enabled_ids = dependency_closed_enabled_set(&packs, candidate_ids);

        let mut enabled_packs = Vec::new();
        let mut omitted_packs = Vec::new();
        for (pack_id, pack) in &packs {
            let state = state_for(pack_id, &states);
            if enabled_ids.contains(pack_id) {
                let reasons = canonical_reasons(selected.remove(pack_id).unwrap_or_default());
                enabled_packs.push(EnabledPack {
                    pack_id: pack_id.clone(),
                    version: pack.version.clone(),
                    reason_rule_refs: reason_refs(&reasons),
                });
            } else {
                let mut reasons = selected.remove(pack_id).unwrap_or_default();
                if choices.get(pack_id) == Some(&PackSelection::Disable) {
                    reasons.push(reason(
                        RULE_USER_DISABLED,
                        format!("{pack_id} was explicitly disabled by the user"),
                    ));
                } else {
                    match state {
                        CapabilityState::Unavailable => reasons.push(reason(
                            RULE_CAPABILITY_UNAVAILABLE,
                            format!("{pack_id} is unavailable on this system"),
                        )),
                        CapabilityState::Unauthorized => reasons.push(reason(
                            RULE_CAPABILITY_UNAUTHORIZED,
                            format!("{pack_id} has not been authorized by the user"),
                        )),
                        CapabilityState::Unknown => reasons.push(reason(
                            RULE_CAPABILITY_UNKNOWN,
                            format!("{pack_id} has no confirmed capability state"),
                        )),
                        CapabilityState::Available if reasons.is_empty() => reasons.push(reason(
                            RULE_NOT_SELECTED,
                            format!(
                                "{pack_id} was not selected by any confirmed need or user choice"
                            ),
                        )),
                        CapabilityState::Available => {
                            add_blocked_dependency_reasons(
                                pack,
                                &packs,
                                &enabled_ids,
                                &mut reasons,
                            );
                        }
                    }
                }
                let reasons = canonical_reasons(reasons);
                omitted_packs.push(OmittedPack {
                    pack_id: pack_id.clone(),
                    reason_rule_refs: reason_refs(&reasons),
                });
            }
        }

        let domain_profiles = selected_profiles
            .iter()
            .map(|profile| SelectedDomainProfile {
                profile_id: profile.id.clone(),
                version: profile.version.clone(),
            })
            .collect();

        let mut user_parameter_overrides = input.user_parameter_overrides.clone();
        user_parameter_overrides.sort_by(|left, right| {
            (
                &left.protocol_id,
                &left.protocol_version,
                &left.parameter_key,
            )
                .cmp(&(
                    &right.protocol_id,
                    &right.protocol_version,
                    &right.parameter_key,
                ))
        });
        let mut integration_bindings = input.integration_bindings.clone();
        integration_bindings.sort();
        integration_bindings.dedup();

        Ok(SystemManifest {
            schema_version: input.schema_version,
            manifest_version: input.manifest_version,
            composer_version: input.composer_version.clone(),
            composer_ruleset_version: input.composer_ruleset_version.clone(),
            answers_snapshot_ref: input.answers_snapshot_ref.clone(),
            enabled_packs,
            omitted_packs,
            domain_profiles,
            protocol_selections: vec![],
            user_parameter_overrides,
            integration_bindings,
            path_composition_refs,
            manifest_changelog: input.manifest_changelog.clone(),
        })
    }
}

impl NeedMatcher {
    fn matches(&self, value: &LiteralValue) -> bool {
        match self {
            Self::Present => true,
            Self::Equals(expected) => expected == value,
        }
    }
}

fn unique_map<T, K, F, E>(
    values: &[T],
    key: F,
    duplicate_error: E,
) -> Result<BTreeMap<K, &T>, ComposerError>
where
    K: Ord + Clone,
    F: Fn(&T) -> K,
    E: Fn(K) -> ComposerError,
{
    let mut result = BTreeMap::new();
    for value in values {
        let value_key = key(value);
        if result.insert(value_key.clone(), value).is_some() {
            return Err(duplicate_error(value_key));
        }
    }
    Ok(result)
}

fn validate_rules(
    rules: &[ComposerRule],
    packs: &BTreeMap<PackId, &CapabilityPack>,
) -> Result<(), ComposerError> {
    let mut ids = BTreeSet::new();
    for rule in rules {
        if !ids.insert(rule.id.clone()) {
            return Err(ComposerError::DuplicateRule(rule.id.clone()));
        }
        if rule.reason.trim().is_empty() {
            return Err(ComposerError::EmptyRuleReason(rule.id.clone()));
        }
        if !packs.contains_key(&rule.pack_id) {
            return Err(ComposerError::UnknownPack(rule.pack_id.clone()));
        }
    }
    Ok(())
}

fn validate_catalog_references(
    packs: &BTreeMap<PackId, &CapabilityPack>,
) -> Result<(), ComposerError> {
    for pack in packs.values() {
        for dependency in pack.requires.iter().chain(&pack.optional_requires) {
            if !packs.contains_key(&dependency.pack_id) {
                return Err(ComposerError::UnknownPack(dependency.pack_id.clone()));
            }
        }
    }
    Ok(())
}

fn confirmed_needs(
    needs: &[ComposerNeed],
) -> Result<BTreeMap<ComposerNeedId, LiteralValue>, ComposerError> {
    let mut result = BTreeMap::new();
    for need in needs {
        if need.confirmation != NeedConfirmation::Confirmed {
            return Err(ComposerError::UnconfirmedNeed(need.id.clone()));
        }
        if result.insert(need.id.clone(), need.value.clone()).is_some() {
            return Err(ComposerError::DuplicateNeed(need.id.clone()));
        }
    }
    Ok(result)
}

fn capability_states(
    states: &[CapabilityAvailability],
    packs: &BTreeMap<PackId, &CapabilityPack>,
) -> Result<BTreeMap<PackId, CapabilityState>, ComposerError> {
    let mut result = BTreeMap::new();
    for availability in states {
        if !packs.contains_key(&availability.pack_id) {
            return Err(ComposerError::UnknownPack(availability.pack_id.clone()));
        }
        if result
            .insert(availability.pack_id.clone(), availability.state)
            .is_some()
        {
            return Err(ComposerError::DuplicateCapabilityState(
                availability.pack_id.clone(),
            ));
        }
    }
    Ok(result)
}

fn pack_choices(
    choices: &[PackChoice],
    packs: &BTreeMap<PackId, &CapabilityPack>,
) -> Result<BTreeMap<PackId, PackSelection>, ComposerError> {
    let mut result = BTreeMap::new();
    for choice in choices {
        if !packs.contains_key(&choice.pack_id) {
            return Err(ComposerError::UnknownPack(choice.pack_id.clone()));
        }
        if result
            .insert(choice.pack_id.clone(), choice.selection)
            .is_some()
        {
            return Err(ComposerError::DuplicatePackChoice(choice.pack_id.clone()));
        }
    }
    Ok(result)
}

fn selected_profiles<'a>(
    selected: &[DomainProfileId],
    profiles: &BTreeMap<DomainProfileId, &'a DomainProfile>,
) -> Result<Vec<&'a DomainProfile>, ComposerError> {
    let mut ids = selected.to_vec();
    ids.sort();
    ids.dedup();
    ids.into_iter()
        .map(|id| {
            profiles
                .get(&id)
                .copied()
                .ok_or(ComposerError::UnknownDomainProfile(id))
        })
        .collect()
}

fn path_references(
    references: &[PathCompositionRef],
) -> Result<Vec<PathCompositionRef>, ComposerError> {
    let mut result = references.to_vec();
    if let Some(invalid) = result
        .iter()
        .find(|reference| reference.validate().is_err())
    {
        return Err(ComposerError::InvalidPathVersion(
            invalid.pursuit_ref.clone(),
        ));
    }
    result.sort();
    if let Some(duplicate) = result
        .windows(2)
        .find(|window| window[0].pursuit_ref == window[1].pursuit_ref)
    {
        return Err(ComposerError::DuplicatePathPursuit(
            duplicate[0].pursuit_ref.clone(),
        ));
    }
    Ok(result)
}

fn add_required_dependencies(
    packs: &BTreeMap<PackId, &CapabilityPack>,
    choices: &BTreeMap<PackId, PackSelection>,
    selected: &mut BTreeMap<PackId, Vec<CompositionReason>>,
) -> Result<(), ComposerError> {
    loop {
        let before = selected.len();
        let selected_ids = selected.keys().cloned().collect::<Vec<_>>();
        for pack_id in selected_ids {
            let pack = packs
                .get(&pack_id)
                .ok_or_else(|| ComposerError::UnknownPack(pack_id.clone()))?;
            let mut dependencies = pack.requires.iter().collect::<Vec<_>>();
            dependencies.sort_by(|left, right| left.pack_id.cmp(&right.pack_id));
            for dependency in dependencies {
                let dependency_pack = packs
                    .get(&dependency.pack_id)
                    .ok_or_else(|| ComposerError::UnknownPack(dependency.pack_id.clone()))?;
                if dependency.version_requirement != dependency_pack.version.as_str() {
                    continue;
                }
                if choices.get(&dependency.pack_id) != Some(&PackSelection::Disable) {
                    selected
                        .entry(dependency.pack_id.clone())
                        .or_default()
                        .push(reason(
                            RULE_REQUIRED_DEPENDENCY,
                            format!(
                                "{pack_id} requires {} {}",
                                dependency.pack_id, dependency.version_requirement
                            ),
                        ));
                }
            }
        }
        if selected.len() == before {
            return Ok(());
        }
    }
}

fn dependency_closed_enabled_set(
    packs: &BTreeMap<PackId, &CapabilityPack>,
    mut enabled: BTreeSet<PackId>,
) -> BTreeSet<PackId> {
    loop {
        let blocked = enabled
            .iter()
            .filter(|pack_id| {
                packs.get(*pack_id).is_none_or(|pack| {
                    pack.requires.iter().any(|dependency| {
                        packs
                            .get(&dependency.pack_id)
                            .is_none_or(|dependency_pack| {
                                dependency.version_requirement != dependency_pack.version.as_str()
                                    || !enabled.contains(&dependency.pack_id)
                            })
                    })
                })
            })
            .cloned()
            .collect::<Vec<_>>();
        if blocked.is_empty() {
            return enabled;
        }
        for pack_id in blocked {
            enabled.remove(&pack_id);
        }
    }
}

fn add_blocked_dependency_reasons(
    pack: &CapabilityPack,
    packs: &BTreeMap<PackId, &CapabilityPack>,
    enabled: &BTreeSet<PackId>,
    reasons: &mut Vec<CompositionReason>,
) {
    let mut blocked = pack
        .requires
        .iter()
        .filter(|dependency| {
            packs
                .get(&dependency.pack_id)
                .is_none_or(|dependency_pack| {
                    dependency.version_requirement != dependency_pack.version.as_str()
                        || !enabled.contains(&dependency.pack_id)
                })
        })
        .collect::<Vec<_>>();
    blocked.sort_by(|left, right| left.pack_id.cmp(&right.pack_id));
    for dependency in blocked {
        reasons.push(reason(
            RULE_DEPENDENCY_BLOCKED,
            format!(
                "{} requires unavailable or incompatible dependency {} {}",
                pack.id, dependency.pack_id, dependency.version_requirement
            ),
        ));
    }
}

fn state_for(pack_id: &PackId, states: &BTreeMap<PackId, CapabilityState>) -> CapabilityState {
    states
        .get(pack_id)
        .copied()
        .unwrap_or(CapabilityState::Unknown)
}

fn reason(rule_id: &str, explanation: String) -> CompositionReason {
    CompositionReason {
        rule_id: ComposerRuleId::new(rule_id).expect("kernel Composer rule IDs are non-empty"),
        explanation,
    }
}

fn canonical_reasons(mut reasons: Vec<CompositionReason>) -> Vec<CompositionReason> {
    reasons.sort_by(|left, right| {
        (&left.rule_id, &left.explanation).cmp(&(&right.rule_id, &right.explanation))
    });
    reasons.dedup();
    reasons
}

fn reason_refs(reasons: &[CompositionReason]) -> Vec<ComposerRuleId> {
    let mut refs = reasons
        .iter()
        .map(|reason| reason.rule_id.clone())
        .collect::<Vec<_>>();
    refs.sort();
    refs.dedup();
    refs
}
