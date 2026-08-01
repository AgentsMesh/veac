use super::*;
use crate::test_support::identity_layout_visual;

#[test]
fn generated_canvas_identity_is_not_charged_for_an_unemitted_pad() {
    let mut project = sample_project();
    let sequence = &mut project.project.sequences[0];
    sequence.settings.width = 4_096;
    sequence.settings.height = 4_096;
    sequence.tracks.truncate(1);
    let clip = &mut sequence.tracks[0].clips[0];
    clip.source = ClipSource::Generated {
        generator: Generator::Transparent,
    };
    clip.source_mapping = None;
    clip.effects.clear();
    clip.visual = Some(identity_layout_visual());

    let diagnostics = validate(&project)
        .err()
        .into_iter()
        .flat_map(ValidationErrors::into_diagnostics)
        .collect::<Vec<_>>();
    assert!(
        !diagnostics
            .iter()
            .any(|error| error.code == "BUDGET_VISUAL_INTERMEDIATE_PIXELS"),
        "{diagnostics:?}"
    );
}

#[test]
fn large_shadow_offset_is_rejected_by_the_visual_intermediate_budget() {
    let mut project = sample_project();
    let shadow = project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .card
        .as_mut()
        .unwrap()
        .shadow
        .as_mut()
        .unwrap();
    shadow.offset = Vec2 {
        x: 100_000.0,
        y: 100_000.0,
    };

    let codes = validate(&project)
        .unwrap_err()
        .into_diagnostics()
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    assert!(
        codes
            .iter()
            .any(|code| code == "BUDGET_VISUAL_INTERMEDIATE_PIXELS"),
        "{codes:?}"
    );
}
