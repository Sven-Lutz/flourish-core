//! Read-only bridge from validated personal learning into future Method consideration.
//!
//! This is context, not authority: PersonalPatterns remain `INFERRED`/`HYPOTHESIS`, calibrations
//! remain user-specific Protocol state, and every item retains its Observation provenance. The
//! bridge has no Scientific Evidence mutation path and creates no parallel memory/state system.

use crate::method_outcome::{
    validate_personal_learning, MethodOutcomeViolation, PersonalLearningEvidence,
};
use crate::{
    CalibrationRecordId, CalibrationStatus, EpistemicClass, LiteralValue, ObservationId,
    PersonalPatternId, TypedRef, VersionedProtocolRef,
};

/// One bounded personal signal available to a future recommendation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonalOutcomeContextItem {
    /// A cautious user-specific inference with explicit uncertainty and observation basis.
    Pattern {
        pattern_ref: PersonalPatternId,
        subject_ref: TypedRef,
        epistemic_class: EpistemicClass,
        statement: String,
        supporting_observation_refs: Vec<ObservationId>,
        uncertainty: LiteralValue,
    },
    /// A user-specific Protocol calibration. It is never generalized beyond this Protocol version.
    Calibration {
        calibration_ref: CalibrationRecordId,
        protocol_ref: VersionedProtocolRef,
        parameter_key: String,
        proposed_value: LiteralValue,
        applied_value: Option<LiteralValue>,
        status: CalibrationStatus,
        supporting_observation_refs: Vec<ObservationId>,
    },
}

/// Validated personal context projected for one exact recommendation subject and Protocol.
/// Fields are private so external callers cannot fabricate a supposedly validated context bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalOutcomeContext {
    subject_ref: TypedRef,
    protocol_ref: VersionedProtocolRef,
    items: Vec<PersonalOutcomeContextItem>,
}

impl PersonalOutcomeContext {
    /// Projects only already-validated personal learning relevant to this exact subject/Protocol.
    ///
    /// Patterns are deliberately filtered to an exact subject match. Cross-subject generalization
    /// must be separately derived as a new PersonalPattern rather than inferred here. Calibrations
    /// are filtered to the exact Protocol version, preventing one Method's personal adaptation from
    /// leaking into another.
    pub fn from_validated_learning(
        learning: &PersonalLearningEvidence<'_>,
        subject_ref: TypedRef,
        protocol_ref: VersionedProtocolRef,
    ) -> Result<Self, MethodOutcomeViolation> {
        validate_personal_learning(learning)?;

        let mut items = Vec::new();
        for pattern in learning.personal_patterns {
            if pattern.subject_ref() != &subject_ref {
                continue;
            }
            items.push(PersonalOutcomeContextItem::Pattern {
                pattern_ref: pattern.id().clone(),
                subject_ref: pattern.subject_ref().clone(),
                epistemic_class: pattern.epistemic_class(),
                statement: pattern.statement().to_owned(),
                supporting_observation_refs: pattern.supporting_observation_refs().to_vec(),
                uncertainty: pattern.uncertainty().clone(),
            });
        }

        for calibration in learning.calibrations {
            if calibration.protocol_ref.0 != protocol_ref.protocol_id
                || calibration.protocol_ref.1 != protocol_ref.protocol_version
            {
                continue;
            }
            items.push(PersonalOutcomeContextItem::Calibration {
                calibration_ref: calibration.id.clone(),
                protocol_ref: VersionedProtocolRef {
                    protocol_id: calibration.protocol_ref.0.clone(),
                    protocol_version: calibration.protocol_ref.1.clone(),
                },
                parameter_key: calibration.parameter_key.clone(),
                proposed_value: calibration.proposed_value.clone(),
                applied_value: calibration.applied_value.clone(),
                status: calibration.status,
                supporting_observation_refs: calibration.evidentiary_observation_refs.clone(),
            });
        }

        Ok(Self {
            subject_ref,
            protocol_ref,
            items,
        })
    }

    pub fn subject_ref(&self) -> &TypedRef {
        &self.subject_ref
    }

    pub fn protocol_ref(&self) -> &VersionedProtocolRef {
        &self.protocol_ref
    }

    pub fn items(&self) -> &[PersonalOutcomeContextItem] {
        &self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
