mod output_helpers;
mod outputs;
mod requests;

pub(crate) use output_helpers::*;
pub(crate) use outputs::*;
pub(crate) use requests::*;

use serde_json::json;
use veac_artifact::{artifact_key, ArtifactKind, ArtifactRecord, ContentDigest};
use veac_ir::{RationalTime, Rect, TimeRange};

use crate::*;

pub(crate) fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 100).unwrap()
}

pub(crate) fn range(start: i64, duration: i64) -> TimeRange {
    TimeRange::new(time(start), time(duration)).unwrap()
}

pub(crate) fn rect() -> Rect {
    Rect {
        x: 0.1,
        y: 0.1,
        width: 0.5,
        height: 0.5,
    }
}

pub(crate) fn input(media_type: MediaType) -> InputArtifact {
    InputArtifact {
        content: ContentDigest::sha256(format!("{media_type:?}")),
        media_type,
        stream_index: Some(0),
        range: Some(range(0, 100)),
    }
}

pub(crate) fn fingerprint() -> ProviderFingerprint {
    ProviderFingerprint {
        provider: "fixture-provider".to_owned(),
        implementation_version: "2.1.0".to_owned(),
        model: "fixture-model".to_owned(),
        model_version: "2026-07".to_owned(),
        configuration: ContentDigest::sha256(b"fixed configuration"),
    }
}

pub(crate) fn negotiated(capability: Capability) -> NegotiatedCapability {
    NegotiatedCapability {
        capability,
        contract_version: CAPABILITY_CONTRACT_VERSION,
        provider: fingerprint(),
    }
}

pub(crate) fn artifact(
    kind: ArtifactKind,
    role: &str,
    provider: &ProviderFingerprint,
    request: ContentDigest,
) -> ProviderArtifact {
    let descriptor = provider_artifact_descriptor(kind, provider, request, vec![], json!({}))
        .expect("fixture descriptor");
    ProviderArtifact {
        role: role.to_owned(),
        record: ArtifactRecord {
            key: artifact_key(&descriptor).unwrap(),
            content: ContentDigest::sha256(role.as_bytes()),
            size_bytes: role.len() as u64,
        },
        descriptor,
    }
}

pub(crate) fn decision() -> ReviewDecision {
    ReviewDecision {
        id: "decision-1".to_owned(),
        range: range(0, 10),
        action: "review".to_owned(),
        rationale: "provider evidence".to_owned(),
        confidence: 0.9,
    }
}

pub(crate) fn scalar() -> ScalarSample {
    ScalarSample {
        time: time(0),
        value: 0.5,
        confidence: Some(0.8),
    }
}

pub(crate) fn crop() -> CropSample {
    CropSample {
        time: time(0),
        rect: rect(),
        confidence: 0.9,
    }
}
