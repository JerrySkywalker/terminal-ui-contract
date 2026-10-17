//! Product-cleared semantic facts. Smart and Raw borrow the exact same facts.
//! No raw provider stdout, shell command, transcript or secret ingestion API.
use crate::keymap::{ActionEffect, ActionIntent};
use std::{collections::BTreeSet, fmt, time::SystemTime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Verdict {
    Pass,
    StaticPass,
    Fail,
    Unproven,
    Blocked,
    Stale,
    Unknown,
}
impl Verdict {
    pub fn marker(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::StaticPass => "STATIC_PASS",
            Self::Fail => "FAIL",
            Self::Unproven => "UNPROVEN",
            Self::Blocked => "BLOCKED",
            Self::Stale => "STALE",
            Self::Unknown => "UNKNOWN",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Information,
    Attention,
    Error,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactFreshness {
    Current,
    Historical,
    Unknown,
}

/// Product redaction is an explicit trust boundary. It must remove credentials,
/// commands and private payloads before construction; TUC cannot infer secrecy
/// from arbitrary strings or relabel a real provider's authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PermittedText(String);
impl PermittedText {
    pub fn from_product_redaction(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceField {
    pub label: String,
    pub value: PermittedText,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SafeNextStep {
    pub label: String,
    pub intent: ActionIntent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiFact {
    pub id: String,
    pub label: String,
    pub verdict: Verdict,
    pub severity: Severity,
    pub source: String,
    pub scope: String,
    pub sampled_at: SystemTime,
    pub freshness: FactFreshness,
    pub summary: PermittedText,
    pub impact: PermittedText,
    pub evidence: Vec<EvidenceField>,
    pub next_steps: Vec<SafeNextStep>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiDocument {
    revision: u64,
    sampled_at: SystemTime,
    facts: Vec<UiFact>,
}
impl UiDocument {
    pub fn new(
        revision: u64,
        sampled_at: SystemTime,
        facts: Vec<UiFact>,
    ) -> Result<Self, DocumentError> {
        let mut ids = BTreeSet::new();
        for fact in &facts {
            if fact.id.is_empty()
                || fact.source.is_empty()
                || fact.scope.is_empty()
                || !ids.insert(&fact.id)
            {
                return Err(DocumentError(
                    "facts require unique IDs, source and scope".into(),
                ));
            }
            if fact
                .next_steps
                .iter()
                .any(|step| step.intent.effect == ActionEffect::ProductWriteRequest)
            {
                return Err(DocumentError(
                    "presentation next step cannot acquire write authority".into(),
                ));
            }
        }
        Ok(Self {
            revision,
            sampled_at,
            facts,
        })
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn sampled_at(&self) -> SystemTime {
        self.sampled_at
    }
    pub fn facts(&self) -> &[UiFact] {
        &self.facts
    }
    pub fn project(&self, density: ViewDensity) -> DocumentProjection<'_> {
        DocumentProjection {
            document: self,
            density,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentError(pub String);
impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for DocumentError {}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ViewDensity {
    #[default]
    Smart,
    Raw,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ViewState {
    pub density: ViewDensity,
}
impl ViewState {
    pub fn toggle(&mut self) {
        self.density = match self.density {
            ViewDensity::Smart => ViewDensity::Raw,
            ViewDensity::Raw => ViewDensity::Smart,
        };
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DocumentProjection<'a> {
    document: &'a UiDocument,
    pub density: ViewDensity,
}
impl<'a> DocumentProjection<'a> {
    pub fn revision(&self) -> u64 {
        self.document.revision()
    }
    /// Both projections expose all facts, retaining fail/unknown/blocked and stable identity.
    pub fn facts(&self) -> &'a [UiFact] {
        self.document.facts()
    }
    pub fn lines(&self) -> Vec<String> {
        let mut lines = vec![];
        for fact in self.document.facts() {
            lines.push(format!(
                "[{}] {} ({})",
                fact.verdict.marker(),
                fact.label,
                fact.id
            ));
            lines.push(format!(
                "{} | source={} | scope={} | {:?}",
                fact.summary.as_str(),
                fact.source,
                fact.scope,
                fact.freshness
            ));
            lines.push(format!("Impact: {}", fact.impact.as_str()));
            for step in &fact.next_steps {
                lines.push(format!("Next safe step: {}", step.label));
            }
            if self.density == ViewDensity::Raw {
                lines.push(format!(
                    "sampled_at={:?}; severity={:?}",
                    fact.sampled_at, fact.severity
                ));
                for field in &fact.evidence {
                    lines.push(format!("{}: {}", field.label, field.value.as_str()));
                }
            }
            lines.push(String::new());
        }
        lines
    }
}
