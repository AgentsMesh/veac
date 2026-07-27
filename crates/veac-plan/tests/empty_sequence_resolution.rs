mod support;

use support::*;
use veac_plan::{canonical::*, resolve, ResolutionErrorKind};

fn assert_duration_unavailable(project: &ProjectEnvelope, sequence_id: &str) {
    let error = resolve(project, None).unwrap_err();
    let diagnostic = error
        .diagnostics()
        .iter()
        .find(|value| value.code == "SEQUENCE_DURATION_UNAVAILABLE")
        .unwrap_or_else(|| panic!("missing sequence duration diagnostic: {error:?}"));
    assert_eq!(
        diagnostic.kind,
        ResolutionErrorKind::TimelineDurationUnavailable
    );
    assert_eq!(diagnostic.object_id.as_deref(), Some(sequence_id));
}

#[test]
fn empty_target_sequence_is_an_editing_state_but_not_a_render_plan() {
    let mut project = project();
    project.project.materials.clear();
    project.project.sequences[0].tracks.clear();

    assert_duration_unavailable(&project, "seq_main");
}

#[test]
fn empty_referenced_sequence_prevents_an_invalid_nested_plan() {
    let mut project = project();
    let sequence_id = SequenceId::new("seq_zempty").unwrap();
    project
        .project
        .sequences
        .push(sequence("seq_zempty", Vec::new()));
    let mut nested = generated_clip("itm_empty_sequence", Generator::Transparent, 0);
    nested.source = ClipSource::Sequence { sequence_id };
    nested.source_mapping = Some(SourceMapping::linear(time(0), Rational::new(1, 1).unwrap()));
    project.project.sequences[0].tracks.push(track(
        "trk_empty_sequence",
        TrackKind::Visual,
        10,
        vec![nested],
    ));

    assert_duration_unavailable(&project, "seq_zempty");
}
