#![allow(dead_code, unused_imports)]

mod applies;
mod media;
mod properties;
mod relation_assertions;
mod relations;
mod timeline;

pub use applies::*;
pub use media::*;
pub use properties::*;
pub use relation_assertions::*;
pub use relations::*;
pub use timeline::*;

use veac_plan::{canonical::*, ResolutionErrors};

pub fn project() -> ProjectEnvelope {
    let mut envelope: ProjectEnvelope = serde_json::from_str(include_str!(
        "../../../veac-ir/tests/fixtures/minimal-project.json"
    ))
    .expect("fixture is valid JSON");
    let identity = identity('a');
    envelope.project.materials[0].identity = Some(identity.clone());
    envelope.project.materials[0].probe = Some(video_probe(identity));
    envelope
}

pub fn diagnostic_codes(errors: &ResolutionErrors) -> Vec<&str> {
    errors
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

pub fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 600).expect("fixture time")
}

pub fn range(start: i64, duration: i64) -> TimeRange {
    TimeRange::new(time(start), time(duration)).expect("fixture range")
}
