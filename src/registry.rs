//! Context-fact and event-topic registration contracts and eligibility rules.

use crate::{ContextFactRef, EventTopic, LiteralValue, ObservationType, OwningRef, Predicate};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Closed category shared by Context Facts and Event Topics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FactCategory {
    /// Schema, permission, secret, migration, backup, or referential integrity.
    SystemIntegrity,
    /// Facts derived from user actions or choices over time.
    Behavioral,
    /// Motivation/Quest-pack-derived facts.
    Motivational,
    /// Direct manual self-report.
    SelfReport,
    /// Regulation/wellbeing or linguistic-analysis-derived facts.
    WellbeingDerived,
    /// Ordinary product and operational state.
    Operational,
}

/// Directly observable basis that can qualify an operational fact or topic for
/// a Commitment Device condition.
///
/// The enum is intentionally closed and has no catch-all/default variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationalBasis {
    /// Explicit date, time, or calendar boundary.
    TimeBoundary,
    /// Mechanical count of directly recorded, unambiguous actions.
    DirectActionCount,
    /// Explicit completion of one directly recorded operation.
    DirectActionCompletion,
    /// Technical availability or integrity condition.
    TechnicalAvailability,
}

/// Declarative shape of a Context Fact value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueSchema {
    /// Boolean value.
    Boolean,
    /// Signed integer value.
    Integer,
    /// Unsigned integer value.
    Unsigned,
    /// Decimal value.
    Decimal,
    /// Text value.
    Text,
    /// UTC timestamp value.
    Timestamp,
    /// Duration value.
    Duration,
    /// Ordered homogeneous list.
    List(Box<ValueSchema>),
    /// Fixed object fields.
    Object(Vec<(String, ValueSchema)>),
    /// Closed set of literal values.
    Enum(Vec<LiteralValue>),
}

/// Typed provenance for a derived Context Fact definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DerivedFrom {
    /// An Observation type contributes to the fact.
    Observation(ObservationType),
    /// An Event Topic contributes to the fact.
    Event(EventTopic),
}

/// Invalid registry-definition shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryDefinitionViolation {
    /// Only an operational definition may carry an operational basis.
    OperationalBasisOnNonOperationalCategory,
    /// Topic schema versions are positive monotonic integers.
    ZeroTopicSchemaVersion,
}

impl Display for RegistryDefinitionViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::OperationalBasisOnNonOperationalCategory => formatter
                .write_str("operational_basis must be absent unless category is OPERATIONAL"),
            Self::ZeroTopicSchemaVersion => {
                formatter.write_str("topic_schema_version must be greater than zero")
            }
        }
    }
}

impl Error for RegistryDefinitionViolation {}

/// Registered definition of one Context Fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextFactDefinition {
    fact_id: ContextFactRef,
    category: FactCategory,
    operational_basis: Option<OperationalBasis>,
    owning_ref: OwningRef,
    value_schema: ValueSchema,
    derived_from: Vec<DerivedFrom>,
}

impl ContextFactDefinition {
    /// Creates a registry definition and enforces category/basis consistency.
    pub fn new(
        fact_id: ContextFactRef,
        category: FactCategory,
        operational_basis: Option<OperationalBasis>,
        owning_ref: OwningRef,
        value_schema: ValueSchema,
        derived_from: Vec<DerivedFrom>,
    ) -> Result<Self, RegistryDefinitionViolation> {
        if category != FactCategory::Operational && operational_basis.is_some() {
            return Err(RegistryDefinitionViolation::OperationalBasisOnNonOperationalCategory);
        }
        Ok(Self {
            fact_id,
            category,
            operational_basis,
            owning_ref,
            value_schema,
            derived_from,
        })
    }

    /// Stable fact identifier.
    pub fn fact_id(&self) -> &ContextFactRef {
        &self.fact_id
    }

    /// Closed fact category.
    pub const fn category(&self) -> FactCategory {
        self.category
    }

    /// Optional qualifying basis for operational commitment conditions.
    pub const fn operational_basis(&self) -> Option<OperationalBasis> {
        self.operational_basis
    }

    /// Kernel service or pack that defines the fact.
    pub fn owning_ref(&self) -> &OwningRef {
        &self.owning_ref
    }

    /// Declared fact value schema.
    pub fn value_schema(&self) -> &ValueSchema {
        &self.value_schema
    }

    /// Typed inputs from which the fact is derived.
    pub fn derived_from(&self) -> &[DerivedFrom] {
        &self.derived_from
    }
}

/// Registered definition of one Event Topic version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventTopicDefinition {
    topic: EventTopic,
    topic_schema_version: u64,
    publisher: OwningRef,
    category: FactCategory,
    operational_basis: Option<OperationalBasis>,
}

impl EventTopicDefinition {
    /// Creates a topic definition and enforces version and category/basis rules.
    pub fn new(
        topic: EventTopic,
        topic_schema_version: u64,
        publisher: OwningRef,
        category: FactCategory,
        operational_basis: Option<OperationalBasis>,
    ) -> Result<Self, RegistryDefinitionViolation> {
        if topic_schema_version == 0 {
            return Err(RegistryDefinitionViolation::ZeroTopicSchemaVersion);
        }
        if category != FactCategory::Operational && operational_basis.is_some() {
            return Err(RegistryDefinitionViolation::OperationalBasisOnNonOperationalCategory);
        }
        Ok(Self {
            topic,
            topic_schema_version,
            publisher,
            category,
            operational_basis,
        })
    }

    /// Stable topic identifier.
    pub fn topic(&self) -> &EventTopic {
        &self.topic
    }

    /// Monotonic schema version for this topic.
    pub const fn topic_schema_version(&self) -> u64 {
        self.topic_schema_version
    }

    /// Fixed publisher for the topic.
    pub fn publisher(&self) -> &OwningRef {
        &self.publisher
    }

    /// Closed topic category.
    pub const fn category(&self) -> FactCategory {
        self.category
    }

    /// Optional qualifying basis for operational commitment conditions.
    pub const fn operational_basis(&self) -> Option<OperationalBasis> {
        self.operational_basis
    }
}

/// Read-only Context Fact registry boundary.
pub trait ContextFactRegistry {
    /// Resolves a fact definition by its stable identifier.
    fn resolve_fact(&self, fact_id: &ContextFactRef) -> Option<&ContextFactDefinition>;
}

/// Read-only Event Topic registry boundary.
pub trait EventTopicRegistry {
    /// Resolves the current registered definition for a topic.
    fn resolve_topic(&self, topic: &EventTopic) -> Option<&EventTopicDefinition>;
}

/// Failure of the mechanical `INVARIANT` eligibility rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvariantViolation {
    /// A referenced Context Fact is not registered.
    UnregisteredFact(ContextFactRef),
    /// A referenced Event Topic is not registered.
    UnregisteredTopic(EventTopic),
    /// A fact is registered under a non-system-integrity category.
    IneligibleFact {
        fact_id: ContextFactRef,
        category: FactCategory,
    },
    /// A topic is registered under a non-system-integrity category.
    IneligibleTopic {
        topic: EventTopic,
        category: FactCategory,
    },
}

impl Display for InvariantViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnregisteredFact(fact_id) => {
                write!(
                    formatter,
                    "INVARIANT references unregistered fact {fact_id}"
                )
            }
            Self::UnregisteredTopic(topic) => {
                write!(formatter, "INVARIANT references unregistered topic {topic}")
            }
            Self::IneligibleFact { fact_id, category } => write!(
                formatter,
                "INVARIANT fact {fact_id} has ineligible category {category:?}"
            ),
            Self::IneligibleTopic { topic, category } => write!(
                formatter,
                "INVARIANT topic {topic} has ineligible category {category:?}"
            ),
        }
    }
}

impl Error for InvariantViolation {}

/// Checks the mechanical `INVARIANT` criterion.
///
/// Every Context Fact and Event Topic leaf must resolve and must be registered
/// as `SystemIntegrity`. `operational_basis` is deliberately ignored.
pub fn check_invariant_eligibility(
    predicate: &Predicate,
    fact_registry: &dyn ContextFactRegistry,
    topic_registry: &dyn EventTopicRegistry,
) -> Result<(), InvariantViolation> {
    match predicate {
        Predicate::Fact { fact_id } | Predicate::Compare { fact_id, .. } => {
            let definition = fact_registry
                .resolve_fact(fact_id)
                .ok_or_else(|| InvariantViolation::UnregisteredFact(fact_id.clone()))?;
            if definition.category() != FactCategory::SystemIntegrity {
                return Err(InvariantViolation::IneligibleFact {
                    fact_id: fact_id.clone(),
                    category: definition.category(),
                });
            }
            Ok(())
        }
        Predicate::TimeElapsedSince { event_topic, .. }
        | Predicate::EventOccurred { event_topic, .. } => {
            let definition = topic_registry
                .resolve_topic(event_topic)
                .ok_or_else(|| InvariantViolation::UnregisteredTopic(event_topic.clone()))?;
            if definition.category() != FactCategory::SystemIntegrity {
                return Err(InvariantViolation::IneligibleTopic {
                    topic: event_topic.clone(),
                    category: definition.category(),
                });
            }
            Ok(())
        }
        Predicate::And(children) | Predicate::Or(children) => {
            for child in children {
                check_invariant_eligibility(child, fact_registry, topic_registry)?;
            }
            Ok(())
        }
        Predicate::Not(child) => check_invariant_eligibility(child, fact_registry, topic_registry),
    }
}

/// Failure of the Commitment Device `condition_bound` eligibility rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionBoundViolation {
    /// A referenced Context Fact is not registered.
    UnregisteredFact(ContextFactRef),
    /// A referenced Event Topic is not registered.
    UnregisteredTopic(EventTopic),
    /// A prohibited category was referenced through a Context Fact.
    IneligibleFactCategory {
        fact_id: ContextFactRef,
        category: FactCategory,
    },
    /// A prohibited category was referenced through an Event Topic.
    IneligibleTopicCategory {
        topic: EventTopic,
        category: FactCategory,
    },
    /// An operational Context Fact lacks a qualifying basis.
    MissingFactOperationalBasis(ContextFactRef),
    /// An operational Event Topic lacks a qualifying basis.
    MissingTopicOperationalBasis(EventTopic),
}

impl Display for ConditionBoundViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnregisteredFact(fact_id) => write!(
                formatter,
                "CommitmentDevice condition references unregistered fact {fact_id}"
            ),
            Self::UnregisteredTopic(topic) => write!(
                formatter,
                "CommitmentDevice condition references unregistered topic {topic}"
            ),
            Self::IneligibleFactCategory { fact_id, category } => write!(
                formatter,
                "CommitmentDevice fact {fact_id} has prohibited category {category:?}"
            ),
            Self::IneligibleTopicCategory { topic, category } => write!(
                formatter,
                "CommitmentDevice topic {topic} has prohibited category {category:?}"
            ),
            Self::MissingFactOperationalBasis(fact_id) => write!(
                formatter,
                "operational fact {fact_id} has no condition-bound operational basis"
            ),
            Self::MissingTopicOperationalBasis(topic) => write!(
                formatter,
                "operational topic {topic} has no condition-bound operational basis"
            ),
        }
    }
}

impl Error for ConditionBoundViolation {}

/// Checks the independent Commitment Device `condition_bound` rule.
///
/// `SystemIntegrity` leaves are accepted. `Operational` leaves are accepted
/// only with any of the four closed `OperationalBasis` variants. Behavioral,
/// motivational, self-report, and wellbeing-derived leaves always reject.
/// This function intentionally does not delegate to the stricter INVARIANT
/// checker because the rules must remain independently testable and divergent.
pub fn check_condition_bound_eligibility(
    predicate: &Predicate,
    fact_registry: &dyn ContextFactRegistry,
    topic_registry: &dyn EventTopicRegistry,
) -> Result<(), ConditionBoundViolation> {
    match predicate {
        Predicate::Fact { fact_id } | Predicate::Compare { fact_id, .. } => {
            let definition = fact_registry
                .resolve_fact(fact_id)
                .ok_or_else(|| ConditionBoundViolation::UnregisteredFact(fact_id.clone()))?;
            match definition.category() {
                FactCategory::SystemIntegrity => Ok(()),
                FactCategory::Operational if definition.operational_basis().is_some() => Ok(()),
                FactCategory::Operational => Err(
                    ConditionBoundViolation::MissingFactOperationalBasis(fact_id.clone()),
                ),
                category => Err(ConditionBoundViolation::IneligibleFactCategory {
                    fact_id: fact_id.clone(),
                    category,
                }),
            }
        }
        Predicate::TimeElapsedSince { event_topic, .. }
        | Predicate::EventOccurred { event_topic, .. } => {
            let definition = topic_registry
                .resolve_topic(event_topic)
                .ok_or_else(|| ConditionBoundViolation::UnregisteredTopic(event_topic.clone()))?;
            match definition.category() {
                FactCategory::SystemIntegrity => Ok(()),
                FactCategory::Operational if definition.operational_basis().is_some() => Ok(()),
                FactCategory::Operational => Err(
                    ConditionBoundViolation::MissingTopicOperationalBasis(event_topic.clone()),
                ),
                category => Err(ConditionBoundViolation::IneligibleTopicCategory {
                    topic: event_topic.clone(),
                    category,
                }),
            }
        }
        Predicate::And(children) | Predicate::Or(children) => {
            for child in children {
                check_condition_bound_eligibility(child, fact_registry, topic_registry)?;
            }
            Ok(())
        }
        Predicate::Not(child) => {
            check_condition_bound_eligibility(child, fact_registry, topic_registry)
        }
    }
}
