use std::collections::BTreeMap;
use std::path::Path;
use veac_ir::*;
use veac_plan::{resolve_one, ResolvedRenderPlan};

use crate::*;

pub fn plan(payload: &[u8]) -> ResolvedRenderPlan {
    let project = project(payload);
    resolve_one(&project, &RenderConfigId::new("out_main").unwrap()).expect("resolved plan")
}

pub fn project(payload: &[u8]) -> ProjectEnvelope {
    let mut project: ProjectEnvelope = serde_json::from_str(include_str!(
        "../../veac-ir/tests/fixtures/minimal-project.json"
    ))
    .expect("canonical fixture");
    let identity = media_identity(payload);
    project.project.materials[0].identity = Some(identity.clone());
    project.project.materials[0].probe = Some(probe(identity));
    project
}

pub fn media_identity(payload: &[u8]) -> MediaIdentity {
    MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: ContentDigest::sha256(payload).value,
    }
}

pub fn original_bindings(plan: &ResolvedRenderPlan, path: &Path) -> ExecutionBindings {
    let paths = plan
        .inputs
        .iter()
        .map(|input| (input.id.clone(), path.to_owned()))
        .collect::<BTreeMap<_, _>>();
    ExecutionBindings::from_originals(plan, &paths).expect("typed original bindings")
}

pub fn descriptor() -> ArtifactDescriptor {
    ArtifactDescriptor::new(
        producer(),
        vec![ArtifactDependency::new(
            ArtifactDependencyRole::Source,
            ContentDigest::sha256(b"source"),
        )],
        ArtifactParameters::ProxyVideo(ProxyVideoSpec {
            source_stream: StreamSelection {
                global_index: 0,
                type_index: 0,
            },
            source_clock: SourceClockSpec::Identity {
                duration: RationalTime::new(6_000, 600).unwrap(),
            },
            width: 1280,
            height: 720,
            frame_rate: Rational::new(30, 1).unwrap(),
            crf: 24,
        }),
    )
}

pub fn producer() -> ProducerFingerprint {
    ProducerFingerprint {
        name: "veac-proxy".to_owned(),
        version: "1.0.0".to_owned(),
        configuration: ContentDigest::sha256(b"configuration"),
    }
}

fn probe(identity: MediaIdentity) -> MediaProbeSnapshot {
    let duration = RationalTime::new(6_000, 600).unwrap();
    MediaProbeSnapshot {
        schema_version: MEDIA_PROBE_SCHEMA_VERSION,
        engine: "ffprobe-8.0".to_owned(),
        selection_policy: "veac-stream-selection-v1".to_owned(),
        container_format: "mov,mp4,m4a,3gp,3g2,mj2".to_owned(),
        container_brand: Some("isom".to_owned()),
        observed_identity: identity,
        container_duration: Some(duration),
        streams: vec![ProbedStream {
            global_index: 0,
            type_index: 0,
            media_type: ProbedStreamType::Video,
            codec: "h264".to_owned(),
            time_base: Some(Rational::new(1, 600).unwrap()),
            start_time: None,
            duration: Some(duration),
            disposition: StreamDisposition {
                default: true,
                attached_picture: false,
                timed_thumbnail: false,
            },
            video: Some(VideoStreamInfo {
                width: 1920,
                height: 1080,
                frame_rate: Some(Rational::new(30, 1).unwrap()),
                cadence: VideoCadence::Constant,
                pixel_format: "yuv420p".to_owned(),
                profile: Some("High".to_owned()),
                level: Some(40),
                sample_aspect_ratio: Rational::new(1, 1).unwrap(),
                rotation_degrees: 0,
            }),
            audio: None,
        }],
        selected_video_stream: Some(StreamSelection {
            global_index: 0,
            type_index: 0,
        }),
        selected_audio_stream: None,
    }
}
