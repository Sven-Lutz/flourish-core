//! Universal Capture contracts.
//!
//! Capture is universal infrastructure, not a persona capability and not a Path lifecycle. Raw
//! user input is preserved first; semantic candidates are proposals only. Voice enters this same
//! contract as a corrected transcript. Raw audio is deliberately absent from the durable model.

use crate::{EpistemicClass, SensitivityClass};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureInputKind {
    Text,
    VoiceTranscript,
    Link,
    File,
    Integration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureIntent {
    Capture,
    Reflect,
    Plan,
    ThinkDecide,
    ReviewUpdate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    Interpret,
    JustListen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureCandidateKind {
    Task,
    Event,
    Project,
    Person,
    Decision,
    Question,
    Reflection,
    Note,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureCandidateAction {
    Keep,
    Archive,
    RouteToSpecialist,
    ProposeStructuredChange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureCandidate {
    pub id: String,
    pub kind: CaptureCandidateKind,
    pub label: String,
    pub source_excerpt: String,
    pub epistemic_class: EpistemicClass,
    pub allowed_actions: Vec<CaptureCandidateAction>,
    pub unresolved_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureInterpretation {
    pub intent: CaptureIntent,
    pub mode: CaptureMode,
    pub candidates: Vec<CaptureCandidate>,
    /// Capture interpretation never grants durable mutation authority.
    pub requires_grouped_confirmation: bool,
}

impl CaptureInterpretation {
    pub fn no_operational_writes(&self) -> bool {
        self.mode == CaptureMode::JustListen || self.candidates.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureRequest {
    pub input_kind: CaptureInputKind,
    pub intent: CaptureIntent,
    pub mode: CaptureMode,
    pub raw_text: String,
    pub source_label: Option<String>,
    pub sensitivity: SensitivityClass,
}

impl CaptureRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.raw_text.trim().is_empty() {
            return Err("capture text must not be empty");
        }
        if self.input_kind == CaptureInputKind::Link {
            let parsed = url::Url::parse(self.raw_text.trim())
                .map_err(|_| "link capture requires a valid HTTP or HTTPS URL")?;
            if !matches!(parsed.scheme(), "http" | "https")
                || parsed.host_str().is_none()
                || !parsed.username().is_empty()
                || parsed.password().is_some()
            {
                return Err("link capture requires a credential-free HTTP or HTTPS URL");
            }
        }
        if (self.mode == CaptureMode::JustListen || self.intent == CaptureIntent::Reflect)
            && self.sensitivity != SensitivityClass::ReflectivePrivate
        {
            return Err("private reflection requires reflective-private sensitivity");
        }
        if self.input_kind == CaptureInputKind::VoiceTranscript
            && self.source_label.as_deref() == Some("raw-audio")
        {
            return Err("raw audio is not a durable capture source");
        }
        Ok(())
    }
}

/// Deterministic no-AI interpretation. It deliberately recognizes only explicit prefixes. Natural
/// language enrichment may add candidates later, but it must return through the same reviewable
/// contract rather than gaining writer authority.
pub fn interpret_capture(request: &CaptureRequest) -> Result<CaptureInterpretation, &'static str> {
    request.validate()?;
    if request.mode == CaptureMode::JustListen {
        return Ok(CaptureInterpretation {
            intent: request.intent,
            mode: request.mode,
            candidates: vec![],
            requires_grouped_confirmation: false,
        });
    }

    let mut candidates = Vec::new();
    for (index, raw_line) in request.raw_text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let lower = line.to_lowercase();
        let prefixes: &[(&[&str], CaptureCandidateKind)] = &[
            (&["todo:", "task:", "aufgabe:"], CaptureCandidateKind::Task),
            (&["termin:", "event:"], CaptureCandidateKind::Event),
            (&["projekt:", "project:"], CaptureCandidateKind::Project),
            (&["person:"], CaptureCandidateKind::Person),
            (
                &["entscheidung:", "decision:"],
                CaptureCandidateKind::Decision,
            ),
            (&["frage:", "question:"], CaptureCandidateKind::Question),
            (
                &["reflexion:", "reflection:"],
                CaptureCandidateKind::Reflection,
            ),
            (&["notiz:", "note:"], CaptureCandidateKind::Note),
        ];
        let mut matched = None;
        for (candidate_prefixes, kind) in prefixes {
            if let Some(prefix) = candidate_prefixes
                .iter()
                .find(|prefix| lower.starts_with(**prefix))
            {
                let label = line[prefix.len()..].trim();
                if !label.is_empty() {
                    matched = Some((*kind, label.to_string()));
                }
                break;
            }
        }
        if let Some((kind, label)) = matched {
            let routeable = request.sensitivity == SensitivityClass::Standard
                && matches!(
                    kind,
                    CaptureCandidateKind::Task
                        | CaptureCandidateKind::Event
                        | CaptureCandidateKind::Project
                        | CaptureCandidateKind::Person
                );
            let mut allowed_actions = vec![
                CaptureCandidateAction::Keep,
                CaptureCandidateAction::Archive,
            ];
            if routeable {
                allowed_actions.push(CaptureCandidateAction::RouteToSpecialist);
                allowed_actions.push(CaptureCandidateAction::ProposeStructuredChange);
            }
            candidates.push(CaptureCandidate {
                id: format!("capture-candidate-{}", index + 1),
                kind,
                label,
                source_excerpt: line.to_string(),
                epistemic_class: EpistemicClass::Inferred,
                allowed_actions,
                unresolved_fields: vec![],
            });
        }
    }

    Ok(CaptureInterpretation {
        intent: request.intent,
        mode: request.mode,
        requires_grouped_confirmation: !candidates.is_empty(),
        candidates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reflective_private_input_can_be_reviewed_without_routing_private_words() {
        let private = CaptureRequest {
            input_kind: CaptureInputKind::Text,
            intent: CaptureIntent::Reflect,
            mode: CaptureMode::Interpret,
            raw_text: "Aufgabe: Kunde anrufen".into(),
            source_label: None,
            sensitivity: SensitivityClass::ReflectivePrivate,
        };
        let candidate = interpret_capture(&private).unwrap().candidates.remove(0);
        assert_eq!(
            candidate.allowed_actions,
            vec![
                CaptureCandidateAction::Keep,
                CaptureCandidateAction::Archive
            ]
        );

        let ordinary = CaptureRequest {
            intent: CaptureIntent::Capture,
            sensitivity: SensitivityClass::Standard,
            ..private
        };
        assert!(interpret_capture(&ordinary).unwrap().candidates[0]
            .allowed_actions
            .contains(&CaptureCandidateAction::RouteToSpecialist));
    }

    #[test]
    fn link_capture_requires_an_http_url_without_embedded_credentials() {
        let mut request = CaptureRequest {
            input_kind: CaptureInputKind::Link,
            intent: CaptureIntent::Capture,
            mode: CaptureMode::Interpret,
            raw_text: "https://example.org/article?part=1".into(),
            source_label: None,
            sensitivity: SensitivityClass::Standard,
        };
        assert!(request.validate().is_ok());
        for invalid in [
            "a note, not a link",
            "file:///private/notes.txt",
            "https://user:secret@example.org/article",
            "https://",
        ] {
            request.raw_text = invalid.into();
            assert!(request.validate().is_err(), "accepted {invalid}");
        }
    }
}
