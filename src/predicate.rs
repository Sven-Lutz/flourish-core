//! Declarative predicate grammar shared by Protocols, Composer rules, and
//! Commitment Devices.

use crate::{ContextFactRef, EventTopic, LiteralValue};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Comparison operation for a typed fact and literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    /// Equal.
    Eq,
    /// Not equal.
    Neq,
    /// Less than.
    Lt,
    /// Less than or equal.
    Lte,
    /// Greater than.
    Gt,
    /// Greater than or equal.
    Gte,
    /// Membership in a literal collection.
    In,
}

/// Non-negative duration represented without external time libraries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DurationMillis(u64);

impl DurationMillis {
    /// Creates a duration from milliseconds.
    pub const fn new(milliseconds: u64) -> Self {
        Self(milliseconds)
    }

    /// Returns the duration in milliseconds.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Small, typed, non-executable predicate tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Predicate {
    /// Reads a Context Fact as a boolean/truthy fact.
    Fact { fact_id: ContextFactRef },
    /// Compares a Context Fact with a typed literal.
    Compare {
        fact_id: ContextFactRef,
        op: CompareOp,
        literal: LiteralValue,
    },
    /// Tests elapsed time since a registered event topic.
    TimeElapsedSince {
        event_topic: EventTopic,
        duration: DurationMillis,
    },
    /// Tests whether a registered event topic occurred.
    EventOccurred {
        event_topic: EventTopic,
        within: Option<DurationMillis>,
    },
    /// Conjunction of child predicates.
    And(Vec<Predicate>),
    /// Disjunction of child predicates.
    Or(Vec<Predicate>),
    /// Negation of a child predicate.
    Not(Box<Predicate>),
}

impl Predicate {
    /// Convenience constructor for a Context Fact leaf.
    pub fn fact(fact_id: ContextFactRef) -> Self {
        Self::Fact { fact_id }
    }

    /// Convenience constructor for an event-occurrence leaf.
    pub fn event_occurred(event_topic: EventTopic) -> Self {
        Self::EventOccurred {
            event_topic,
            within: None,
        }
    }
}

/// Read-only view of the Context Facts and Event Topics a pack declared it
/// may consume.
///
/// Implementations are assembled from a pack's `requires`/`subscribes`
/// declarations at composition time. This access boundary is deliberately
/// separate from category eligibility: being declared here never makes a
/// predicate legal for an `INVARIANT` or Commitment Device condition.
pub trait PredicateAccessDeclaration {
    /// Whether the owning pack declared access to a Context Fact.
    fn allows_fact(&self, fact_id: &ContextFactRef) -> bool;

    /// Whether the owning pack subscribed to an Event Topic.
    fn allows_topic(&self, event_topic: &EventTopic) -> bool;
}

/// Predicate leaf that is outside its owning pack's declared access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredicateAccessViolation {
    /// A Context Fact was not declared through the pack's access contracts.
    UndeclaredFact(ContextFactRef),
    /// An Event Topic was not named in the pack's subscriptions.
    UnsubscribedTopic(EventTopic),
}

impl Display for PredicateAccessViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UndeclaredFact(fact_id) => {
                write!(formatter, "predicate references undeclared fact {fact_id}")
            }
            Self::UnsubscribedTopic(topic) => {
                write!(formatter, "predicate references unsubscribed topic {topic}")
            }
        }
    }
}

impl Error for PredicateAccessViolation {}

/// Validates every leaf against the owning pack's declared access.
///
/// This pure structural check covers both Context Fact (`Fact`/`Compare`) and
/// Event Topic (`EventOccurred`/`TimeElapsedSince`) families. Category legality
/// remains the independent responsibility of `check_invariant_eligibility` or
/// `check_condition_bound_eligibility`, depending on the consuming surface.
pub fn check_predicate_access(
    predicate: &Predicate,
    declaration: &dyn PredicateAccessDeclaration,
) -> Result<(), PredicateAccessViolation> {
    match predicate {
        Predicate::Fact { fact_id } | Predicate::Compare { fact_id, .. } => declaration
            .allows_fact(fact_id)
            .then_some(())
            .ok_or_else(|| PredicateAccessViolation::UndeclaredFact(fact_id.clone())),
        Predicate::TimeElapsedSince { event_topic, .. }
        | Predicate::EventOccurred { event_topic, .. } => declaration
            .allows_topic(event_topic)
            .then_some(())
            .ok_or_else(|| PredicateAccessViolation::UnsubscribedTopic(event_topic.clone())),
        Predicate::And(children) | Predicate::Or(children) => {
            for child in children {
                check_predicate_access(child, declaration)?;
            }
            Ok(())
        }
        Predicate::Not(child) => check_predicate_access(child, declaration),
    }
}
