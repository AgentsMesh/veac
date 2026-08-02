use crate::{test_support::*, *};

#[test]
fn enabled_solo_and_mute_have_domain_specific_activity() {
    let mut project = sample_project();
    let sequence = &mut project.project.sequences[0];
    sequence.tracks[1].state.enabled = false;
    sequence.tracks[1].state.solo = true;
    let activity = SequenceActivity::new(sequence);
    assert!(activity.visual_live(&sequence.tracks[0], &sequence.tracks[0].clips[0]));
    assert!(activity.audio_live(&sequence.tracks[0], &sequence.tracks[0].clips[0]));

    sequence.tracks[1].state.enabled = true;
    let activity = SequenceActivity::new(sequence);
    assert!(!activity.item_live(&sequence.tracks[0], &sequence.tracks[0].clips[0]));
    assert!(activity.visual_live(&sequence.tracks[1], &sequence.tracks[1].clips[0]));

    sequence.tracks[1].state.solo = false;
    sequence.tracks[0].state.muted = true;
    let activity = SequenceActivity::new(sequence);
    assert!(activity.visual_live(&sequence.tracks[0], &sequence.tracks[0].clips[0]));
    assert!(!activity.audio_live(&sequence.tracks[0], &sequence.tracks[0].clips[0]));
}

#[test]
fn track_domains_define_implicit_audio_and_visual_components() {
    let mut project = sample_project();
    let sequence = &mut project.project.sequences[0];
    let mut track = sequence.tracks[0].clone();
    track.id = TrackId::new("trk_implicit_audio").unwrap();
    track.kind = TrackKind::Audio;
    track.clips[0].id = ItemId::new("itm_implicit_audio").unwrap();
    track.clips[0].visual = None;
    track.clips[0].audio = None;
    sequence.tracks.push(track);
    let activity = SequenceActivity::new(sequence);
    let audio = sequence.tracks.last().unwrap();
    assert!(activity.audio_typed(audio, &audio.clips[0]));
    assert!(activity.track_has_live_audio(audio));
    assert!(!activity.visual_typed(audio, &audio.clips[0]));
    assert!(!activity.audio_typed(&sequence.tracks[1], &sequence.tracks[1].clips[0]));

    let video = &sequence.tracks[0];
    assert!(video.clips[0].visual.is_some());
    let mut default_visual = video.clips[0].clone();
    default_visual.visual = None;
    assert!(activity.visual_typed(video, &default_visual));
}

#[test]
fn apply_stage_execution_and_half_open_target_windows_are_exact() {
    let mut project = sample_project();
    let sequence = &mut project.project.sequences[0];
    let mut apply = color_apply();
    apply.stages[0].active_range = Some(range(600, 1));
    assert!(!SequenceActivity::new(sequence).apply_live(&apply));

    apply.stages[0].active_range = Some(range(599, 1));
    assert!(SequenceActivity::new(sequence).apply_live(&apply));
    assert_eq!(
        SequenceActivity::new(sequence)
            .live_apply_targets(&apply)
            .len(),
        1
    );

    apply.enabled = false;
    assert!(!SequenceActivity::new(sequence).apply_live(&apply));
    apply.enabled = true;
    apply.stages[0].enabled = false;
    assert!(!apply.stages[0].is_executable());
    assert!(!SequenceActivity::new(sequence).apply_live(&apply));

    apply.stages[0].enabled = true;
    let mut effect = sequence.tracks[0].clips[0].effects[0].clone();
    effect.enabled = false;
    apply.stages[0].operation = ApplyOperation::Effect { effect };
    assert!(!apply.stages[0].is_executable());
    let ApplyOperation::Effect { effect } = &mut apply.stages[0].operation else {
        unreachable!()
    };
    effect.enabled = true;
    assert!(apply.stages[0].is_executable());
}

#[test]
fn layer_and_band_targets_select_only_live_visual_items() {
    let project = sample_project();
    let sequence = &project.project.sequences[0];
    let mut apply = color_apply();
    apply.target = ApplyTarget::Layer {
        track_id: TrackId::new("trk_video").unwrap(),
    };
    assert_eq!(
        SequenceActivity::new(sequence)
            .live_apply_targets(&apply)
            .len(),
        1
    );

    apply.target = ApplyTarget::CompositeBand {
        from_track_id: TrackId::new("trk_video").unwrap(),
        through_track_id: TrackId::new("trk_captions").unwrap(),
    };
    assert_eq!(
        SequenceActivity::new(sequence)
            .live_apply_targets(&apply)
            .len(),
        2
    );

    let missing = TrackId::new("trk_missing").unwrap();
    apply.target = ApplyTarget::CompositeBand {
        from_track_id: missing,
        through_track_id: TrackId::new("trk_captions").unwrap(),
    };
    assert!(!SequenceActivity::new(sequence).apply_live(&apply));
}

fn color_apply() -> Apply {
    Apply {
        id: ApplyId::new("apl_activity").unwrap(),
        enabled: true,
        record_range: range(0, 601),
        target: ApplyTarget::ItemSet {
            item_ids: vec![ItemId::new("itm_video").unwrap()],
        },
        stages: vec![ApplyStage {
            id: ApplyStageId::new("aps_activity").unwrap(),
            enabled: true,
            active_range: None,
            operation: ApplyOperation::Color {
                pipeline: empty_pipeline(),
            },
        }],
        mix: ApplyMix::default(),
    }
}

fn empty_pipeline() -> ColorPipeline {
    let space = ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Bt709,
        matrix: ColorMatrix::Bt709,
        range: ColorRange::Limited,
    };
    ColorPipeline {
        input: space,
        working: space,
        output: space,
        stages: vec![],
    }
}
