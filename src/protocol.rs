//! Protocol, pack, profile, manifest, event, and derived-evidence contracts.

use crate::{
    check_invariant_eligibility, Attempt, ComposerRuleId, ContentRegistryId, ContextFactRegistry,
    ContextSnapshotId, DomainProfileId, EventId, EventTopic, EventTopicRegistry,
    IntegrationBindingRef, LiteralValue, Observation, OpaqueRef, OwningRef, PackId,
    PathCompositionRef, Predicate, ProtocolId, ReflectionOutcome, SemVer, StrategyId, Timestamp,
    TypedRef, VersionedIntegrationCapabilityRef, VersionedMethodRef,
};
use crate::{DecisionRecord, FactCategory, InvariantViolation, ServiceId};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Protocol enforcement taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnforcementClass {
    /// Default recommendation with alternatives and deferral.
    Advisory,
    /// Exceptional, justified and scoped guardrail.
    Guardrail,
    /// Technical/data-integrity invariant only.
    Invariant,
}

/// Declarative Protocol parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolParameter {
    /// Stable parameter key.
    pub key: String,
    /// Default value owned by this Protocol Version.
    pub default: LiteralValue,
    /// Optional accepted values/range contract.
    pub constraints: Option<LiteralValue>,
    /// Provenance label for the default.
    pub provenance: String,
}

/// Recommendation policy represented without executable callbacks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecommendationPolicy {
    /// Declarative policy conditions.
    pub conditions: Vec<Predicate>,
    /// Whether a normal `not_now`/defer path exists.
    pub allows_not_now: bool,
}

/// Protocol alternative.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolAlternative {
    /// Stable alternative key within the Protocol Version.
    pub key: String,
    /// User-readable label.
    pub label: String,
    /// Typed alternative payload.
    pub value: LiteralValue,
}

/// Calibration policy shape for a Protocol parameter set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalibrationPolicy {
    /// Whether personal calibration is supported.
    pub enabled: bool,
    /// Observation types eligible as a personal basis.
    pub observation_types: Vec<crate::ObservationType>,
    /// User-readable reversal semantics.
    pub reversal_semantics: String,
}

/// Invalid Protocol construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolViolation {
    /// An `INVARIANT` attempted to expose deferral.
    InvariantAllowsNotNow,
    /// An `INVARIANT` predicate referenced an ineligible or unresolved definition.
    InvariantEligibility(InvariantViolation),
}

impl Display for ProtocolViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvariantAllowsNotNow => {
                formatter.write_str("INVARIANT recommendation policy must not allow not_now/defer")
            }
            Self::InvariantEligibility(violation) => Display::fmt(violation, formatter),
        }
    }
}

impl Error for ProtocolViolation {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvariantEligibility(violation) => Some(violation),
            Self::InvariantAllowsNotNow => None,
        }
    }
}

impl From<InvariantViolation> for ProtocolViolation {
    fn from(value: InvariantViolation) -> Self {
        Self::InvariantEligibility(value)
    }
}

/// Immutable versioned policy definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Protocol {
    id: ProtocolId,
    version: SemVer,
    owning_capability: PackId,
    enforcement_class: EnforcementClass,
    purpose: String,
    applies_to: Vec<Predicate>,
    triggers: Vec<Predicate>,
    parameters: Vec<ProtocolParameter>,
    rationale_refs: Vec<OpaqueRef>,
    recommendation_policy: RecommendationPolicy,
    alternatives: Vec<ProtocolAlternative>,
    observation_schema: Vec<crate::ObservationType>,
    calibration_policy: CalibrationPolicy,
    supersedes: Option<(ProtocolId, SemVer)>,
    disabled_semantics: String,
    method_ref: Option<VersionedMethodRef>,
}

impl Protocol {
    /// Constructs a Protocol Version and mechanically validates every
    /// `INVARIANT` predicate family through the registries.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ProtocolId,
        version: SemVer,
        owning_capability: PackId,
        enforcement_class: EnforcementClass,
        purpose: String,
        applies_to: Vec<Predicate>,
        triggers: Vec<Predicate>,
        parameters: Vec<ProtocolParameter>,
        rationale_refs: Vec<OpaqueRef>,
        recommendation_policy: RecommendationPolicy,
        alternatives: Vec<ProtocolAlternative>,
        observation_schema: Vec<crate::ObservationType>,
        calibration_policy: CalibrationPolicy,
        supersedes: Option<(ProtocolId, SemVer)>,
        disabled_semantics: String,
        fact_registry: &dyn ContextFactRegistry,
        topic_registry: &dyn EventTopicRegistry,
    ) -> Result<Self, ProtocolViolation> {
        Self::new_with_method(
            id,
            version,
            owning_capability,
            enforcement_class,
            purpose,
            applies_to,
            triggers,
            parameters,
            rationale_refs,
            recommendation_policy,
            alternatives,
            observation_schema,
            calibration_policy,
            supersedes,
            disabled_semantics,
            None,
            fact_registry,
            topic_registry,
        )
    }

    /// Constructs a Protocol Version linked to its optional catalog Method.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_method(
        id: ProtocolId,
        version: SemVer,
        owning_capability: PackId,
        enforcement_class: EnforcementClass,
        purpose: String,
        applies_to: Vec<Predicate>,
        triggers: Vec<Predicate>,
        parameters: Vec<ProtocolParameter>,
        rationale_refs: Vec<OpaqueRef>,
        recommendation_policy: RecommendationPolicy,
        alternatives: Vec<ProtocolAlternative>,
        observation_schema: Vec<crate::ObservationType>,
        calibration_policy: CalibrationPolicy,
        supersedes: Option<(ProtocolId, SemVer)>,
        disabled_semantics: String,
        method_ref: Option<VersionedMethodRef>,
        fact_registry: &dyn ContextFactRegistry,
        topic_registry: &dyn EventTopicRegistry,
    ) -> Result<Self, ProtocolViolation> {
        if enforcement_class == EnforcementClass::Invariant {
            if recommendation_policy.allows_not_now {
                return Err(ProtocolViolation::InvariantAllowsNotNow);
            }
            for predicate in applies_to
                .iter()
                .chain(triggers.iter())
                .chain(recommendation_policy.conditions.iter())
            {
                check_invariant_eligibility(predicate, fact_registry, topic_registry)?;
            }
        }
        Ok(Self {
            id,
            version,
            owning_capability,
            enforcement_class,
            purpose,
            applies_to,
            triggers,
            parameters,
            rationale_refs,
            recommendation_policy,
            alternatives,
            observation_schema,
            calibration_policy,
            supersedes,
            disabled_semantics,
            method_ref,
        })
    }

    /// Stable Protocol identifier.
    pub fn id(&self) -> &ProtocolId {
        &self.id
    }

    /// Immutable Protocol version.
    pub fn version(&self) -> &SemVer {
        &self.version
    }

    /// Owning Capability Pack.
    pub fn owning_capability(&self) -> &PackId {
        &self.owning_capability
    }

    /// Enforcement class.
    pub const fn enforcement_class(&self) -> EnforcementClass {
        self.enforcement_class
    }

    /// User-readable purpose.
    pub fn purpose(&self) -> &str {
        &self.purpose
    }

    /// Applicability predicates.
    pub fn applies_to(&self) -> &[Predicate] {
        &self.applies_to
    }

    /// Trigger predicates.
    pub fn triggers(&self) -> &[Predicate] {
        &self.triggers
    }

    /// Protocol-owned parameters.
    pub fn parameters(&self) -> &[ProtocolParameter] {
        &self.parameters
    }

    /// Evidence/design rationale references.
    pub fn rationale_refs(&self) -> &[OpaqueRef] {
        &self.rationale_refs
    }

    /// Declarative recommendation policy.
    pub fn recommendation_policy(&self) -> &RecommendationPolicy {
        &self.recommendation_policy
    }

    /// Meaningful alternatives.
    pub fn alternatives(&self) -> &[ProtocolAlternative] {
        &self.alternatives
    }

    /// Observation types emitted by the Protocol.
    pub fn observation_schema(&self) -> &[crate::ObservationType] {
        &self.observation_schema
    }

    /// Personal calibration policy.
    pub fn calibration_policy(&self) -> &CalibrationPolicy {
        &self.calibration_policy
    }

    /// Superseded immutable version, when applicable.
    pub fn supersedes(&self) -> Option<&(ProtocolId, SemVer)> {
        self.supersedes.as_ref()
    }

    /// Exact behavior when disabled.
    pub fn disabled_semantics(&self) -> &str {
        &self.disabled_semantics
    }

    /// Catalog Method implemented by this Protocol, if it is a human-development Method.
    pub fn method_ref(&self) -> Option<&VersionedMethodRef> {
        self.method_ref.as_ref()
    }
}

/// One capability or interface provided by a pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackProvide {
    /// Stable strategy implementation.
    LearningStrategy(StrategyId),
    /// Other namespaced capability contract.
    Capability(String),
}

/// Dependency on another Capability Pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackDependency {
    /// Required pack.
    pub pack_id: PackId,
    /// Version requirement encoded by the pack registry.
    pub version_requirement: String,
    /// Typed fallback when this is an optional dependency.
    pub degradation: Option<String>,
}

/// In-tree, versioned Capability Pack manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityPack {
    /// Stable pack identifier.
    pub id: PackId,
    /// Semantic version.
    pub version: SemVer,
    /// Provided capability contracts.
    pub provides: Vec<PackProvide>,
    /// Hard dependencies resolved at composition time.
    pub requires: Vec<PackDependency>,
    /// Optional dependencies with typed fallback.
    pub optional_requires: Vec<PackDependency>,
    /// Immutable Protocol Versions shipped by the pack.
    pub protocols: Vec<(ProtocolId, SemVer)>,
    /// Namespaced entity extension identifiers.
    pub entity_extensions: Vec<String>,
    /// Published Event Topics.
    pub publishes: Vec<EventTopic>,
    /// Subscribed Event Topics and Context Fact identifiers.
    pub subscribes: Vec<String>,
    /// Product surfaces contributed by the pack.
    pub surfaces: Vec<String>,
    /// Explicit permissions.
    pub permissions: Vec<String>,
    /// Kernel Content Registry references.
    pub content: Vec<ContentRegistryId>,
    /// Declarative settings schema.
    pub settings_schema: LiteralValue,
    /// Non-destructive enable behavior.
    pub enable_semantics: String,
    /// Non-destructive disable behavior.
    pub disable_semantics: String,
    /// Complete export behavior.
    pub export_semantics: String,
    /// Explicit erase behavior.
    pub erase_semantics: String,
}

/// Selection of one immutable Protocol Version in a Domain Profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolSelection {
    /// Selected Protocol.
    pub protocol_id: ProtocolId,
    /// Selected immutable version.
    pub protocol_version: SemVer,
    /// Optional pack-provided Learning Strategy.
    pub strategy_id: Option<StrategyId>,
    /// Declarative parameter overrides.
    pub parameter_overrides: BTreeMap<String, LiteralValue>,
}

/// Context-only Domain Profile. It cannot select or activate a Personal Path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainProfile {
    pub id: DomainProfileId,
    pub version: SemVer,
    pub name: String,
    pub summary: String,
    pub provenance: LiteralValue,
    context_priors: LiteralValue,
    pub vocabulary_mappings: BTreeMap<String, String>,
    pub example_content_refs: Vec<ContentRegistryId>,
    pub prompt_refs: Vec<ContentRegistryId>,
    pub method_candidate_refs: Vec<VersionedMethodRef>,
    pub integration_candidate_refs: Vec<VersionedIntegrationCapabilityRef>,
}

impl DomainProfile {
    /// Creates a non-binding context profile and rejects generator/curriculum-shaped priors.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: DomainProfileId,
        version: SemVer,
        name: impl Into<String>,
        summary: impl Into<String>,
        provenance: LiteralValue,
        context_priors: LiteralValue,
        vocabulary_mappings: BTreeMap<String, String>,
        example_content_refs: Vec<ContentRegistryId>,
        prompt_refs: Vec<ContentRegistryId>,
        method_candidate_refs: Vec<VersionedMethodRef>,
        integration_candidate_refs: Vec<VersionedIntegrationCapabilityRef>,
    ) -> Result<Self, DomainProfileViolation> {
        let name = name.into();
        let summary = summary.into();
        if name.trim().is_empty() || summary.trim().is_empty() {
            return Err(DomainProfileViolation::MissingDescription);
        }
        if provenance == LiteralValue::Null {
            return Err(DomainProfileViolation::MissingProvenance);
        }
        if !is_allowed_context_priors(&context_priors) {
            return Err(DomainProfileViolation::ForbiddenGeneratorRole);
        }
        if vocabulary_mappings
            .iter()
            .any(|(key, value)| key.trim().is_empty() || value.trim().is_empty())
        {
            return Err(DomainProfileViolation::InvalidVocabulary);
        }
        if has_duplicates(&example_content_refs)
            || has_duplicates(&prompt_refs)
            || has_duplicates(&method_candidate_refs)
            || has_duplicates(&integration_candidate_refs)
        {
            return Err(DomainProfileViolation::DuplicateReference);
        }
        Ok(Self {
            id,
            version,
            name,
            summary,
            provenance,
            context_priors,
            vocabulary_mappings,
            example_content_refs,
            prompt_refs,
            method_candidate_refs,
            integration_candidate_refs,
        })
    }

    /// Non-binding factual/context priors; never executable generation rules.
    pub fn context_priors(&self) -> &LiteralValue {
        &self.context_priors
    }
}

/// Domain Profile compiler violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainProfileViolation {
    MissingDescription,
    MissingProvenance,
    ForbiddenGeneratorRole,
    InvalidVocabulary,
    DuplicateReference,
}

impl Display for DomainProfileViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingDescription => "Domain Profile requires name and summary",
            Self::MissingProvenance => "Domain Profile provenance is required",
            Self::ForbiddenGeneratorRole => {
                "Domain Profile cannot encode curricula, Path graphs, schedulers, selection, or activation"
            }
            Self::InvalidVocabulary => "Domain Profile vocabulary entries must not be empty",
            Self::DuplicateReference => "Domain Profile candidate/content refs must be unique",
        })
    }
}

impl Error for DomainProfileViolation {}

/// Enabled Pack entry in a System Manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnabledPack {
    /// Pack identifier.
    pub pack_id: PackId,
    /// Exact enabled version.
    pub version: SemVer,
    /// Deterministic rule identifiers explaining inclusion.
    pub reason_rule_refs: Vec<ComposerRuleId>,
}

/// Omitted Pack and deterministic explanation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmittedPack {
    /// Pack identifier.
    pub pack_id: PackId,
    /// Deterministic rule identifiers explaining omission.
    pub reason_rule_refs: Vec<ComposerRuleId>,
}

/// Domain Profile version selected in the manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedDomainProfile {
    /// Profile identifier.
    pub profile_id: DomainProfileId,
    /// Exact version.
    pub version: SemVer,
}

/// User parameter override with explicit provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserParameterOverride {
    /// Protocol whose parameter is overridden.
    pub protocol_id: ProtocolId,
    /// Immutable Protocol Version.
    pub protocol_version: SemVer,
    /// Parameter key.
    pub parameter_key: String,
    /// User-specific value.
    pub value: LiteralValue,
    /// Provenance such as `user_override` or `personal_calibration`.
    pub provenance: String,
    /// Optional source record.
    pub source_ref: Option<OpaqueRef>,
}

/// Auditable System Manifest change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestChange {
    /// Manifest version produced by the change.
    pub manifest_version: u64,
    /// Change time.
    pub changed_at: Timestamp,
    /// User-readable change summary.
    pub summary: String,
    /// Decision authorizing a meaningful change, when applicable.
    pub decision_ref: Option<crate::DecisionRecordId>,
}

/// Versioned composition of one user's Flourish system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemManifest {
    /// Monotonic kernel manifest-shape version.
    pub schema_version: u64,
    /// Monotonic per-user manifest version.
    pub manifest_version: u64,
    /// Composer implementation version.
    pub composer_version: SemVer,
    /// Deterministic ruleset version.
    pub composer_ruleset_version: SemVer,
    /// Typed onboarding/answer snapshot.
    pub answers_snapshot_ref: ContextSnapshotId,
    /// Enabled Pack Versions and rule-backed reasons.
    pub enabled_packs: Vec<EnabledPack>,
    /// Omitted Packs and rule-backed reasons.
    pub omitted_packs: Vec<OmittedPack>,
    /// Selected Domain Profiles.
    pub domain_profiles: Vec<SelectedDomainProfile>,
    /// Selected immutable Protocol Versions.
    pub protocol_selections: Vec<ProtocolSelection>,
    /// User-specific parameter overrides.
    pub user_parameter_overrides: Vec<UserParameterOverride>,
    /// External bindings by reference, never secrets.
    pub integration_bindings: Vec<IntegrationBindingRef>,
    /// Current accepted Path version per Pursuit; empty is the valid pre-V2/no-Path shape.
    pub path_composition_refs: Vec<PathCompositionRef>,
    /// Auditable change history.
    pub manifest_changelog: Vec<ManifestChange>,
}

impl SystemManifest {
    /// Replaces Path refs after deterministic sorting and duplicate-Pursuit rejection.
    pub fn set_path_composition_refs(
        &mut self,
        mut references: Vec<PathCompositionRef>,
    ) -> Result<(), ManifestPathViolation> {
        if references
            .iter()
            .any(|reference| reference.validate().is_err())
        {
            return Err(ManifestPathViolation::InvalidPathVersion);
        }
        references.sort_by(|left, right| left.pursuit_ref.cmp(&right.pursuit_ref));
        if references
            .windows(2)
            .any(|window| window[0].pursuit_ref == window[1].pursuit_ref)
        {
            return Err(ManifestPathViolation::DuplicatePursuit);
        }
        self.path_composition_refs = references;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestPathViolation {
    DuplicatePursuit,
    InvalidPathVersion,
}

impl Display for ManifestPathViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::DuplicatePursuit => "manifest may reference at most one Path version per Pursuit",
            Self::InvalidPathVersion => "manifest Path references require a positive version",
        })
    }
}

impl Error for ManifestPathViolation {}

/// Versioned event envelope carrying identifiers/provenance, not copied domain state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// Stable identifier.
    pub id: EventId,
    /// Stable topic.
    pub topic: EventTopic,
    /// Publisher-owned topic schema version.
    pub topic_schema_version: u64,
    /// Occurrence time.
    pub occurred_at: Timestamp,
    /// Typed subject reference.
    pub subject_ref: TypedRef,
    /// Fixed kernel-service or pack publisher.
    pub publisher: OwningRef,
    /// Optional diagnostic source detail; never authoritative provenance.
    pub source: Option<String>,
    /// Topic-schema-typed payload.
    pub payload: LiteralValue,
}

/// Four-axis interface implemented only by Capability Packs.
pub trait LearningStrategy {
    /// Stable strategy identifier registered in `CapabilityPack.provides`.
    fn strategy_id(&self) -> &StrategyId;

    /// Declares feedback source and latency behavior.
    fn feedback_contract(&self) -> &str;

    /// Declares correctness or multidimensional quality behavior.
    fn quality_contract(&self) -> &str;

    /// Declares item-decomposition behavior.
    fn item_decomposition_contract(&self) -> &str;

    /// Declares re-exposure scheduling behavior.
    fn reexposure_contract(&self) -> &str;
}

/// Complete primary inputs from which Capability Evidence must be reconstructable.
#[derive(Debug, Clone, Copy)]
pub struct CapabilityEvidenceInputs<'a> {
    /// Discrete Attempts.
    pub attempts: &'a [Attempt],
    /// Immutable typed Observations.
    pub observations: &'a [Observation],
    /// User/system decisions.
    pub decisions: &'a [DecisionRecord],
    /// Structured reflection outcomes.
    pub reflection_outcomes: &'a [ReflectionOutcome],
}

/// Derived-view computation contract for Capability Evidence.
///
/// Implementations are owned by a pack's `LearningStrategy` together with
/// context assembly. The output may be cached only as a rebuildable projection:
/// it must be derivable solely from the supplied primary records, must never be
/// written by Motivation/Quest/gamification, and must never be a Calibration
/// Record's source of truth.
pub trait CapabilityEvidenceProjection {
    /// Pack/strategy-specific derived evidence shape.
    type Output;

    /// Reconstructs evidence for one subject from primary history.
    fn derive(&self, subject: &TypedRef, inputs: CapabilityEvidenceInputs<'_>) -> Self::Output;
}

/// Compile-time inventory helper for the fixed kernel-service set.
pub const KERNEL_SERVICES: [ServiceId; 14] = [
    ServiceId::ManifestResolver,
    ServiceId::ContextAssembler,
    ServiceId::ProtocolRuntime,
    ServiceId::RecommendationComposer,
    ServiceId::AvailabilityService,
    ServiceId::DueWorkScheduler,
    ServiceId::AttentionArbiter,
    ServiceId::ContentRegistry,
    ServiceId::CapabilityProbe,
    ServiceId::EventBus,
    ServiceId::MigrationEngine,
    ServiceId::ExportEraseService,
    ServiceId::BackupRestoreService,
    ServiceId::PermissionService,
];

/// Categories rejected by both mechanical `INVARIANT` and Commitment Device
/// condition checks. This constant is descriptive only; the two validators
/// remain independent functions with distinct acceptance rules.
pub const INFERENTIAL_CATEGORIES: [FactCategory; 4] = [
    FactCategory::Behavioral,
    FactCategory::Motivational,
    FactCategory::SelfReport,
    FactCategory::WellbeingDerived,
];

fn has_duplicates<T: PartialEq>(values: &[T]) -> bool {
    values
        .iter()
        .enumerate()
        .any(|(index, value)| values[index + 1..].contains(value))
}

fn is_allowed_context_priors(value: &LiteralValue) -> bool {
    const ALLOWED_ROLES: [&str; 6] = [
        "domain_constraints",
        "domain_context",
        "explanation_context",
        "factual_context",
        "ui_terminology",
        "vocabulary_context",
    ];
    match value {
        LiteralValue::Null => true,
        LiteralValue::Object(entries) => entries.iter().all(|(role, context)| {
            ALLOWED_ROLES.contains(&role.as_str()) && is_unordered_context_value(context)
        }),
        _ => false,
    }
}

fn is_unordered_context_value(value: &LiteralValue) -> bool {
    match value {
        LiteralValue::Text(_) | LiteralValue::List(_) => false,
        LiteralValue::Object(entries) => entries
            .iter()
            .all(|(key, nested)| !key.trim().is_empty() && is_unordered_context_value(nested)),
        _ => true,
    }
}
