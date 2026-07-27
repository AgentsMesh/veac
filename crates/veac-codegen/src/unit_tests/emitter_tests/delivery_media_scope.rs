use veac_artifact::{ArtifactKind, ArtifactStore, MediaRole, SourceClock};
use veac_codegen::emitter::{emit_all, BackendAction};
use veac_plan::canonical::*;
use veac_plan::{PlanInputId, ResolvedClipSource};

use super::binding_routes::{av_plan, proxy, typed};
use super::support::output_bindings;

#[test]
fn image_and_audio_tasks_receive_only_their_physical_proxy_role() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());

    let mut image = av_plan();
    image.output.deliverables[0].file_name = "frame-%d.png".to_owned();
    image.output.deliverables[0].kind = DeliverableKind::ImageSequence(ImageSequenceOutput {
        format: ImageFormat::Png,
        start_number: 1,
    });
    let image_input = &image.inputs[0];
    let video = proxy(
        &store,
        image_input,
        ArtifactKind::ProxyVideo,
        b"scoped video",
        SourceClock::identity(600).unwrap(),
    );
    let audio = proxy(
        &store,
        image_input,
        ArtifactKind::ProxyAudio,
        b"scoped audio",
        SourceClock::identity(600).unwrap(),
    );
    let mut bindings = typed(&image);
    bindings
        .bind_verified_proxy(image_input, MediaRole::Video, &video)
        .unwrap();
    bindings
        .bind_verified_proxy(image_input, MediaRole::Audio, &audio)
        .unwrap();
    let bundle = emit_all(&image, &bindings).unwrap();
    assert_task_resource(&bundle, video.payload_path());

    let mut stem = av_plan();
    stem.output.deliverables[0].file_name = "master.wav".to_owned();
    stem.output.deliverables[0].kind = DeliverableKind::AudioStem(AudioStemOutput {
        format: AudioStemFormat::Wav,
        audio: AudioOutput {
            codec: AudioCodec::PcmS16Le,
            sample_rate: 48_000,
            channels: 2,
        },
        source: AudioStemSource::Master,
    });
    let stem_input = &stem.inputs[0];
    let video = proxy(
        &store,
        stem_input,
        ArtifactKind::ProxyVideo,
        b"scoped video",
        SourceClock::identity(600).unwrap(),
    );
    let audio = proxy(
        &store,
        stem_input,
        ArtifactKind::ProxyAudio,
        b"scoped audio",
        SourceClock::identity(600).unwrap(),
    );
    let mut bindings = typed(&stem);
    bindings
        .bind_verified_proxy(stem_input, MediaRole::Video, &video)
        .unwrap();
    bindings
        .bind_verified_proxy(stem_input, MediaRole::Audio, &audio)
        .unwrap();
    let bundle = emit_all(&stem, &bindings).unwrap();
    assert_task_resource(&bundle, audio.payload_path());
}

#[test]
fn nested_visual_sequence_binds_only_its_transitive_media_input() {
    let mut plan = av_plan();
    let DeliverableKind::Video(settings) = &mut plan.output.deliverables[0].kind else {
        unreachable!()
    };
    settings.audio = None;
    let mut nested_input = plan.inputs[0].clone();
    nested_input.id = PlanInputId::new("pin_nested_only").unwrap();
    let mut child = plan.sequences.last().unwrap().clone();
    child.id = SequenceId::new("seq_nested_scope").unwrap();
    child.name = "Nested scope".to_owned();
    child.tracks[0].id = TrackId::new("trk_nested_scope").unwrap();
    child.tracks[0].clips[0].id = ItemId::new("itm_nested_scope").unwrap();
    let ResolvedClipSource::Media { input_id, .. } = &mut child.tracks[0].clips[0].source else {
        unreachable!()
    };
    *input_id = nested_input.id.clone();
    plan.sequences.last_mut().unwrap().tracks[0].clips[0].source = ResolvedClipSource::Sequence {
        sequence_id: child.id.clone(),
    };
    plan.sequences.insert(0, child);
    plan.inputs.push(nested_input.clone());
    plan.inputs.sort_by(|left, right| left.id.cmp(&right.id));
    let mut bindings = output_bindings(&plan);
    let path = std::path::PathBuf::from("/tmp/nested-only.mov");
    bindings.bind_original(&nested_input, path.clone()).unwrap();

    let bundle = emit_all(&plan, &bindings).unwrap();

    assert_task_resource(&bundle, &path);
}

#[test]
fn two_pass_first_phase_has_video_only_input_authority() {
    let plan = {
        let mut value = av_plan();
        let DeliverableKind::Video(settings) = &mut value.output.deliverables[0].kind else {
            unreachable!()
        };
        settings.pass_mode = PassMode::TwoPass;
        settings.hardware = HardwareSelection::Software;
        settings.video.rate_control = VideoRateControl::Bitrate {
            target_bps: 1_000_000,
            max_bps: Some(1_500_000),
            buffer_bps: Some(2_000_000),
        };
        value
    };
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let input = &plan.inputs[0];
    let video = proxy(
        &store,
        input,
        ArtifactKind::ProxyVideo,
        b"two pass video",
        SourceClock::identity(600).unwrap(),
    );
    let audio = proxy(
        &store,
        input,
        ArtifactKind::ProxyAudio,
        b"two pass audio",
        SourceClock::identity(600).unwrap(),
    );
    let mut bindings = typed(&plan);
    bindings
        .bind_verified_proxy(input, MediaRole::Video, &video)
        .unwrap();
    bindings
        .bind_verified_proxy(input, MediaRole::Audio, &audio)
        .unwrap();
    let bundle = emit_all(&plan, &bindings).unwrap();
    assert_eq!(bundle.tasks().len(), 2);
    let BackendAction::Ffmpeg(first) = &bundle.tasks()[0].action else {
        panic!("first pass must use FFmpeg")
    };
    let BackendAction::Ffmpeg(second) = &bundle.tasks()[1].action else {
        panic!("second pass must use FFmpeg")
    };
    assert_eq!(first.inputs.len(), 1);
    assert_eq!(first.inputs[0].path, video.payload_path());
    assert_eq!(second.inputs.len(), 2);
    assert_eq!(second.inputs[1].path, audio.payload_path());
}

fn assert_task_resource(bundle: &veac_codegen::emitter::BackendBundle, expected: &std::path::Path) {
    assert_eq!(bundle.protected_resources().len(), 1);
    assert_eq!(bundle.protected_resources()[0].path, expected);
    let BackendAction::Ffmpeg(command) = &bundle.tasks()[0].action else {
        panic!("media delivery must use FFmpeg")
    };
    assert_eq!(command.inputs.len(), 1);
    assert_eq!(command.inputs[0].path, expected);
}
