use crate::test_support::{range, time};

use super::*;

#[test]
fn empty_sequences_are_valid_editing_states_before_render_resolution() {
    let mut project = sample_project();
    for track in &mut project.project.sequences[0].tracks {
        track.clips.clear();
    }
    validate(&project).unwrap();
}

#[test]
fn sequence_track_identity_settings_order_and_routing_are_checked() {
    let mut project = sample_project();
    let sequence = &mut project.project.sequences[0];
    sequence.name.clear();
    sequence.settings.width = 0;
    sequence.settings.frame_rate = Rational::new(-1, 1).unwrap();
    sequence.settings.sample_rate = 0;
    sequence.tracks[0].id = serde_json::from_str("\"bad\"").unwrap();
    sequence.tracks[1].id = sequence.tracks[0].id.clone();
    sequence.tracks[1].order = sequence.tracks[0].order;
    let mut duplicate = sequence.clone();
    duplicate.name = "Duplicate".to_owned();
    project.project.sequences.push(duplicate);
    let codes = validation_codes(&project);
    for code in [
        "SEQUENCE_NAME",
        "SEQUENCE_SETTINGS",
        "INVALID_ID",
        "DUPLICATE_SEQUENCE_ID",
        "DUPLICATE_TRACK_ID",
        "DUPLICATE_TRACK_ORDER",
    ] {
        assert_code(&codes, code);
    }
}

#[test]
fn sequence_settings_must_fit_ffmpeg_integer_domains() {
    let mutations: [fn(&mut SequenceSettings); 7] = [
        |settings| settings.width = i32::MAX as u32 + 1,
        |settings| settings.height = i32::MAX as u32 + 1,
        |settings| settings.width = MAX_DIMENSION + 1,
        |settings| (settings.width, settings.height) = (7_680, 4_320),
        |settings| settings.frame_rate.numerator = i64::from(i32::MAX) + 1,
        |settings| {
            settings.frame_rate = Rational::new(i64::from(MAX_FRAME_RATE) + 1, 1).unwrap();
        },
        |settings| settings.sample_rate = i32::MAX as u32 + 1,
    ];
    for mutate in mutations {
        let mut project = sample_project();
        mutate(&mut project.project.sequences[0].settings);
        assert_code(&validation_codes(&project), "SEQUENCE_SETTINGS");
    }
}

#[test]
fn clip_timing_mapping_references_and_track_types_fail_closed() {
    let mut project = sample_project();
    let track = &mut project.project.sequences[0].tracks[0];
    track.kind = TrackKind::Audio;
    track.clips[0].record_range.start = RationalTime::new(-1, 600).unwrap();
    track.clips[0].source_mapping = None;
    let mut duplicate = track.clips[0].clone();
    duplicate.id = track.clips[0].id.clone();
    duplicate.record_range = range(-2, 1);
    duplicate.source = ClipSource::Media {
        material_id: MaterialId::new("med_missing").unwrap(),
    };
    duplicate.source_mapping = Some(SourceMapping {
        time_map: SourceTimeMap::Linear {
            source_start: RationalTime::new(-1, 600).unwrap(),
            rate: Rational::new(0, 1).unwrap(),
            repeat: 0,
            direction: PlaybackDirection::Forward,
        },
        frame_synthesis: FrameSynthesisPolicy::Blend,
        out_of_range: SourceOutOfRangePolicy::Strict,
    });
    track.clips.push(duplicate);
    let caption = &mut project.project.sequences[0].tracks[1].clips[0];
    caption.id = serde_json::from_str("\"bad\"").unwrap();
    caption.source_mapping = Some(SourceMapping::linear(time(0), Rational::new(1, 1).unwrap()));
    caption.source_mapping.as_mut().unwrap().frame_synthesis =
        FrameSynthesisPolicy::MotionCompensated;
    caption.audio = Some(AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::FollowSpeed,
        processors: vec![],
        crossfade: None,
    });
    let codes = validation_codes(&project);
    for code in [
        "RECORD_RANGE",
        "TRACK_ORDER",
        "MAGNETIC_CONTINUITY",
        "SOURCE_MAPPING",
        "DUPLICATE_ITEM_ID",
        "MATERIAL_NOT_FOUND",
        "SOURCE_TIME",
        "SOURCE_RATE",
        "INVALID_ID",
        "TRACK_MEDIA_TYPE",
    ] {
        assert_code(&codes, code);
    }
}

#[test]
fn text_freeze_nested_and_font_references_are_validated() {
    let mut project = sample_project();
    let caption = &mut project.project.sequences[0].tracks[1].clips[0];
    if let ClipSource::Caption { text, style, .. } = &mut caption.source {
        text.clear();
        style.font = FontRef::Material {
            material_id: MaterialId::new("med_video").unwrap(),
        };
    }
    caption.visual = None;
    let mut freeze = caption.clone();
    freeze.id = ItemId::new("itm_freeze").unwrap();
    freeze.source = ClipSource::FreezeFrame {
        material_id: MaterialId::new("med_missing").unwrap(),
        source_time: RationalTime::new(-1, 600).unwrap(),
    };
    freeze.record_range = range(400, 60);
    let mut nested = freeze.clone();
    nested.id = ItemId::new("itm_nested").unwrap();
    nested.source = ClipSource::Sequence {
        sequence_id: SequenceId::new("seq_missing").unwrap(),
    };
    nested.record_range = range(500, 60);
    project.project.sequences[0].tracks[1]
        .clips
        .extend([freeze, nested]);
    let codes = validation_codes(&project);
    for code in [
        "EMPTY_TEXT",
        "FONT_MATERIAL_NOT_FOUND",
        "TEXT_VISUALS",
        "MATERIAL_NOT_FOUND",
        "SOURCE_TIME",
        "SEQUENCE_NOT_FOUND",
        "TRACK_MEDIA_TYPE",
    ] {
        assert_code(&codes, code);
    }
}
