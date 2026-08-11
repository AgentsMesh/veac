use super::*;

#[test]
fn every_runtime_filter_number_clamps_to_its_scaled_sink_bounds() {
    for case in cases() {
        let mut plan = resolved(&fixture());
        let mut value = with_parameter(case.kind, case.parameter, animated(0.5));
        if case.kind == EffectKind::VideoDirectionalBlur
            && case.parameter == EffectParameter::AngleDegrees
        {
            assert_eq!(
                value.set_parameter(
                    EffectParameter::Radius,
                    EffectParameterValue::Curve(Animatable::constant(1.0)),
                ),
                Some(true)
            );
        }
        clip(&mut plan).effects = vec![effect("fx_sink_bounds", value)];
        let rendered = graph(&plan);
        let command = rendered
            .split(&format!(" {} ", case.option))
            .nth(1)
            .unwrap_or_else(|| panic!("missing {}: {rendered}", case.option));
        assert!(command.starts_with("clip(("), "{}: {command}", case.option);
        assert!(command.contains(case.bounds), "{}: {command}", case.option);
        if let Some(scale) = case.scale {
            assert!(command.contains(scale), "{}: {command}", case.option);
        }
    }
}

struct Case {
    kind: EffectKind,
    parameter: EffectParameter,
    option: &'static str,
    bounds: &'static str,
    scale: Option<&'static str>,
}

fn cases() -> [Case; 11] {
    use EffectKind::*;
    use EffectParameter::*;
    [
        case(VideoBlur, Radius, "sigma", "\\\\,0\\\\,100)"),
        case(
            VideoDirectionalBlur,
            AngleDegrees,
            "angle",
            "\\\\,0\\\\,360)",
        ),
        case(VideoDirectionalBlur, Radius, "radius", "\\\\,0\\\\,100)"),
        Case {
            scale: Some(")*0.1)"),
            ..case(VideoSharpen, Amount, "strength", "\\\\,0\\\\,1)")
        },
        case(
            VideoChromaKey,
            Similarity,
            "similarity",
            "\\\\,0.00001\\\\,1)",
        ),
        case(VideoChromaKey, Blend, "blend", "\\\\,0\\\\,1)"),
        case(VideoLumaKey, Threshold, "threshold", "\\\\,0\\\\,1)"),
        case(VideoLumaKey, Tolerance, "tolerance", "\\\\,0\\\\,1)"),
        case(VideoLumaKey, Softness, "softness", "\\\\,0\\\\,1)"),
        case(VideoChromaSpill, Amount, "mix", "\\\\,0\\\\,1)"),
        case(VideoChromaSpill, Range, "expand", "\\\\,0\\\\,1)"),
    ]
}

const fn case(
    kind: EffectKind,
    parameter: EffectParameter,
    option: &'static str,
    bounds: &'static str,
) -> Case {
    Case {
        kind,
        parameter,
        option,
        bounds,
        scale: None,
    }
}
