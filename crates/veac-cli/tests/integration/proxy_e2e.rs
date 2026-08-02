use veac_artifact::{
    ArtifactStore, ContentDigest, DigestAlgorithm, MediaArtifactRequest, MediaArtifactSpec,
    ProxyVideoSpec, SourceClockSpec,
};
use veac_ir::{RationalTime, TimeRange};
use veac_runtime::executor::{FfmpegEnvironment, SystemFfmpeg};
use veac_runtime::workflow::{media_artifact_producer, MediaWorkflow};

use super::support::*;

#[test]
fn automatic_proxy_selection_normalizes_probe_time_and_renders_real_media() {
    let temp = tempdir().unwrap();
    let media = temp.path().join("clip.mp4");
    make_video(&media);
    let project = compile_ir(&temp, MEDIA_SOURCE);
    let output = veac()
        .args(["plan", project.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let plan: veac_plan::ResolvedRenderPlan = serde_json::from_slice(&output.stdout).unwrap();
    let duration = source_duration(&plan);
    assert_ne!(duration.timescale, plan.header.source.timebase);
    let request = proxy_request(&plan);
    let store = ArtifactStore::new(temp.path().join(".veac-artifacts"));
    MediaWorkflow::new("ffmpeg")
        .derive(&store, &media, &request)
        .unwrap();

    veac()
        .args([
            "render",
            project.to_str().unwrap(),
            "--proxy-policy",
            "require",
            "--render-segment-policy",
            "original",
        ])
        .assert()
        .success();
    assert_eq!(
        video_dimensions(&temp.path().join("media-render.mp4")),
        "32x24"
    );
}

#[test]
fn render_requires_semantic_proxy_postflight_and_prefer_falls_back() {
    let temp = tempdir().unwrap();
    let media = temp.path().join("clip.mp4");
    make_video(&media);
    let project = compile_ir(&temp, MEDIA_SOURCE);
    let output = veac()
        .args(["plan", project.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let plan: veac_plan::ResolvedRenderPlan = serde_json::from_slice(&output.stdout).unwrap();
    let request = proxy_request(&plan);
    ArtifactStore::new(temp.path().join(".veac-artifacts"))
        .put(&request.descriptor().unwrap(), b"not a video artifact")
        .unwrap();

    veac()
        .args([
            "render",
            project.to_str().unwrap(),
            "--proxy-policy",
            "require",
            "--render-segment-policy",
            "original",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("PROXY_POSTFLIGHT_FAILED"));
    assert!(!temp.path().join("media-render.mp4").exists());

    veac()
        .args([
            "render",
            project.to_str().unwrap(),
            "--proxy-policy",
            "prefer",
            "--render-segment-policy",
            "original",
        ])
        .assert()
        .success();
    assert_eq!(
        video_dimensions(&temp.path().join("media-render.mp4")),
        "32x24"
    );
}

fn proxy_request(plan: &veac_plan::ResolvedRenderPlan) -> MediaArtifactRequest {
    let input = &plan.inputs[0];
    let stream = input.video.as_ref().unwrap();
    let raster = plan.output.raster.as_ref().unwrap();
    let clock = normalized_clock(
        stream.start_time,
        source_duration(plan),
        plan.header.source.timebase,
    );
    let fingerprint = FfmpegEnvironment::fingerprint(&SystemFfmpeg::default()).unwrap();
    MediaArtifactRequest {
        source_identity: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: input.observed_identity.digest.clone(),
        },
        producer: media_artifact_producer(&fingerprint).unwrap(),
        spec: MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream: stream.selection,
            source_clock: clock,
            width: raster.width,
            height: raster.height,
            frame_rate: raster.frame_rate,
            crf: 28,
        }),
    }
}

fn source_duration(plan: &veac_plan::ResolvedRenderPlan) -> RationalTime {
    let input = &plan.inputs[0];
    input
        .video
        .as_ref()
        .and_then(|value| value.duration)
        .or_else(|| {
            input
                .probe
                .as_ref()
                .and_then(|value| value.container_duration)
        })
        .unwrap()
}

fn normalized_clock(
    start: Option<RationalTime>,
    duration: RationalTime,
    timescale: u32,
) -> SourceClockSpec {
    let duration = rescale(duration, timescale);
    let Some(start) = start.filter(|value| value.value > 0) else {
        return SourceClockSpec::Identity { duration };
    };
    SourceClockSpec::Bounded {
        logical_range: TimeRange::new(rescale(start, timescale), duration).unwrap(),
    }
}

fn rescale(value: RationalTime, timescale: u32) -> RationalTime {
    let numerator = i128::from(value.value) * i128::from(timescale);
    assert_eq!(numerator % i128::from(value.timescale), 0);
    RationalTime::new(
        i64::try_from(numerator / i128::from(value.timescale)).unwrap(),
        timescale,
    )
    .unwrap()
}
