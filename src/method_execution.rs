//! User-authorized Method execution linked to the existing Session identity.
//!
//! This module starts no independent lifecycle. A recommendation response is checked against the
//! exact subject, context, Protocol and optional AI proposal provenance, then produces the existing
//! `MethodExecution` metadata keyed by `SessionId`. Decline/defer/modify remain complete outcomes.

use crate::method_runtime::MethodRecommendation;
use crate::{
    DecisionKind, DecisionOutcome, DecisionRecord, EnforcementClass, MethodDefinition,
    MethodExecution, MethodExecutionInitiation, Protocol, ProtocolResponseChoice, RecommendationRef,
    Session, Timestamp, TypedRef, VersionedMethodRef, VersionedProtocolRef,
};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodDecisionResolution {
    Start(MethodExecution),
    /// A modified/alternative choice needs its own resolved Protocol before execution. The original
    /// recommendation is never silently reused.
    DoNotStart(ProtocolResponseChoice),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodDecisionViolation {
    SubjectMismatch,
    ContextMismatch,
    DecisionKindMismatch,
    RecommendationReferenceMismatch,
    MethodProtocolMismatch,
    ProtocolVariantNotRegistered,
    BehaviorMethodMustBeAdvisory,
    BehaviorMethodMustAllowDeferral,
    BehaviorMethodRequiresAlternative,
    ExecutionMetadata(crate::MethodExecutionViolation),
}

impl Display for MethodDecisionViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::SubjectMismatch => "Session or decision does not match recommendation subject",
            Self::ContextMismatch => "Session or decision does not cite recommendation context",
            Self::DecisionKindMismatch => "decision kind does not match recommendation provenance",
            Self::RecommendationReferenceMismatch => {
                "decision does not reference the recommendation it answers"
            }
            Self::MethodProtocolMismatch => {
                "Protocol method_ref does not name the requested Method"
            }
            Self::ProtocolVariantNotRegistered => {
                "Method does not register the requested Protocol as a variant"
            }
            Self::BehaviorMethodMustBeAdvisory => {
                "behavior-facing Method Protocol must be ADVISORY"
            }
            Self::BehaviorMethodMustAllowDeferral => {
                "behavior-facing Method Protocol must allow not-now/defer"
            }
            Self::BehaviorMethodRequiresAlternative => {
                "behavior-facing Method Protocol requires a meaningful alternative"
            }
            Self::ExecutionMetadata(_) => "Method execution metadata is invalid",
        })
    }
}

impl Error for MethodDecisionViolation {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ExecutionMetadata(error) => Some(error),
            _ => None,
        }
    }
}

/// Resolves an explicit user response to a surfaced Method recommendation.
///
/// Only `Accept` starts the exact recommended Protocol. Modify/Alternative/Decline/Defer do not
/// start it; another reviewed Protocol resolution is required for a changed choice.
pub fn resolve_method_decision(
    recommendation: &MethodRecommendation,
    decision: &DecisionRecord,
    session: &Session,
    created_at: Timestamp,
) -> Result<MethodDecisionResolution, MethodDecisionViolation> {
    if decision.subject_ref() != &recommendation.composition.subject_ref {
        return Err(MethodDecisionViolation::SubjectMismatch);
    }
    if let TypedRef::Pursuit(pursuit_id) = &recommendation.composition.subject_ref {
        if session.pursuit_ref.as_ref() != Some(pursuit_id) {
            return Err(MethodDecisionViolation::SubjectMismatch);
        }
    }
    if session.context_snapshot_ref.as_ref()
        != Some(&recommendation.composition.context_snapshot_ref)
        || decision.context_snapshot_ref() != &recommendation.composition.context_snapshot_ref
    {
        return Err(MethodDecisionViolation::ContextMismatch);
    }

    let response = match (
        recommendation.composition.source_proposal_ref.as_ref(),
        decision.decision_kind(),
        decision.recommendation_ref(),
        decision.outcome(),
    ) {
        (
            None,
            DecisionKind::ProtocolResponse,
            Some(RecommendationRef::Recommendation(id)),
            DecisionOutcome::ProtocolResponse(choice),
        ) if id == &recommendation.id => *choice,
        (
            None,
            DecisionKind::ProtocolResponse,
            Some(RecommendationRef::ProtocolVersion {
                protocol_id,
                protocol_version,
            }),
            DecisionOutcome::ProtocolResponse(choice),
        ) if protocol_id == &recommendation.composition.protocol_ref.protocol_id
            && protocol_version == &recommendation.composition.protocol_ref.protocol_version =>
        {
            *choice
        }
        (
            Some(source_proposal),
            DecisionKind::AiProposalResponse,
            Some(RecommendationRef::AiProposal(reference)),
            DecisionOutcome::AiProposalResponse(choice),
        ) if reference == source_proposal => *choice,
        (Some(_), DecisionKind::AiProposalResponse, _, _)
        | (None, DecisionKind::ProtocolResponse, _, _) => {
            return Err(MethodDecisionViolation::RecommendationReferenceMismatch)
        }
        _ => return Err(MethodDecisionViolation::DecisionKindMismatch),
    };

    if response != ProtocolResponseChoice::Accept {
        return Ok(MethodDecisionResolution::DoNotStart(response));
    }

    let execution = MethodExecution::new(
        session.id.clone(),
        recommendation.composition.method_ref.clone(),
        Some(recommendation.composition.protocol_ref.clone()),
        MethodExecutionInitiation::AcceptedRecommendation,
        recommendation.composition.source_proposal_ref.clone(),
        Some(decision.id().clone()),
        created_at,
    )
    .map_err(MethodDecisionViolation::ExecutionMetadata)?;

    Ok(MethodDecisionResolution::Start(execution))
}

/// Starts a Method the user explicitly chose, without fabricating a recommendation/AI provenance.
pub fn start_user_requested_method(
    method: &MethodDefinition,
    protocol: &Protocol,
    session: &Session,
    created_at: Timestamp,
) -> Result<MethodExecution, MethodDecisionViolation> {
    validate_method_protocol(method, protocol)?;
    MethodExecution::new(
        session.id.clone(),
        VersionedMethodRef {
            method_id: method.id.clone(),
            method_version: method.version.clone(),
        },
        Some(VersionedProtocolRef {
            protocol_id: protocol.id().clone(),
            protocol_version: protocol.version().clone(),
        }),
        MethodExecutionInitiation::User,
        None,
        None,
        created_at,
    )
    .map_err(MethodDecisionViolation::ExecutionMetadata)
}

fn validate_method_protocol(
    method: &MethodDefinition,
    protocol: &Protocol,
) -> Result<(), MethodDecisionViolation> {
    let method_ref = VersionedMethodRef {
        method_id: method.id.clone(),
        method_version: method.version.clone(),
    };
    if protocol.method_ref() != Some(&method_ref) {
        return Err(MethodDecisionViolation::MethodProtocolMismatch);
    }
    let protocol_ref = VersionedProtocolRef {
        protocol_id: protocol.id().clone(),
        protocol_version: protocol.version().clone(),
    };
    if !method.protocol_variants.contains(&protocol_ref) {
        return Err(MethodDecisionViolation::ProtocolVariantNotRegistered);
    }
    if protocol.enforcement_class() != EnforcementClass::Advisory {
        return Err(MethodDecisionViolation::BehaviorMethodMustBeAdvisory);
    }
    if !protocol.recommendation_policy().allows_not_now {
        return Err(MethodDecisionViolation::BehaviorMethodMustAllowDeferral);
    }
    if protocol.alternatives().is_empty() {
        return Err(MethodDecisionViolation::BehaviorMethodRequiresAlternative);
    }
    Ok(())
}
