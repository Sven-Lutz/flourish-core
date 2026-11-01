//! Governed scientific-evidence contracts for Flourish.
//!
//! The authoritative authoring workflow lives in version-controlled evidence files. Runtime code
//! may validate and read compiled snapshots, but it must never promote review state or turn
//! personal observations into scientific authority. See `docs/evidence/EVIDENCE_GOVERNANCE_V1.md`
//! and `docs/flourish/product-v2/FLOURISH_MASTER_APP_CANON_V2.md`.

use crate::{Confidence, DesignHypothesisId, EvidenceClaimId, Timestamp};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Stable identifier for an independently locatable scientific/practitioner source.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId(String);

impl SourceId {
    pub fn new(value: impl Into<String>) -> Result<Self, EvidenceViolation> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(EvidenceViolation::EmptyText("source.id"));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for SourceId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Shared review-state machine for `Source`, `EvidenceClaim`, and `DesignHypothesis`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceReviewState {
    Draft,
    NeedsVerification,
    Verified,
    Adopted,
    Rejected,
    Superseded,
}

impl EvidenceReviewState {
    /// True if the record reached a human-verified state at some point in its history.
    pub const fn has_human_verification(self) -> bool {
        matches!(self, Self::Verified | Self::Adopted | Self::Superseded)
    }

    /// True if a current Protocol may use this claim as an active scientific rationale.
    /// Superseded claims stay inspectable for provenance, but are not active authority.
    pub const fn is_active_protocol_basis(self) -> bool {
        matches!(self, Self::Verified | Self::Adopted)
    }
}

/// Source classification from the evidence-governance schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    PrimaryEmpirical,
    MetaAnalysis,
    SystematicReview,
    Theoretical,
    ConsensusStatement,
    Book,
    Other,
}

/// Evidence grade. `P` is personal-only and must never be presented as general scientific truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceGrade {
    A,
    B,
    C,
    H,
    P,
}

/// An externally locatable, independently authored artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub id: SourceId,
    pub citation: String,
    pub identifier: String,
    pub year: i32,
    pub source_type: SourceType,
    pub relevance_justification: String,
    pub added_by: String,
    pub added_at: Timestamp,
    pub review_state: EvidenceReviewState,
}

impl Source {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: SourceId,
        citation: impl Into<String>,
        identifier: impl Into<String>,
        year: i32,
        source_type: SourceType,
        relevance_justification: impl Into<String>,
        added_by: impl Into<String>,
        added_at: Timestamp,
        review_state: EvidenceReviewState,
    ) -> Result<Self, EvidenceViolation> {
        let citation = citation.into();
        let identifier = identifier.into();
        let relevance_justification = relevance_justification.into();
        let added_by = added_by.into();

        for (field, value) in [
            ("source.citation", citation.as_str()),
            ("source.identifier", identifier.as_str()),
            ("source.relevance_justification", relevance_justification.as_str()),
            ("source.added_by", added_by.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(EvidenceViolation::EmptyText(field));
            }
        }
        // A repository-relative `docs/` path is an internal project document. An independently
        // locatable HTTP(S) source is not internal merely because its URL happens to contain a
        // `/docs/` path segment.
        if is_internal_document_identifier(&identifier) {
            return Err(EvidenceViolation::InternalDocumentAsSource);
        }

        Ok(Self {
            id,
            citation,
            identifier,
            year,
            source_type,
            relevance_justification,
            added_by,
            added_at,
            review_state,
        })
    }
}

/// One source citation supporting an EvidenceClaim, anchored to a specific location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceClaimSourceRef {
    pub source_id: SourceId,
    pub source_anchor: String,
}

/// Atomic statement about what the cited source material supports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceClaim {
    pub id: EvidenceClaimId,
    pub atomic_statement: String,
    pub sources: Vec<EvidenceClaimSourceRef>,
    pub population_context: String,
    pub boundary_conditions: String,
    pub limitations: String,
    pub grade: EvidenceGrade,
    pub review_state: EvidenceReviewState,
    pub reviewer: Option<String>,
    pub reviewed_at: Option<Timestamp>,
    pub rejection_reason: Option<String>,
    pub superseded_by: Option<EvidenceClaimId>,
}

impl EvidenceClaim {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: EvidenceClaimId,
        atomic_statement: impl Into<String>,
        sources: Vec<EvidenceClaimSourceRef>,
        population_context: impl Into<String>,
        boundary_conditions: impl Into<String>,
        limitations: impl Into<String>,
        grade: EvidenceGrade,
        review_state: EvidenceReviewState,
        reviewer: Option<String>,
        reviewed_at: Option<Timestamp>,
        superseded_by: Option<EvidenceClaimId>,
    ) -> Result<Self, EvidenceViolation> {
        Self::new_with_lifecycle(
            id,
            atomic_statement,
            sources,
            population_context,
            boundary_conditions,
            limitations,
            grade,
            review_state,
            reviewer,
            reviewed_at,
            None,
            superseded_by,
        )
    }

    /// Constructs a claim with explicit terminal-state provenance.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_lifecycle(
        id: EvidenceClaimId,
        atomic_statement: impl Into<String>,
        sources: Vec<EvidenceClaimSourceRef>,
        population_context: impl Into<String>,
        boundary_conditions: impl Into<String>,
        limitations: impl Into<String>,
        grade: EvidenceGrade,
        review_state: EvidenceReviewState,
        reviewer: Option<String>,
        reviewed_at: Option<Timestamp>,
        rejection_reason: Option<String>,
        superseded_by: Option<EvidenceClaimId>,
    ) -> Result<Self, EvidenceViolation> {
        let atomic_statement = atomic_statement.into();
        let population_context = population_context.into();
        let boundary_conditions = boundary_conditions.into();
        let limitations = limitations.into();

        if atomic_statement.trim().is_empty() {
            return Err(EvidenceViolation::EmptyText("evidence_claim.atomic_statement"));
        }

        for source_ref in &sources {
            if source_ref.source_anchor.trim().is_empty() {
                return Err(EvidenceViolation::EmptyText(
                    "evidence_claim.sources.source_anchor",
                ));
            }
        }

        if sources.is_empty()
            && matches!(grade, EvidenceGrade::A | EvidenceGrade::B | EvidenceGrade::C)
        {
            return Err(EvidenceViolation::ClaimRequiresSource);
        }

        if review_state.has_human_verification()
            && (reviewer
                .as_ref()
                .is_none_or(|value| value.trim().is_empty())
                || reviewed_at.is_none())
        {
            return Err(EvidenceViolation::ClaimRequiresHumanReviewMetadata);
        }

        if matches!(review_state, EvidenceReviewState::Rejected)
            != rejection_reason
                .as_deref()
                .is_some_and(|reason| !reason.trim().is_empty())
        {
            return Err(EvidenceViolation::InvalidRejectionProvenance);
        }
        match (review_state, superseded_by.is_some()) {
            (EvidenceReviewState::Superseded, false) => {
                return Err(EvidenceViolation::SupersededClaimRequiresReplacement)
            }
            (EvidenceReviewState::Superseded, true) | (_, false) => {}
            (_, true) => return Err(EvidenceViolation::ReplacementOnlyAllowedWhenSuperseded),
        }
        // A record that supersedes itself passes both rules above and is still nonsense: it makes
        // the replacement chain a cycle, so provenance can never be walked to a live record.
        if superseded_by.as_ref() == Some(&id) {
            return Err(EvidenceViolation::SelfSupersession);
        }

        Ok(Self {
            id,
            atomic_statement,
            sources,
            population_context,
            boundary_conditions,
            limitations,
            grade,
            review_state,
            reviewer,
            reviewed_at,
            rejection_reason,
            superseded_by,
        })
    }
}

/// Explicit product inference drawn from one or more governed EvidenceClaims.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignHypothesis {
    pub id: DesignHypothesisId,
    pub statement: String,
    pub based_on: Vec<EvidenceClaimId>,
    pub confidence: Confidence,
    pub disconfirming_observation: String,
    pub review_state: EvidenceReviewState,
    pub reviewer: Option<String>,
    pub reviewed_at: Option<Timestamp>,
    pub rejection_reason: Option<String>,
    pub superseded_by: Option<DesignHypothesisId>,
}

impl DesignHypothesis {
    pub fn new(
        id: DesignHypothesisId,
        statement: impl Into<String>,
        based_on: Vec<EvidenceClaimId>,
        confidence: Confidence,
        disconfirming_observation: impl Into<String>,
        review_state: EvidenceReviewState,
    ) -> Result<Self, EvidenceViolation> {
        Self::new_with_lifecycle(
            id,
            statement,
            based_on,
            confidence,
            disconfirming_observation,
            review_state,
            None,
            None,
            None,
            None,
        )
    }

    /// Constructs a hypothesis with explicit human-review and terminal-state provenance.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_lifecycle(
        id: DesignHypothesisId,
        statement: impl Into<String>,
        based_on: Vec<EvidenceClaimId>,
        confidence: Confidence,
        disconfirming_observation: impl Into<String>,
        review_state: EvidenceReviewState,
        reviewer: Option<String>,
        reviewed_at: Option<Timestamp>,
        rejection_reason: Option<String>,
        superseded_by: Option<DesignHypothesisId>,
    ) -> Result<Self, EvidenceViolation> {
        let statement = statement.into();
        if statement.trim().is_empty() {
            return Err(EvidenceViolation::EmptyText("design_hypothesis.statement"));
        }
        if based_on.is_empty() {
            return Err(EvidenceViolation::HypothesisRequiresEvidenceClaim);
        }
        let disconfirming_observation = disconfirming_observation.into();
        if disconfirming_observation.trim().is_empty() {
            return Err(EvidenceViolation::EmptyText(
                "design_hypothesis.disconfirming_observation",
            ));
        }
        let reviewer_is_blank = reviewer
            .as_deref()
            .map(|value| value.trim().is_empty())
            .unwrap_or(true);
        if review_state.has_human_verification() && (reviewer_is_blank || reviewed_at.is_none()) {
            return Err(EvidenceViolation::ClaimRequiresHumanReviewMetadata);
        }
        if matches!(review_state, EvidenceReviewState::Rejected)
            != rejection_reason
                .as_deref()
                .is_some_and(|reason| !reason.trim().is_empty())
        {
            return Err(EvidenceViolation::InvalidRejectionProvenance);
        }
        match (review_state, superseded_by.is_some()) {
            (EvidenceReviewState::Superseded, false) => {
                return Err(EvidenceViolation::SupersededClaimRequiresReplacement)
            }
            (EvidenceReviewState::Superseded, true) | (_, false) => {}
            (_, true) => return Err(EvidenceViolation::ReplacementOnlyAllowedWhenSuperseded),
        }
        if superseded_by.as_ref() == Some(&id) {
            return Err(EvidenceViolation::SelfSupersession);
        }
        Ok(Self {
            id,
            statement,
            based_on,
            confidence,
            disconfirming_observation,
            review_state,
            reviewer,
            reviewed_at,
            rejection_reason,
            superseded_by,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceViolation {
    EmptyText(&'static str),
    InternalDocumentAsSource,
    ClaimRequiresSource,
    ClaimRequiresHumanReviewMetadata,
    SupersededClaimRequiresReplacement,
    ReplacementOnlyAllowedWhenSuperseded,
    InvalidRejectionProvenance,
    SelfSupersession,
    HypothesisRequiresEvidenceClaim,
}

impl Display for EvidenceViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyText(field) => write!(formatter, "{field} must not be empty"),
            Self::InternalDocumentAsSource => formatter.write_str(
                "a Source identifier must resolve to an independently locatable artifact, not an internal project document",
            ),
            Self::ClaimRequiresSource => formatter.write_str(
                "an EvidenceClaim graded A, B, or C must cite at least one Source",
            ),
            Self::ClaimRequiresHumanReviewMetadata => formatter.write_str(
                "VERIFIED, ADOPTED, or SUPERSEDED review state requires reviewer and reviewed_at",
            ),
            Self::SupersededClaimRequiresReplacement => formatter.write_str(
                "a SUPERSEDED EvidenceClaim must identify its replacement",
            ),
            Self::ReplacementOnlyAllowedWhenSuperseded => formatter.write_str(
                "superseded_by is valid only for a SUPERSEDED EvidenceClaim",
            ),
            Self::InvalidRejectionProvenance => formatter.write_str(
                "REJECTED review state requires a non-empty rejection_reason, and other states must not carry one",
            ),
            Self::SelfSupersession => {
                formatter.write_str("an evidence record cannot supersede itself")
            }
            Self::HypothesisRequiresEvidenceClaim => formatter.write_str(
                "a DesignHypothesis must be based on at least one EvidenceClaim",
            ),
        }
    }
}

impl Error for EvidenceViolation {}

fn is_internal_document_identifier(identifier: &str) -> bool {
    let normalized = identifier.trim().replace('\\', "/");
    if normalized.starts_with("http://") || normalized.starts_with("https://") {
        return false;
    }
    normalized
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != "." && *segment != "..")
        .next()
        == Some("docs")
}
