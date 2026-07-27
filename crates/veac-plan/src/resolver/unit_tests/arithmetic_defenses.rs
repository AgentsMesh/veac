use super::{defenses::direct, support::*};
use crate::{canonical::*, resolver::bounds::BoundContext, resolver::material::InputUsage, *};

#[test]
fn direct_output_and_bound_arithmetic_fail_with_diagnostics() {
    let mut value = project();
    value.project.render_configs[0].id = serde_json::from_str("\"out_bad/slash\"").unwrap();
    let mut resolver = direct(&value);
    assert!(resolver.output_id().is_none());
    assert_eq!(resolver.diagnostics[0].code, "PLAN_OUTPUT_ID");
    let (plan, errors) = direct(&value).build();
    assert!(plan.is_none());
    assert_eq!(errors[0].code, "PLAN_OUTPUT_ID");

    let value = project();
    let mut resolver = direct(&value);
    let input = resolver
        .material_input(
            &MaterialId::new("med_video").unwrap(),
            InputUsage {
                video: true,
                ..InputUsage::default()
            },
        )
        .unwrap();
    let clip = &value.project.sequences[0].tracks[0].clips[0];
    resolver.check_source_bounds(
        &input,
        TimeRange {
            start: RationalTime::new(MAX_SAFE_INTEGER as i64, 1).unwrap(),
            duration: RationalTime::new(1, 1).unwrap(),
        },
        InputUsage {
            video: true,
            ..InputUsage::default()
        },
        BoundContext::new(clip, "/clip", SourceOutOfRangePolicy::Strict),
        false,
    );
    assert!(resolver
        .diagnostics
        .iter()
        .any(|item| item.code == "SOURCE_BOUND_TIME_ARITHMETIC"));
}

#[test]
fn private_time_font_and_empty_sequence_invariants_are_total() {
    assert!(super::super::time::source_range(
        time(0),
        time(MAX_SAFE_INTEGER as i64),
        Rational::new(MAX_SAFE_INTEGER as i64, 1).unwrap(),
        1,
    )
    .is_none());
    assert!(
        super::super::time::source_range(time(0), time(1), Rational::new(1, 1).unwrap(), 0)
            .is_none()
    );

    let value = project();
    let mut resolver = direct(&value);
    let style = text_style(FontRef::Material {
        material_id: MaterialId::new("med_video").unwrap(),
    });
    assert!(resolver.resolve_text("bad font", &style, "/clip").is_none());
    assert!(resolver
        .diagnostics
        .iter()
        .any(|item| item.code == "FONT_INPUT_KIND"));

    let mut empty = project();
    empty.project.materials.clear();
    empty.project.sequences[0].tracks.clear();
    let error = resolve(&empty, None).unwrap_err();
    assert!(error
        .diagnostics()
        .iter()
        .any(|item| item.code == "SEQUENCE_DURATION_UNAVAILABLE"));
}
