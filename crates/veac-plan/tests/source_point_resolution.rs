mod support;

use support::*;
use veac_plan::{canonical::*, resolve, ResolvedSourceTimeMap};

#[test]
fn nested_hold_point_is_checked_against_the_child_duration() {
    let mut project = project();
    let mut child = generated_clip("itm_child", Generator::Transparent, 0);
    child.record_range = range(0, 600);
    child.visual = Some(visual_properties());
    project.project.sequences.push(sequence(
        "seq_child",
        vec![track("trk_child", TrackKind::Visual, 0, vec![child])],
    ));

    let parent = &mut project.project.sequences[0].tracks[0].clips[0];
    parent.source = ClipSource::Sequence {
        sequence_id: SequenceId::new("seq_child").unwrap(),
    };
    parent.source_mapping = Some(point_mapping(300, SourceOutOfRangePolicy::Strict));
    parent.visual = Some(visual_properties());

    let plan = resolve(&project, None).unwrap().remove(0);
    let main = plan
        .sequences
        .iter()
        .find(|sequence| sequence.id.as_str() == "seq_main")
        .unwrap();
    let mapping = main.tracks[0].clips[0].source_mapping.as_ref().unwrap();
    let ResolvedSourceTimeMap::Curve { segments } = &mapping.time_map else {
        panic!("point curve")
    };
    assert_eq!(segments[0].source_start, time(300));
}

#[test]
fn strict_late_hold_reports_video_and_audio_point_bounds() {
    let mut project = project();
    project.project.render_configs[0]
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.audio = Some(audio_properties());
    clip.source_mapping = Some(point_mapping(60_000, SourceOutOfRangePolicy::Strict));

    let error = resolve(&project, None).unwrap_err();
    let messages: Vec<_> = error
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.code == "SOURCE_RANGE_OUT_OF_BOUNDS")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
        messages.iter().any(|message| message.contains("video")),
        "{messages:?}: {error:?}"
    );
    assert!(
        messages.iter().any(|message| message.contains("audio")),
        "{messages:?}: {error:?}"
    );
}

fn point_mapping(value: i64, out_of_range: SourceOutOfRangePolicy) -> SourceMapping {
    SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: vec![SourceTimeSegment {
                record_duration: time(600),
                source_start: time(value),
                source_end: time(value),
                interpolation: SourceTimeInterpolation::Hold,
            }],
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range,
    }
}
