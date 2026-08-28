//! Report-safe boundary: internal session/state → allow-listed projection →
//! defence-in-depth sanitizer → human-readable preview. No file export.
//!
//! The allow-list is the privacy boundary: only deliberately approved fields
//! cross from internal state into the report-safe model. The sanitizer is a
//! second, defence-in-depth layer — its job is to catch unsafe-looking content
//! that reaches a report-safe textual field unexpectedly, not to define what
//! is private. Sanitizer success must not substitute for allow-list design.
//! Semantics per design §8.

use crate::identity::Unit;
use crate::outcome::Outcome;
use crate::provenance::Provenance;
use crate::session::{Snapshot, SnapshotEntry};
use crate::support::Support;
use crate::value::{LimitationCode, SampleValue, Timestamp};

/// Metric ids deliberately approved to appear in a shareable report. Anything
/// not listed here is dropped by [`project`] and never reaches the report-safe
/// model, even if it carried only safe values.
pub const REPORT_ALLOWED_METRICS: &[&str] = &[
    "os.ram.available",
    "os.ram.used",
    "os.swap.used",
    "os.cpu.load",
    "os.disk.used",
];

const REDACTED: &str = "[redacted]";

/// The report-safe status of a metric. A projection of the internal outcome:
/// `Ok` → `Available`, `NoValue` → `NotAvailable`, `Failed` → `Failed`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReportStatus {
    Available,
    NotAvailable,
    Failed,
}

impl ReportStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::NotAvailable => "not available",
            Self::Failed => "failed",
        }
    }
}

/// The six provenance categories, as a payload-free report projection. The
/// internal `Calculated`/`Estimated` inputs, formula, and assumptions do not
/// cross into the report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReportProvenance {
    OperatingSystemReported,
    DriverReported,
    RuntimeReported,
    ApplicationMeasured,
    Calculated,
    Estimated,
}

/// A numeric report value. Text-valued metrics are not in the allow-list, so a
/// report value is always a number with a unit.
#[derive(Clone, Debug, PartialEq)]
pub struct ReportValue {
    pub amount: f64,
    pub unit: Unit,
}

/// One metric's report-safe projection. Carries the display name (not the
/// internal id), a status, an optional value, an optional provenance (present
/// only when a value exists), and controlled limitation codes. There is no
/// raw error field and no internal id field.
#[derive(Clone, Debug, PartialEq)]
pub struct ReportEntry {
    pub display_name: String,
    pub status: ReportStatus,
    pub value: Option<ReportValue>,
    pub provenance: Option<ReportProvenance>,
    pub limitation_codes: Vec<LimitationCode>,
}

/// A report-safe snapshot: a time and the projected, allow-listed entries.
#[derive(Clone, Debug, PartialEq)]
pub struct ReportSnapshot {
    pub at: Timestamp,
    pub entries: Vec<ReportEntry>,
}

/// Project an internal [`Snapshot`] into a report-safe [`ReportSnapshot`].
/// Non-allow-listed metrics are dropped entirely. `Ok` maps to `Available` with
/// a numeric value and provenance; `NoValue` to `NotAvailable`; `Failed` to
/// `Failed` with no raw error. The display name (not the internal id) and
/// controlled limitation codes are the only textual content that crosses.
pub fn project(snapshot: &Snapshot) -> ReportSnapshot {
    let entries = snapshot
        .entries
        .iter()
        .filter(|e| REPORT_ALLOWED_METRICS.contains(&e.descriptor.id.as_str()))
        .map(project_entry)
        .collect();
    ReportSnapshot {
        at: snapshot.at,
        entries,
    }
}

fn project_entry(entry: &SnapshotEntry) -> ReportEntry {
    let display_name = entry.descriptor.display_name.clone();
    let (status, value, provenance, limitation_codes) = match &entry.result.support {
        Support::Unsupported { limitation } => (
            ReportStatus::NotAvailable,
            None,
            None,
            vec![limitation.clone()],
        ),
        Support::Supported => match &entry.result.outcome {
            Outcome::Ok(sample) => (
                ReportStatus::Available,
                Some(ReportValue {
                    amount: sample_value_to_f64(&sample.value),
                    unit: sample.unit.clone(),
                }),
                Some(map_provenance(&sample.provenance)),
                sample.limitations.clone(),
            ),
            Outcome::NoValue => (ReportStatus::NotAvailable, None, None, Vec::new()),
            Outcome::Failed(_) => (ReportStatus::Failed, None, None, Vec::new()),
        },
    };
    ReportEntry {
        display_name,
        status,
        value,
        provenance,
        limitation_codes,
    }
}

fn map_provenance(p: &Provenance) -> ReportProvenance {
    match p {
        Provenance::OperatingSystemReported => ReportProvenance::OperatingSystemReported,
        Provenance::DriverReported => ReportProvenance::DriverReported,
        Provenance::RuntimeReported => ReportProvenance::RuntimeReported,
        Provenance::ApplicationMeasured => ReportProvenance::ApplicationMeasured,
        Provenance::Calculated { .. } => ReportProvenance::Calculated,
        Provenance::Estimated { .. } => ReportProvenance::Estimated,
    }
}

fn sample_value_to_f64(v: &SampleValue) -> f64 {
    match v {
        SampleValue::Integer(i) => *i as f64,
        SampleValue::Float(f) => *f,
        SampleValue::Count(c) => *c as f64,
    }
}

/// Defence-in-depth: redact any report-safe textual field that looks unsafe.
/// This is a backstop — the allow-list projection is the primary boundary. Any
/// future `String` field on a report-safe type must be passed through
/// [`redact_string`].
pub fn sanitize(snapshot: &mut ReportSnapshot) {
    for entry in &mut snapshot.entries {
        entry.display_name = redact_string(&entry.display_name);
    }
}

/// Replace a string with `[redacted]` if it looks like an absolute path, a
/// home-directory path, an email/user@host, an IPv4 address, or a hostname with
/// a local-network suffix. Otherwise return it unchanged.
fn redact_string(s: &str) -> String {
    if looks_unsafe(s) {
        String::from(REDACTED)
    } else {
        s.to_string()
    }
}

fn looks_unsafe(s: &str) -> bool {
    s.starts_with('/')
        || s.contains('\\')
        || s.contains('@')
        || s.contains("/home/")
        || s.contains("/Users/")
        || looks_like_ipv4(s)
        || has_hostname_suffix(s)
}

const HOSTNAME_SUFFIXES: &[&str] = &[
    ".local",
    ".lan",
    ".internal",
    ".home",
    ".corp",
    ".arpa",
    ".private",
];

fn has_hostname_suffix(s: &str) -> bool {
    HOSTNAME_SUFFIXES.iter().any(|suffix| s.contains(suffix))
}

/// Detect a substring shaped like an IPv4 address (four 1-3 digit groups
/// separated by dots), without a regex dependency.
fn looks_like_ipv4(s: &str) -> bool {
    let bytes = s.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    while i < n {
        if bytes[i].is_ascii_digit() {
            let mut j = i;
            let mut groups = 0;
            loop {
                let run_start = j;
                while j < n && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                let dlen = j - run_start;
                if dlen == 0 || dlen > 3 {
                    break;
                }
                groups += 1;
                if groups == 4 {
                    return true;
                }
                if j >= n || bytes[j] != b'.' {
                    break;
                }
                j += 1; // consume the dot
            }
        }
        i += 1;
    }
    false
}

const MAX_EXACT_PRESENTATION_INTEGER: f64 = 9_007_199_254_740_991.0;
const DECIMAL_BYTE_UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB", "EB"];

/// Render a structured human-readable plain-text preview. The report contains
/// only the already-projected report-safe entries and controlled messages; it
/// does not render the snapshot timestamp or create an export format.
pub fn render_preview(snapshot: &ReportSnapshot) -> String {
    let mut out = String::from("AI Engine Room — Observation Report\n");
    for entry in &snapshot.entries {
        out.push_str("\nObservation\n");
        out.push_str(&entry.display_name);
        out.push_str("\nStatus: ");
        out.push_str(status_label(&entry.status));
        out.push('\n');
        render_value(&mut out, entry.value.as_ref());
        if let Some(provenance) = &entry.provenance {
            out.push_str("Source: ");
            out.push_str(provenance_label(provenance));
            out.push('\n');
        }

        let available_memory_os_observation = entry.display_name == "Available memory"
            && entry.provenance.as_ref() == Some(&ReportProvenance::OperatingSystemReported);
        if available_memory_os_observation || !entry.limitation_codes.is_empty() {
            out.push_str("\nQualifications\n");
            if available_memory_os_observation {
                out.push_str("- Available memory is an operating-system observation.\n");
                out.push_str("- Operating systems define available memory differently; values from different platforms are not claimed to be numerically equivalent.\n");
            }
            for limitation in &entry.limitation_codes {
                out.push_str("- ");
                out.push_str(limitation.message());
                out.push('\n');
            }
        }
    }

    out.push_str("\nScope\n");
    out.push_str("- This report contains only allow-listed report-safe observations.\n");
    out.push_str("- Available memory does not establish memory pressure, model fit, headroom, or compute placement.\n");
    out
}

fn status_label(status: &ReportStatus) -> &'static str {
    match status {
        ReportStatus::Available => "Available",
        ReportStatus::NotAvailable => "Not available",
        ReportStatus::Failed => "Failed",
    }
}

fn render_value(out: &mut String, value: Option<&ReportValue>) {
    let Some(value) = value else {
        out.push_str("Value: No value reported\n");
        return;
    };

    if value.unit == Unit::Bytes {
        let Some(friendly) = format_friendly_bytes(value.amount) else {
            out.push_str("Value: No value reported\n");
            out.push_str("Exact value: Not representable exactly in this presentation\n");
            return;
        };
        if is_exact_byte_count(value.amount) {
            out.push_str("Value: ");
            out.push_str(&friendly);
            out.push('\n');
            out.push_str("Exact value: ");
            out.push_str(&format_exact_bytes(value.amount));
            out.push('\n');
        } else {
            out.push_str("Value: ≈");
            out.push_str(&friendly);
            out.push('\n');
            out.push_str("Exact value: Not representable exactly in this presentation\n");
        }
        return;
    }

    if value.amount.is_finite() {
        out.push_str("Value: ");
        out.push_str(&value.amount.to_string());
        out.push(' ');
        out.push_str(unit_label(&value.unit));
        out.push('\n');
    } else {
        out.push_str("Value: No value reported\n");
    }
}

fn format_friendly_bytes(bytes: f64) -> Option<String> {
    if !bytes.is_finite() || bytes < 0.0 {
        return None;
    }
    if bytes < 1000.0 {
        return if bytes.fract() == 0.0 {
            Some(format!("{bytes:.0} B"))
        } else {
            Some(format!("{bytes:.2} B"))
        };
    }

    let mut tier = 0;
    let mut scaled = bytes;
    while scaled >= 1000.0 && tier < DECIMAL_BYTE_UNITS.len() - 1 {
        scaled /= 1000.0;
        tier += 1;
    }
    let formatted = if scaled >= 100.0 {
        format!("{scaled:.0}")
    } else {
        format!("{scaled:.2}")
    };
    Some(format!("{formatted} {}", DECIMAL_BYTE_UNITS[tier]))
}

fn is_exact_byte_count(value: f64) -> bool {
    value.is_finite()
        && (0.0..=MAX_EXACT_PRESENTATION_INTEGER).contains(&value)
        && value.fract() == 0.0
}

fn format_exact_bytes(value: f64) -> String {
    let integer = value as u64;
    let digits = integer.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(character);
    }
    let unit = if integer == 1 { "byte" } else { "bytes" };
    format!("{grouped} {unit}")
}

fn provenance_label(provenance: &ReportProvenance) -> &'static str {
    match provenance {
        ReportProvenance::OperatingSystemReported => "Operating system",
        ReportProvenance::DriverReported => "Hardware driver",
        ReportProvenance::RuntimeReported => "AI runtime",
        ReportProvenance::ApplicationMeasured => "AI Engine Room",
        ReportProvenance::Calculated => "Calculated from other metrics",
        ReportProvenance::Estimated => "Estimate",
    }
}

fn unit_label(u: &Unit) -> &'static str {
    match u {
        Unit::Bytes => "bytes",
        Unit::Count => "count",
        Unit::Percent => "percent",
        Unit::Seconds => "seconds",
    }
}
