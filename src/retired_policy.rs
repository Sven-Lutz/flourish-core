//! Shared retired-claim policy for catalog, product-copy, template, and AI-output boundaries.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Names and mechanisms that cannot re-enter live product content without a new governed
/// Evidence + Safety decision (`EVIDENCE_GOVERNANCE_V1.md` §8).
pub const RETIRED_TERMS: &[&str] = &[
    "boredom_frustration_threshold_seconds",
    "self_interruption_cooldown_seconds",
    "action_crisis_trigger_count",
    "target_asymptotic_stability_value",
    "lally_tolerance_allowed_misses_per_window",
    "failure_threshold_consecutive_days",
    "max_friction_duration_seconds",
    "default_outside_view_multiplier",
    "kausal_word_threshold_percent",
    "insight_word_threshold_percent",
    "pronoun_flexibility_ratio",
    "allow_unscheduled_todos",
    "max_daily_limit_hours",
    "karpicke_lock_intervals_hours",
    "scaffolding_fading_rate",
    "frozen_goal_inactivity_threshold_days",
    "threat_detection_trigger_minutes",
    "unrelated_order_maintenance_duration_minutes",
    "extension_target_hours",
    "ln_rmssd_deviation_tolerance",
    "cv_ln_rmssd_instability_threshold",
    "danger_zone_acwr_threshold",
    "zns_cooldown_hours",
    "cognitive_exhaustion_rpe_modifier",
    "72-hour work or training lock",
    "mental-fatigue work lock",
    "existential-crisis detection",
    "rumination detection",
    "forced detachment",
    "sleep enforcement",
    "mandatory bed-window extension",
    "psychological screen locking",
    "dopamine-pathway",
    "eeg monitoring",
    "burnout model",
    "reengagement readiness index",
    "asymptotic progress index",
    "isvegetativelyunstable",
    "notion_page_id",
    "password_hash",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetiredContentViolation {
    pub location: String,
    pub term: &'static str,
}

impl Display for RetiredContentViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} reintroduces retired content {:?}",
            self.location, self.term
        )
    }
}

impl Error for RetiredContentViolation {}

/// Rejects retired names/mechanisms in any authored or generated user-facing content.
pub fn validate_retired_content(
    location: impl Into<String>,
    content: &str,
) -> Result<(), RetiredContentViolation> {
    let location = location.into();
    let normalized = content.to_lowercase();
    for term in RETIRED_TERMS {
        if normalized.contains(term) {
            return Err(RetiredContentViolation { location, term });
        }
    }
    Ok(())
}
