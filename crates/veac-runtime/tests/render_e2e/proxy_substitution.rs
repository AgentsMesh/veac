use tempfile::tempdir;
use veac_artifact::*;
use veac_codegen::emitter::{emit_all, BackendAction, BackendProduct};
use veac_ir::StreamChoice;
use veac_runtime::executor::{self, FfmpegEnvironment, SystemFfmpeg};
use veac_runtime::workflow::{media_artifact_producer, MediaWorkflow};

use super::support::*;

#[test]
fn split_bounded_av_proxies_render_through_verified_physical_inputs() {
    let temp = tempdir().unwrap();
    let source = retime_fixture(temp.path());
    let mut canonical = project(true);
    canonical.project.materials.push(material(
        "med_source",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Auto,
    ));
    let mut video = media_clip("itm_video", "med_source", 0, 1_000);
    video.source_mapping = Some(SourceMapping::linear(time(1_000), ratio(1, 1)));
    let mut audio = media_clip("itm_audio", "med_source", 0, 1_000);
    audio.source_mapping = Some(SourceMapping::linear(time(1_000), ratio(1, 1)));
    audio.audio = Some(audio_properties(1.0));
    canonical.project.sequences[0].tracks.extend([
        track("trk_video", TrackKind::Video, 0, vec![video]),
        track("trk_audio", TrackKind::Audio, 1, vec![audio]),
    ]);
    let assets = BTreeMap::from([("med_source".to_owned(), source.clone())]);
    hydrate(&mut canonical, &assets);
    let output_id = canonical.project.render_configs[0].id.clone();
    let plan = veac_plan::resolve_one(&canonical, &output_id).unwrap();
    let input = &plan.inputs[0];
    let source_identity = ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: input.observed_identity.digest.clone(),
    };
    let clock = SourceClockSpec::Bounded {
        logical_range: TimeRange::new(time(1_000), time(1_000)).unwrap(),
    };
    let fingerprint = FfmpegEnvironment::fingerprint(&SystemFfmpeg::default()).unwrap();
    let producer = media_artifact_producer(&fingerprint).unwrap();
    let video_request = request(
        source_identity.clone(),
        producer.clone(),
        MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream: input.video.as_ref().unwrap().selection,
            source_clock: clock,
            width: WIDTH,
            height: HEIGHT,
            frame_rate: ratio(FPS, 1),
            crf: 23,
        }),
    );
    let audio_request = request(
        source_identity.clone(),
        producer,
        MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: input.audio.as_ref().unwrap().selection,
            source_clock: clock,
            sample_rate: 48_000,
            channels: 1,
        }),
    );
    let store = ArtifactStore::new(temp.path().join("store"));
    let workflow = MediaWorkflow::new("ffmpeg");
    workflow.derive(&store, &source, &video_request).unwrap();
    workflow.derive(&store, &source, &audio_request).unwrap();
    let selection = select_proxy(
        &store,
        &ProxySelectionRequest {
            source_identity,
            video: Some(video_request.descriptor().unwrap()),
            audio: Some(audio_request.descriptor().unwrap()),
        },
    )
    .unwrap();
    let video_proxy = selection.video.as_ref().unwrap().payload_path().to_owned();
    let audio_proxy = selection.audio.as_ref().unwrap().payload_path().to_owned();
    let paths = BTreeMap::from([(input.id.clone(), source)]);
    let mut bindings = ExecutionBindings::from_originals(&plan, &paths).unwrap();
    bindings.bind_proxy_selection(input, &selection).unwrap();
    let output = temp.path().join("proxy-render.mp4");
    bindings
        .bind_output(plan.output.deliverables[0].id.clone(), output.clone())
        .unwrap();
    let bundle = emit_all(&plan, &bindings).unwrap();
    let command = bundle
        .tasks()
        .iter()
        .find_map(|task| match (&task.action, task.product) {
            (BackendAction::Ffmpeg(command), BackendProduct::VideoMaster) => Some(command),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        command
            .inputs
            .iter()
            .map(|value| &value.path)
            .collect::<Vec<_>>(),
        [&video_proxy, &audio_proxy]
    );
    let graph = command.filter_graph.as_deref().unwrap();
    assert!(graph.contains("trim=start=0:duration=1"), "{graph}");
    assert!(graph.contains("atrim=start=0:duration=1"), "{graph}");
    executor::execute_bundle(&bundle, &store).unwrap();
    assert_media_contract(&output, 1, 1.0);
    let center = rgb_at(&output, 0.5, WIDTH / 2, HEIGHT / 2);
    assert!(
        center[1] > 70 && center[0] < 40 && center[2] < 40,
        "{center:?}"
    );
}

fn request(
    source_identity: ContentDigest,
    producer: ProducerFingerprint,
    spec: MediaArtifactSpec,
) -> MediaArtifactRequest {
    MediaArtifactRequest {
        source_identity,
        producer,
        spec,
    }
}
