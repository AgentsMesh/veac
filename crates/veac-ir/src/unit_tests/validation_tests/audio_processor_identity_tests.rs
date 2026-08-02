use super::*;

#[test]
fn same_kind_processors_require_distinct_ids() {
    let mut project = sample_project();
    project.project.sequences[0].tracks[0].clips[0]
        .audio
        .as_mut()
        .unwrap()
        .processors = vec![limiter("aud_first"), limiter("aud_second")];
    validate(&project).unwrap();

    project.project.sequences[0].tracks[0].clips[0]
        .audio
        .as_mut()
        .unwrap()
        .processors[1]
        .id = AudioProcessorId::new("aud_first").unwrap();
    assert_code(&validation_codes(&project), "DUPLICATE_AUDIO_PROCESSOR_ID");
}

#[test]
fn eq_bands_require_distinct_ids_within_their_processor() {
    let mut project = sample_project();
    let band = ParametricEqBand {
        id: EqBandId::new("eqb_presence").unwrap(),
        frequency_hz: 1_000.0,
        gain_db: 1.0,
        q: 1.0,
    };
    project.project.sequences[0].tracks[0].clips[0]
        .audio
        .as_mut()
        .unwrap()
        .processors = vec![AudioProcessor {
        id: AudioProcessorId::new("aud_tone").unwrap(),
        kind: AudioProcessorKind::ParametricEq {
            bands: vec![band.clone(), band],
        },
    }];
    assert_code(&validation_codes(&project), "DUPLICATE_EQ_BAND_ID");
}

#[test]
fn deserialized_processor_and_band_ids_are_revalidated() {
    let mut project = sample_project();
    let invalid_processor = serde_json::from_str::<AudioProcessorId>("\"limiter\"").unwrap();
    let invalid_band = serde_json::from_str::<EqBandId>("\"presence\"").unwrap();
    project.project.sequences[0].tracks[0].clips[0]
        .audio
        .as_mut()
        .unwrap()
        .processors = vec![AudioProcessor {
        id: invalid_processor,
        kind: AudioProcessorKind::ParametricEq {
            bands: vec![ParametricEqBand {
                id: invalid_band,
                frequency_hz: 1_000.0,
                gain_db: 1.0,
                q: 1.0,
            }],
        },
    }];
    let codes = validation_codes(&project);
    assert_eq!(codes.iter().filter(|code| *code == "INVALID_ID").count(), 2);
}

fn limiter(id: &str) -> AudioProcessor {
    AudioProcessor {
        id: AudioProcessorId::new(id).unwrap(),
        kind: AudioProcessorKind::Limiter(Limiter {
            ceiling_db: -1.0,
            attack_ms: 5.0,
            release_ms: 50.0,
        }),
    }
}
