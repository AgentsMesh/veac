use tempfile::tempdir;
use veac_artifact::{ArtifactStore, ExecutionBindings};
use veac_codegen::emitter::{emit_all, BackendAction};
use veac_ir::StreamChoice;
use veac_plan::resolve_one;
use veac_runtime::executor::execute_bundle;

use super::support::*;

#[test]
fn partial_authority_executes_track_stem_and_plain_caption_end_to_end() {
    let temp = tempdir().unwrap();
    let dialogue = tone_fixture(temp.path(), "scoped-dialogue", 440);
    let music = tone_fixture(temp.path(), "scoped-music", 880);
    let font = font_fixture();
    let mut canonical = project(false);
    canonical.project.render_configs[0].raster = None;
    canonical.project.materials.extend([
        material(
            "med_scoped_dialogue",
            MaterialKind::Audio,
            StreamChoice::Disabled,
            StreamChoice::Auto,
        ),
        material(
            "med_scoped_music",
            MaterialKind::Audio,
            StreamChoice::Disabled,
            StreamChoice::Auto,
        ),
        material(
            "med_scoped_font",
            MaterialKind::Font,
            StreamChoice::Disabled,
            StreamChoice::Disabled,
        ),
    ]);
    let mut dialogue_clip = media_clip("itm_scoped_dialogue", "med_scoped_dialogue", 0, 1_000);
    dialogue_clip.audio = Some(audio_properties(0.4));
    let mut music_clip = media_clip("itm_scoped_music", "med_scoped_music", 0, 1_000);
    music_clip.audio = Some(audio_properties(0.4));
    let mut cue = text_clip(
        "itm_scoped_caption",
        "Scoped caption",
        TextStyle {
            font: FontRef::Material {
                material_id: MaterialId::new("med_scoped_font").unwrap(),
            },
            ..TextStyle::default()
        },
        0,
        1_000,
    );
    let ClipSource::Text { text, style } = cue.source else {
        unreachable!()
    };
    cue.source = ClipSource::Caption {
        text,
        speaker: None,
        cue: Box::default(),
        style,
    };
    cue.visual = Some(full_visual());
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_scoped_dialogue",
            TrackKind::Audio,
            0,
            vec![dialogue_clip],
        ),
        track("trk_scoped_music", TrackKind::Audio, 1, vec![music_clip]),
        track("trk_scoped_caption", TrackKind::Caption, 2, vec![cue]),
    ]);
    canonical.project.render_configs[0].deliverables = deliverables();
    let assets = BTreeMap::from([
        ("med_scoped_dialogue".to_owned(), dialogue.clone()),
        ("med_scoped_music".to_owned(), music),
        ("med_scoped_font".to_owned(), font),
    ]);
    hydrate(&mut canonical, &assets);
    let output_id = canonical.project.render_configs[0].id.clone();
    let plan = resolve_one(&canonical, &output_id).unwrap();
    let mut bindings = ExecutionBindings::default();
    let selected = plan
        .inputs
        .iter()
        .find(|input| {
            input
                .material_id
                .as_ref()
                .is_some_and(|id| id.as_str() == "med_scoped_dialogue")
        })
        .unwrap();
    bindings.bind_original(selected, dialogue).unwrap();
    for deliverable in &plan.output.deliverables {
        let name = deliverable.target.file_name().expect("file deliverable");
        bindings
            .bind_output(deliverable.id.clone(), temp.path().join(name))
            .unwrap();
    }
    let bundle = emit_all(&plan, &bindings).unwrap();
    assert_eq!(bundle.protected_resources().len(), 1);
    let command = bundle.tasks().iter().find_map(|task| match &task.action {
        BackendAction::Ffmpeg(command) => Some(command),
        _ => None,
    });
    assert_eq!(command.unwrap().inputs.len(), 1);

    execute_bundle(&bundle, &ArtifactStore::new(temp.path().join("store"))).unwrap();

    assert!(!audio_samples(&temp.path().join("dialogue.wav"), 0.1, 0.5).is_empty());
    assert!(std::fs::read_to_string(temp.path().join("captions.srt"))
        .unwrap()
        .contains("Scoped caption"));
}

fn deliverables() -> Vec<Deliverable> {
    vec![
        Deliverable {
            id: DeliverableId::new("dlv_scoped_caption").unwrap(),
            target: DeliverableTarget::File {
                name: "captions.srt".to_owned(),
            },
            kind: DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
                format: CaptionSidecarFormat::Srt,
                track_ids: vec![TrackId::new("trk_scoped_caption").unwrap()],
            }),
        },
        Deliverable {
            id: DeliverableId::new("dlv_scoped_dialogue").unwrap(),
            target: DeliverableTarget::File {
                name: "dialogue.wav".to_owned(),
            },
            kind: DeliverableKind::AudioStem(AudioStemOutput {
                format: AudioStemFormat::Wav,
                audio: AudioOutput {
                    codec: AudioCodec::PcmS16Le,
                    sample_rate: 48_000,
                    channels: 1,
                },
                source: AudioMixSource::Track {
                    track_id: TrackId::new("trk_scoped_dialogue").unwrap(),
                },
            }),
        },
    ]
}
