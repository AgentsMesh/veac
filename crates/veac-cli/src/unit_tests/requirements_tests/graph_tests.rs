use std::collections::BTreeSet;

use tempfile::tempdir;
use veac_ir::{
    AudioCodec, AudioOutput, AudioStemFormat, AudioStemOutput, AudioStemSource, ClipSource,
    ColorPipeline, ColorSpace, ColorStage, Deliverable, DeliverableId, DeliverableKind,
    LutApplication, LutInterpolation, MaterialId, MulticamAngle, MulticamAngleId, MulticamGroup,
    MulticamGroupId, MulticamSync, MulticamSyncBasis, RationalTime, RenderConfigId, TrackKind,
};

use crate::unit_tests::support::{canonical_project, MEDIA_SOURCE};

#[test]
fn audio_stem_multicam_and_lut_sources_are_reachable() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    let footage = envelope.project.materials[0].id.clone();
    let mut voice = envelope.project.sequences[0].tracks[0].clone();
    voice.kind = TrackKind::Audio;
    voice.clips[0].source = ClipSource::Media {
        material_id: MaterialId::new("med_voice").unwrap(),
    };
    envelope.project.sequences[0].tracks.push(voice);
    envelope.project.render_configs[0]
        .deliverables
        .push(audio_stem());
    assert_eq!(
        required(&envelope),
        BTreeSet::from([footage.clone(), MaterialId::new("med_voice").unwrap()])
    );

    let group_id = MulticamGroupId::new("mcg_main").unwrap();
    envelope
        .project
        .multicam_groups
        .push(multicam_group(&group_id));
    let clip = &mut envelope.project.sequences[0].tracks[0].clips[0];
    clip.source = ClipSource::Multicam {
        group_id,
        switches: vec![],
    };
    clip.source_mapping = None;
    let mut visual = clip.visual.take().unwrap();
    visual.color_pipeline = Some(ColorPipeline {
        input: color_space(),
        working: color_space(),
        output: color_space(),
        stages: vec![ColorStage::Lut {
            application: LutApplication {
                material_id: MaterialId::new("med_lut").unwrap(),
                interpolation: LutInterpolation::Tetrahedral,
            },
        }],
    });
    clip.visual = Some(visual);
    assert_eq!(
        required(&envelope),
        BTreeSet::from([
            MaterialId::new("med_angle_a").unwrap(),
            MaterialId::new("med_angle_b").unwrap(),
            MaterialId::new("med_lut").unwrap(),
            MaterialId::new("med_voice").unwrap(),
        ])
    );
}

fn required(envelope: &veac_ir::ProjectEnvelope) -> BTreeSet<MaterialId> {
    crate::requirements::collect(envelope, &RenderConfigId::new("out_main").unwrap()).unwrap()
}

fn audio_stem() -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_audio").unwrap(),
        file_name: "mix.wav".into(),
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format: AudioStemFormat::Wav,
            audio: AudioOutput {
                codec: AudioCodec::PcmS16Le,
                sample_rate: 48_000,
                channels: 2,
            },
            source: AudioStemSource::Master,
        }),
    }
}

fn multicam_group(id: &MulticamGroupId) -> MulticamGroup {
    let first = MulticamAngleId::new("ang_a").unwrap();
    MulticamGroup {
        id: id.clone(),
        sync: MulticamSync {
            basis: MulticamSyncBasis::Manual,
            reference_angle_id: first.clone(),
        },
        angles: vec![
            MulticamAngle {
                id: first,
                material_id: MaterialId::new("med_angle_a").unwrap(),
                source_offset: RationalTime::zero(600).unwrap(),
            },
            MulticamAngle {
                id: MulticamAngleId::new("ang_b").unwrap(),
                material_id: MaterialId::new("med_angle_b").unwrap(),
                source_offset: RationalTime::zero(600).unwrap(),
            },
        ],
    }
}

fn color_space() -> ColorSpace {
    serde_json::from_value(serde_json::json!({
        "primaries": "bt709", "transfer": "bt709", "matrix": "bt709", "range": "limited"
    }))
    .unwrap()
}
