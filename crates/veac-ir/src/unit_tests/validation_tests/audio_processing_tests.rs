use super::*;
use crate::test_support::{linked_project, range, time};

#[test]
fn complete_audio_chain_and_explicit_sidechain_sources_validate() {
    let mut project = linked_project();
    {
        let sequence = &mut project.project.sequences[0];
        sequence.tracks[1].routing = TrackRouting::AudioBus {
            bus_id: BusId::new("bus_dialogue").unwrap(),
        };
        let audio = sequence.tracks[0].clips[0].audio.as_mut().unwrap();
        audio.processors = complete_chain();
        audio.crossfade = Some(AudioCrossfade {
            fade_in: time(30),
            fade_out: time(30),
            curve: AudioFadeCurve::EqualPower,
        });
    }
    project.project.relations.push(Relation {
        id: RelationId::new("rel_sidechain_explicit").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Sidechain {
            key: RelationEndpoint::track(TrackId::new("trk_audio").unwrap()),
            target: RelationEndpoint::item(ItemId::new("itm_video").unwrap()),
            parameters: SidechainRelationParameters {
                threshold_db: -24.0,
                ratio: 4.0,
                attack_ms: 10.0,
                release_ms: 250.0,
                active_range: Some(range(0, 600)),
            },
        },
    });
    validate(&project).unwrap();

    *sidechain_key(&mut project, "rel_sidechain_explicit") =
        RelationEndpoint::bus(BusId::new("bus_dialogue").unwrap());
    validate(&project).unwrap();
}

#[test]
fn audio_processor_ranges_conflicts_and_fades_fail_closed() {
    let mut project = sample_project();
    let audio = project.project.sequences[0].tracks[0].clips[0]
        .audio
        .as_mut()
        .unwrap();
    audio.normalize = true;
    audio.processors = vec![
        AudioProcessor::HighPass {
            frequency_hz: 24_000.0,
            q: 0.0,
            poles: 3,
        },
        AudioProcessor::Loudness(LoudnessTarget {
            integrated_lufs: -100.0,
            true_peak_dbtp: 1.0,
            loudness_range_lu: 0.0,
        }),
        AudioProcessor::Loudness(LoudnessTarget {
            integrated_lufs: -16.0,
            true_peak_dbtp: -1.0,
            loudness_range_lu: 7.0,
        }),
    ];
    audio.crossfade = Some(AudioCrossfade {
        fade_in: time(400),
        fade_out: time(400),
        curve: AudioFadeCurve::Linear,
    });
    let codes = validation_codes(&project);
    assert_code(&codes, "AUDIO_PROCESSOR");
    assert_code(&codes, "AUDIO_PROCESSOR_CONFLICT");
    assert_code(&codes, "AUDIO_CROSSFADE");
}

#[test]
fn sidechain_rejects_bad_parameters_ranges_and_feedback_sources() {
    let mut project = linked_project();
    {
        let sequence = &mut project.project.sequences[0];
        sequence.tracks[0].routing = TrackRouting::AudioBus {
            bus_id: BusId::new("bus_shared").unwrap(),
        };
        sequence.tracks[1].routing = TrackRouting::AudioBus {
            bus_id: BusId::new("bus_shared").unwrap(),
        };
    }
    project.project.relations.push(Relation {
        id: RelationId::new("rel_sidechain").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Sidechain {
            key: RelationEndpoint::bus(BusId::new("bus_shared").unwrap()),
            target: RelationEndpoint::item(ItemId::new("itm_video").unwrap()),
            parameters: SidechainRelationParameters {
                threshold_db: -90.0,
                ratio: 0.5,
                attack_ms: 0.0,
                release_ms: 10_000.0,
                active_range: Some(range(500, 200)),
            },
        },
    });
    let codes = validation_codes(&project);
    assert_code(&codes, "SIDECHAIN_PARAMETERS");
    assert_code(&codes, "SIDECHAIN_RANGE");
    assert_code(&codes, "SIDECHAIN_SELF_DEPENDENCY");

    *sidechain_key(&mut project, "rel_sidechain") =
        RelationEndpoint::track(TrackId::new("trk_video").unwrap());
    assert_code(&validation_codes(&project), "SIDECHAIN_SELF_DEPENDENCY");
}

fn complete_chain() -> Vec<AudioProcessor> {
    vec![
        AudioProcessor::ParametricEq {
            bands: vec![ParametricEqBand {
                frequency_hz: 1_000.0,
                gain_db: 3.0,
                q: 1.2,
            }],
        },
        AudioProcessor::HighPass {
            frequency_hz: 80.0,
            q: 0.707,
            poles: 2,
        },
        AudioProcessor::LowPass {
            frequency_hz: 18_000.0,
            q: 0.707,
            poles: 2,
        },
        AudioProcessor::Compressor(Compressor {
            threshold_db: -18.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 200.0,
            knee_db: 6.0,
            makeup_gain_db: 3.0,
            mix: 1.0,
        }),
        AudioProcessor::Limiter(Limiter {
            ceiling_db: -1.0,
            attack_ms: 5.0,
            release_ms: 50.0,
        }),
        AudioProcessor::Gate(Gate {
            threshold_db: -45.0,
            ratio: 2.0,
            attack_ms: 5.0,
            release_ms: 150.0,
            range_db: -80.0,
        }),
        AudioProcessor::Loudness(LoudnessTarget {
            integrated_lufs: -16.0,
            true_peak_dbtp: -1.0,
            loudness_range_lu: 7.0,
        }),
    ]
}

fn sidechain_key<'a>(project: &'a mut ProjectEnvelope, id: &str) -> &'a mut RelationEndpoint {
    let relation = project
        .project
        .relations
        .iter_mut()
        .find(|relation| relation.id.as_str() == id)
        .unwrap();
    let RelationKind::Sidechain { key, .. } = &mut relation.kind else {
        unreachable!()
    };
    key
}
