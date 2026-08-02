mod support;

use std::collections::BTreeSet;

use support::*;
use veac_plan::canonical::*;
use veac_plan::{required_material_ids_one, resolve_one};

#[test]
fn nested_repeat_reverse_requires_only_the_intersecting_child_material() {
    let mapping = SourceMapping {
        time_map: SourceTimeMap::Linear {
            source_start: time(0),
            rate: Rational::new(1, 1).unwrap(),
            repeat: 2,
            direction: PlaybackDirection::Reverse,
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: SourceOutOfRangePolicy::Strict,
    };
    assert_exact_nested_closure(mapping, range(50, 100), range(150, 100));
}

#[test]
fn nested_curve_requires_only_the_intersecting_child_segment() {
    let mapping = SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: vec![segment(300, 0, 300), segment(300, 300, 600)],
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: SourceOutOfRangePolicy::Strict,
    };
    assert_exact_nested_closure(mapping, range(350, 50), range(350, 50));
}

fn assert_exact_nested_closure(
    mapping: SourceMapping,
    sidechain_range: TimeRange,
    hit_range: TimeRange,
) {
    let envelope = fixture(mapping, sidechain_range, hit_range);
    let config_id = envelope.project.render_configs[0].id.clone();
    let required = required_material_ids_one(&envelope, &config_id).unwrap();
    assert_eq!(
        required,
        BTreeSet::from([
            MaterialId::new("med_child_hit").unwrap(),
            MaterialId::new("med_video").unwrap(),
        ]),
    );
    let plan = resolve_one(&envelope, &config_id).unwrap();
    let planned: BTreeSet<_> = plan
        .inputs
        .iter()
        .filter_map(|input| input.material_id.clone())
        .collect();
    assert_eq!(planned, required);
    assert!(plan
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
        .all(|clip| clip.id.as_str() != "itm_child_miss"));
}

fn fixture(
    mapping: SourceMapping,
    sidechain_range: TimeRange,
    hit_range: TimeRange,
) -> ProjectEnvelope {
    let mut envelope = project();
    let config = &mut envelope.project.render_configs[0];
    config.raster = None;
    config.deliverables = vec![Deliverable {
        id: DeliverableId::new("dlv_audio").unwrap(),
        target: DeliverableTarget::File {
            name: "audio.wav".to_owned(),
        },
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format: AudioStemFormat::Wav,
            audio: AudioOutput {
                codec: AudioCodec::PcmS16Le,
                sample_rate: 48_000,
                channels: 2,
            },
            source: AudioMixSource::Track {
                track_id: TrackId::new("trk_video").unwrap(),
            },
        }),
    }];
    envelope.project.sequences[0].tracks[0].clips[0].audio = Some(audio_properties());

    let mut nested = generated_clip("itm_nested_key", Generator::Silence, 0);
    nested.record_range = range(0, 600);
    nested.source = ClipSource::Sequence {
        sequence_id: SequenceId::new("seq_child").unwrap(),
    };
    nested.source_mapping = Some(mapping);
    nested.audio = Some(audio_properties());
    let key = track("trk_key", TrackKind::Audio, 1, vec![nested]);
    envelope.project.sequences[0].tracks.push(key);

    let mut hit = media_clip("itm_child_hit", "med_child_hit", hit_range.start.value);
    hit.record_range = hit_range;
    hit.audio = Some(audio_properties());
    let mut miss = media_clip("itm_child_miss", "med_child_miss", 550);
    miss.record_range = range(550, 50);
    miss.audio = Some(audio_properties());
    envelope.project.sequences.push(sequence(
        "seq_child",
        vec![track("trk_child", TrackKind::Audio, 0, vec![hit, miss])],
    ));

    envelope
        .project
        .materials
        .push(audio_material("med_child_hit"));
    let mut missing = audio_material("med_child_miss");
    missing.source = MaterialSource::File {
        uri: "media/definitely-missing-unselected.wav".to_owned(),
    };
    missing.identity = None;
    missing.probe = None;
    envelope.project.materials.push(missing);
    add_sidechain(
        &mut envelope,
        "rel_nested_key",
        "seq_main",
        RelationEndpoint::track(TrackId::new("trk_key").unwrap()),
        "itm_video",
        SidechainRelationParameters {
            threshold_db: -18.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 200.0,
            active_range: Some(sidechain_range),
        },
    );
    envelope
}

fn segment(duration: i64, start: i64, end: i64) -> SourceTimeSegment {
    SourceTimeSegment {
        record_duration: time(duration),
        source_start: time(start),
        source_end: time(end),
        interpolation: SourceTimeInterpolation::Linear,
    }
}
