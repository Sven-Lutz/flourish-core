//! Personal-outcome and longitudinal-learning validation for governed Method execution.
//!
//! This module closes the post-action half of the Flourish loop without creating scientific
//! authority. It distinguishes an accepted recommendation from an observed outcome, keeps optional
//! reflection separate from raw journal prose, and allows PersonalPattern / CalibrationRecord only
//! when their observation basis resolves to explicitly evaluated personal evidence.
//!
//! Scientific Evidence types are intentionally absent from every mutation/validation path here.

use crate::{
    Attempt, CalibrationRecord, DecisionRecordId, MethodExecution, MethodExecutionInitiation,
    Observation, ObservationId, PersonalPattern, ReflectionOutcome, Session, SessionId, Timestamp,
    TypedRef,
};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// User-specific usefulness assessment made only after observable outcome evidence exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodOutcomeUsefulness {
    Useful,
    Neutral,
    NotUseful,
    Unclear,
}

/// Explicit post-action evaluation. An `Accept` decision is never sufficient evidence by itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodOutcomeEvaluation {
    /// Decision that authorized the recommendation-backed execution, when applicable.
    pub source_decision_ref: Option<DecisionRecordId>,
    /// Cautious personal assessment; never a scientific grade or identity claim.
    pub usefulness: MethodOutcomeUsefulness,
    /// Concrete Observations supporting this assessment.
    pub supporting_observation_refs: Vec<ObservationId>,
    /// Conditions under which the outcome was observed.
    pub conditions: Vec<String>,
    /// Costs, trade-offs, or adverse effects noticed by the user/system.
    pub trade_offs: Vec<String>,
    /// Explicit uncertainty / limits of generalization.
    pub uncertainty: String,
    /// Time at which the evaluation was recorded.
    pub evaluated_at: Timestamp,
}

impl MethodOutcomeEvaluation {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_decision_ref: Option<DecisionRecordId>,
        usefulness: MethodOutcomeUsefulness,
        supporting_observation_refs: Vec<ObservationId>,
        conditions: Vec<String>,
        trade_offs: Vec<String>,
        uncertainty: impl Into<String>,
        evaluated_at: Timestamp,
    ) -> Result<Self, MethodOutcomeViolation> {
        if supporting_observation_refs.is_empty() {
            return Err(MethodOutcomeViolation::EvaluationRequiresObservation);
        }
        let mut unique = BTreeSet::new();
        for reference in &supporting_observation_refs {
            if !unique.insert(reference.clone()) {
                return Err(MethodOutcomeViolation::DuplicateEvaluationObservation(
                    reference.clone(),
                ));
            }
        }
        if conditions.iter().any(|value| value.trim().is_empty()) {
            return Err(MethodOutcomeViolation::EmptyEvaluationText("conditions"));
        }
        if trade_offs.iter().any(|value| value.trim().is_empty()) {
            return Err(MethodOutcomeViolation::EmptyEvaluationText("trade_offs"));
        }
        let uncertainty = uncertainty.into();
        if uncertainty.trim().is_empty() {
            return Err(MethodOutcomeViolation::MissingUncertainty);
        }
        Ok(Self {
            source_decision_ref,
            usefulness,
            supporting_observation_refs,
            conditions,
            trade_offs,
            uncertainty,
            evaluated_at,
        })
    }
}

/// Proof that a pack module resolved a pack-owned subject, using its own pack-specific logic, as
/// belonging to this exact execution's Session.
///
/// The kernel has no generic way to resolve pack-owned scope itself — a `TypedRef::PackOwned`
/// carries no pack or execution lineage — so it never treats an unproven pack-owned subject as
/// in scope. A pack module constructs one proof per cited pack-owned subject after doing its own
/// validation; the kernel then enforces only that the proof names the exact subject and the exact
/// Session being validated, and that it carries a non-empty validator attribution for audit.
///
/// This cannot make an untrustworthy pack module honest. What it does is make the assertion
/// explicit and attributable: a caller can no longer get pack-owned evidence into an outcome
/// without naming, per subject, who checked it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackOwnedScopeProof {
    subject_ref: TypedRef,
    execution_session_ref: SessionId,
    validated_by: String,
}

impl PackOwnedScopeProof {
    pub fn new(
        subject_ref: TypedRef,
        execution_session_ref: SessionId,
        validated_by: impl Into<String>,
    ) -> Result<Self, PackOwnedScopeProofViolation> {
        if !matches!(subject_ref, TypedRef::PackOwned { .. }) {
            return Err(PackOwnedScopeProofViolation::NotPackOwned);
        }
        let validated_by = validated_by.into();
        if validated_by.trim().is_empty() {
            return Err(PackOwnedScopeProofViolation::MissingValidatorAttribution);
        }
        Ok(Self {
            subject_ref,
            execution_session_ref,
            validated_by,
        })
    }

    pub fn subject_ref(&self) -> &TypedRef {
        &self.subject_ref
    }

    pub fn execution_session_ref(&self) -> &SessionId {
        &self.execution_session_ref
    }

    pub fn validated_by(&self) -> &str {
        &self.validated_by
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackOwnedScopeProofViolation {
    NotPackOwned,
    MissingValidatorAttribution,
}

impl Display for PackOwnedScopeProofViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NotPackOwned => "a pack-owned scope proof must name a PackOwned subject",
            Self::MissingValidatorAttribution => {
                "a pack-owned scope proof requires a non-empty validator attribution"
            }
        })
    }
}

impl Error for PackOwnedScopeProofViolation {}

/// One bounded Method execution plus the personal evidence actually observed afterwards.
pub struct MethodOutcomeEvidence<'a> {
    pub execution: &'a MethodExecution,
    pub session: &'a Session,
    pub attempts: &'a [Attempt],
    pub observations: &'a [Observation],
    pub reflection: Option<&'a ReflectionOutcome>,
    pub evaluation: &'a MethodOutcomeEvaluation,
    /// One proof per pack-owned subject this outcome cites. Empty is the ordinary case: without a
    /// proof, a pack-owned subject is out of scope rather than silently accepted.
    pub pack_scope_proofs: &'a [PackOwnedScopeProof],
}

/// Cross-execution personal-learning bundle. This is still personal evidence only.
pub struct PersonalLearningEvidence<'a> {
    pub outcomes: &'a [MethodOutcomeEvidence<'a>],
    pub personal_patterns: &'a [PersonalPattern],
    pub calibrations: &'a [CalibrationRecord],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodOutcomeViolation {
    EvaluationRequiresObservation,
    DuplicateEvaluationObservation(ObservationId),
    EmptyEvaluationText(&'static str),
    MissingUncertainty,
    EvaluationBeforeExecution,
    SessionMismatch,
    AttemptSessionMismatch,
    ReflectionSessionMismatch,
    DuplicateObservation(ObservationId),
    ObservationOutOfScope(ObservationId),
    EvaluationObservationMissing(ObservationId),
    EvaluationDecisionMismatch,
    UnexpectedEvaluationDecision,
    EmptyLearningBundle,
    PatternSubjectOutOfScope,
    PatternObservationNotEvaluated(ObservationId),
    PatternObservationSubjectMismatch(ObservationId),
    CalibrationMissingEvidence,
    CalibrationDuplicateObservation(ObservationId),
    CalibrationObservationNotEvaluated(ObservationId),
    CalibrationObservationProtocolMismatch(ObservationId),
    CalibrationProtocolMismatch,
}

impl Display for MethodOutcomeViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EvaluationRequiresObservation => formatter.write_str(
                "a Method outcome evaluation requires at least one supporting Observation",
            ),
            Self::DuplicateEvaluationObservation(_) => formatter
                .write_str("a Method outcome evaluation cannot cite the same Observation twice"),
            Self::EmptyEvaluationText(field) => {
                write!(formatter, "Method outcome evaluation {field} must not contain empty text")
            }
            Self::MissingUncertainty => formatter
                .write_str("a Method outcome evaluation must state uncertainty explicitly"),
            Self::EvaluationBeforeExecution => formatter
                .write_str("a Method outcome evaluation cannot predate the Method execution"),
            Self::SessionMismatch => {
                formatter.write_str("Method execution does not name the supplied Session")
            }
            Self::AttemptSessionMismatch => {
                formatter.write_str("an Attempt belongs to a different Session")
            }
            Self::ReflectionSessionMismatch => formatter
                .write_str("ReflectionOutcome belongs to a different or unspecified Session"),
            Self::DuplicateObservation(_) => formatter
                .write_str("a Method outcome bundle cannot contain the same Observation twice"),
            Self::ObservationOutOfScope(_) => formatter.write_str(
                "an Observation is outside the Session / Attempt / Pursuit / PracticeItem / reflection scope",
            ),
            Self::EvaluationObservationMissing(_) => formatter.write_str(
                "the Method outcome evaluation cites an Observation absent from the outcome bundle",
            ),
            Self::EvaluationDecisionMismatch => formatter.write_str(
                "the Method outcome evaluation does not cite the decision that authorized execution",
            ),
            Self::UnexpectedEvaluationDecision => formatter.write_str(
                "a directly user-started Method outcome cannot fabricate a recommendation decision",
            ),
            Self::EmptyLearningBundle => formatter
                .write_str("personal learning requires at least one validated Method outcome"),
            Self::PatternSubjectOutOfScope => formatter
                .write_str("PersonalPattern subject is outside the validated personal-outcome scope"),
            Self::PatternObservationNotEvaluated(_) => formatter.write_str(
                "PersonalPattern cites an Observation that was not part of an explicit outcome evaluation",
            ),
            Self::PatternObservationSubjectMismatch(_) => formatter.write_str(
                "PersonalPattern cites an evaluated Observation from an outcome that does not contain the pattern subject",
            ),
            Self::CalibrationMissingEvidence => formatter.write_str(
                "CalibrationRecord requires at least one evaluated personal Observation",
            ),
            Self::CalibrationDuplicateObservation(_) => formatter
                .write_str("CalibrationRecord cannot cite the same Observation twice"),
            Self::CalibrationObservationNotEvaluated(_) => formatter.write_str(
                "CalibrationRecord cites an Observation that was not part of an explicit outcome evaluation",
            ),
            Self::CalibrationObservationProtocolMismatch(_) => formatter.write_str(
                "CalibrationRecord cites an evaluated Observation from a different Protocol outcome",
            ),
            Self::CalibrationProtocolMismatch => formatter.write_str(
                "CalibrationRecord does not target a Protocol represented by the validated Method outcomes",
            ),
        }
    }
}

impl Error for MethodOutcomeViolation {}

/// Validates one execution -> observation -> optional reflection -> explicit outcome chain.
pub fn validate_method_outcome(
    outcome: &MethodOutcomeEvidence<'_>,
) -> Result<(), MethodOutcomeViolation> {
    if outcome.execution.session_ref != outcome.session.id {
        return Err(MethodOutcomeViolation::SessionMismatch);
    }
    if outcome.evaluation.evaluated_at < outcome.execution.created_at {
        return Err(MethodOutcomeViolation::EvaluationBeforeExecution);
    }

    for attempt in outcome.attempts {
        if attempt.session_ref.as_ref() != Some(&outcome.session.id) {
            return Err(MethodOutcomeViolation::AttemptSessionMismatch);
        }
    }

    if let Some(reflection) = outcome.reflection {
        if reflection.session_ref.as_ref() != Some(&outcome.session.id) {
            return Err(MethodOutcomeViolation::ReflectionSessionMismatch);
        }
    }

    let mut observation_ids = BTreeSet::new();
    for observation in outcome.observations {
        if !observation_ids.insert(observation.id().clone()) {
            return Err(MethodOutcomeViolation::DuplicateObservation(
                observation.id().clone(),
            ));
        }
        if !observation_in_outcome_scope(observation, outcome) {
            return Err(MethodOutcomeViolation::ObservationOutOfScope(
                observation.id().clone(),
            ));
        }
    }

    for reference in &outcome.evaluation.supporting_observation_refs {
        if !observation_ids.contains(reference) {
            return Err(MethodOutcomeViolation::EvaluationObservationMissing(
                reference.clone(),
            ));
        }
    }

    match outcome.execution.initiation {
        MethodExecutionInitiation::User => {
            if outcome.evaluation.source_decision_ref.is_some() {
                return Err(MethodOutcomeViolation::UnexpectedEvaluationDecision);
            }
        }
        MethodExecutionInitiation::AcceptedRecommendation => {
            if outcome.execution.source_decision_ref.as_ref()
                != outcome.evaluation.source_decision_ref.as_ref()
            {
                return Err(MethodOutcomeViolation::EvaluationDecisionMismatch);
            }
        }
    }

    Ok(())
}

/// Validates PersonalPattern / CalibrationRecord only against already-evaluated personal outcomes.
///
/// This function has no access to `EvidenceClaim`, evidence review state, or scientific grades, so
/// user success cannot flow backwards into Scientific Evidence through this boundary.
pub fn validate_personal_learning(
    learning: &PersonalLearningEvidence<'_>,
) -> Result<(), MethodOutcomeViolation> {
    if learning.outcomes.is_empty() {
        return Err(MethodOutcomeViolation::EmptyLearningBundle);
    }

    let mut evaluated_observations = BTreeSet::new();
    for outcome in learning.outcomes {
        validate_method_outcome(outcome)?;
        evaluated_observations.extend(
            outcome
                .evaluation
                .supporting_observation_refs
                .iter()
                .cloned(),
        );
    }

    for pattern in learning.personal_patterns {
        if !pattern_subject_in_learning_scope(pattern.subject_ref(), learning.outcomes) {
            return Err(MethodOutcomeViolation::PatternSubjectOutOfScope);
        }
        for reference in pattern.supporting_observation_refs() {
            if !evaluated_observations.contains(reference) {
                return Err(MethodOutcomeViolation::PatternObservationNotEvaluated(
                    reference.clone(),
                ));
            }
            let correlated = learning.outcomes.iter().any(|outcome| {
                outcome
                    .evaluation
                    .supporting_observation_refs
                    .contains(reference)
                    && pattern_subject_in_outcome_scope(pattern.subject_ref(), outcome)
            });
            if !correlated {
                return Err(MethodOutcomeViolation::PatternObservationSubjectMismatch(
                    reference.clone(),
                ));
            }
        }
    }

    for calibration in learning.calibrations {
        if calibration.evidentiary_observation_refs.is_empty() {
            return Err(MethodOutcomeViolation::CalibrationMissingEvidence);
        }
        let protocol_present = learning.outcomes.iter().any(|outcome| {
            outcome.execution.protocol_ref.as_ref().is_some_and(|protocol| {
                protocol.protocol_id == calibration.protocol_ref.0
                    && protocol.protocol_version == calibration.protocol_ref.1
            })
        });
        if !protocol_present {
            return Err(MethodOutcomeViolation::CalibrationProtocolMismatch);
        }

        let mut unique = BTreeSet::new();
        for reference in &calibration.evidentiary_observation_refs {
            if !unique.insert(reference.clone()) {
                return Err(MethodOutcomeViolation::CalibrationDuplicateObservation(
                    reference.clone(),
                ));
            }
            if !evaluated_observations.contains(reference) {
                return Err(MethodOutcomeViolation::CalibrationObservationNotEvaluated(
                    reference.clone(),
                ));
            }
            let evaluated_under_target_protocol = learning.outcomes.iter().any(|outcome| {
                outcome.execution.protocol_ref.as_ref().is_some_and(|protocol| {
                    protocol.protocol_id == calibration.protocol_ref.0
                        && protocol.protocol_version == calibration.protocol_ref.1
                }) && outcome
                    .evaluation
                    .supporting_observation_refs
                    .contains(reference)
            });
            if !evaluated_under_target_protocol {
                return Err(MethodOutcomeViolation::CalibrationObservationProtocolMismatch(
                    reference.clone(),
                ));
            }
        }
    }

    Ok(())
}

fn observation_in_outcome_scope(
    observation: &Observation,
    outcome: &MethodOutcomeEvidence<'_>,
) -> bool {
    match observation.subject_ref() {
        TypedRef::Session(id) => id == &outcome.session.id,
        TypedRef::Attempt(id) => outcome.attempts.iter().any(|attempt| &attempt.id == id),
        TypedRef::Pursuit(id) => outcome.session.pursuit_ref.as_ref() == Some(id),
        TypedRef::PracticeItem(id) => {
            outcome.session.practice_item_refs.iter().any(|candidate| candidate == id)
                || outcome
                    .attempts
                    .iter()
                    .any(|attempt| &attempt.practice_item_ref == id)
        }
        TypedRef::ReflectionOutcome(id) => outcome
            .reflection
            .is_some_and(|reflection| &reflection.id == id),
        // A pack-owned subject carries no lineage the kernel can resolve, so it is in scope only
        // through an exact-match proof for this subject and this Session. Every other subject kind
        // stays out of scope rather than being silently accepted as part of this execution.
        subject @ TypedRef::PackOwned { .. } => pack_owned_subject_proven(subject, outcome),
        _ => false,
    }
}

/// Whether a pack module explicitly vouched for this exact pack-owned subject in this exact
/// Session. Both must match: a proof for a sibling pack entity, or the same entity in another
/// Session, authorizes nothing.
fn pack_owned_subject_proven(subject: &TypedRef, outcome: &MethodOutcomeEvidence<'_>) -> bool {
    outcome.pack_scope_proofs.iter().any(|proof| {
        proof.subject_ref() == subject && proof.execution_session_ref() == &outcome.session.id
    })
}

fn pattern_subject_in_learning_scope(
    subject: &TypedRef,
    outcomes: &[MethodOutcomeEvidence<'_>],
) -> bool {
    outcomes
        .iter()
        .any(|outcome| pattern_subject_in_outcome_scope(subject, outcome))
}

fn pattern_subject_in_outcome_scope(
    subject: &TypedRef,
    outcome: &MethodOutcomeEvidence<'_>,
) -> bool {
    match subject {
        TypedRef::Session(id) => id == &outcome.session.id,
        TypedRef::Attempt(id) => outcome.attempts.iter().any(|attempt| &attempt.id == id),
        TypedRef::Pursuit(id) => outcome.session.pursuit_ref.as_ref() == Some(id),
        TypedRef::PracticeItem(id) => {
            outcome.session.practice_item_refs.iter().any(|candidate| candidate == id)
                || outcome
                    .attempts
                    .iter()
                    .any(|attempt| &attempt.practice_item_ref == id)
        }
        // A proof alone puts an Observation in scope; a Pattern additionally requires that the
        // Observation was explicitly evaluated. Personal truth about a pack-owned subject therefore
        // needs both the pack's attestation and the outcome's own evaluation of the evidence.
        TypedRef::PackOwned { .. } => {
            pack_owned_subject_proven(subject, outcome)
                && outcome.observations.iter().any(|observation| {
                    observation.subject_ref() == subject
                        && outcome
                            .evaluation
                            .supporting_observation_refs
                            .contains(observation.id())
                })
        }
        _ => false,
    }
}
