use flourish_core::*;

fn timestamp() -> Timestamp {
    Timestamp::new("2026-09-07T16:00:00.000Z").unwrap()
}

fn source() -> Source {
    Source::new(
        SourceId::new("src.ellis_davidi_2005").unwrap(),
        "Ellis, S., & Davidi, I. (2005). After-event reviews: drawing lessons from successful and failed experience. Journal of Applied Psychology, 90(4), 857-871.",
        "https://doi.org/10.1037/0021-9010.90.4.857",
        2005,
        SourceType::PrimaryEmpirical,
        "Supports a bounded question about structured after-action review and subsequent performance.",
        "evidence-intake",
        timestamp(),
        EvidenceReviewState::NeedsVerification,
    )
    .unwrap()
}

fn claim(
    review_state: EvidenceReviewState,
    reviewer: Option<&str>,
    reviewed_at: Option<Timestamp>,
    superseded_by: Option<EvidenceClaimId>,
) -> Result<EvidenceClaim, EvidenceViolation> {
    EvidenceClaim::new(
        EvidenceClaimId::new("evidence.reflection.boundaries").unwrap(),
        "Structured after-action review can improve performance on a subsequent similar task.",
        vec![EvidenceClaimSourceRef {
            source_id: source().id,
            source_anchor: "p.860, Study 2".into(),
        }],
        "Adults performing bounded, repeatable tasks.",
        "Effect varies by task type and review structure.",
        "A bounded claim does not justify mandatory reflection for every user or activity.",
        EvidenceGrade::C,
        review_state,
        reviewer.map(str::to_owned),
        reviewed_at,
        superseded_by,
    )
}

#[test]
fn scientific_grades_require_a_source() {
    let error = EvidenceClaim::new(
        EvidenceClaimId::new("evidence.learning.example").unwrap(),
        "A bounded scientific claim.",
        vec![],
        "Relevant population.",
        "Context matters.",
        "Limitations remain visible.",
        EvidenceGrade::B,
        EvidenceReviewState::NeedsVerification,
        None,
        None,
        None,
    )
    .unwrap_err();

    assert_eq!(error, EvidenceViolation::ClaimRequiresSource);
}

#[test]
fn verified_claim_requires_human_review_metadata() {
    let error = claim(EvidenceReviewState::Verified, None, None, None).unwrap_err();
    assert_eq!(error, EvidenceViolation::ClaimRequiresHumanReviewMetadata);
}

#[test]
fn superseded_claim_is_historically_verified_but_not_active_protocol_authority() {
    let replacement = EvidenceClaimId::new("evidence.reflection.replacement").unwrap();
    let superseded = claim(
        EvidenceReviewState::Superseded,
        Some("human-reviewer"),
        Some(timestamp()),
        Some(replacement),
    )
    .unwrap();

    assert!(superseded.review_state.has_human_verification());
    assert!(!superseded.review_state.is_active_protocol_basis());
    assert!(EvidenceReviewState::Verified.is_active_protocol_basis());
    assert!(EvidenceReviewState::Adopted.is_active_protocol_basis());
}

#[test]
fn superseded_claim_requires_a_replacement_pointer() {
    let error = claim(
        EvidenceReviewState::Superseded,
        Some("human-reviewer"),
        Some(timestamp()),
        None,
    )
    .unwrap_err();

    assert_eq!(error, EvidenceViolation::SupersededClaimRequiresReplacement);
}

#[test]
fn replacement_pointer_is_rejected_for_current_claim() {
    let error = claim(
        EvidenceReviewState::Verified,
        Some("human-reviewer"),
        Some(timestamp()),
        Some(EvidenceClaimId::new("evidence.other").unwrap()),
    )
    .unwrap_err();

    assert_eq!(error, EvidenceViolation::ReplacementOnlyAllowedWhenSuperseded);
}

#[test]
fn internal_project_document_cannot_masquerade_as_scientific_source() {
    let error = Source::new(
        SourceId::new("src.internal").unwrap(),
        "Internal Flourish document",
        "docs/evidence/EVIDENCE_GOVERNANCE_V1.md",
        2026,
        SourceType::Other,
        "Should be rejected as circular evidence.",
        "evidence-intake",
        timestamp(),
        EvidenceReviewState::Draft,
    )
    .unwrap_err();

    assert_eq!(error, EvidenceViolation::InternalDocumentAsSource);
}

#[test]
fn design_hypothesis_requires_governed_evidence_basis() {
    let error = DesignHypothesis::new(
        DesignHypothesisId::new("hypothesis.after_action_review").unwrap(),
        "If the bounded claim holds, then a short optional review may help in matching contexts.",
        vec![],
        Confidence::Medium,
        "No useful adjustment appears relative to a suitable comparison.",
        EvidenceReviewState::NeedsVerification,
    )
    .unwrap_err();

    assert_eq!(error, EvidenceViolation::HypothesisRequiresEvidenceClaim);
}

#[test]
fn personal_grade_does_not_require_a_scientific_source() {
    let personal = EvidenceClaim::new(
        EvidenceClaimId::new("personal.observation.summary").unwrap(),
        "This is explicitly personal evidence, not a general scientific claim.",
        vec![],
        "One user.",
        "Applies only to the observed personal context.",
        "Cannot change scientific evidence grades.",
        EvidenceGrade::P,
        EvidenceReviewState::Draft,
        None,
        None,
        None,
    )
    .unwrap();

    assert_eq!(personal.grade, EvidenceGrade::P);
}
