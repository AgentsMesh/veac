use super::*;

mod temporal;

#[test]
fn emits_typed_directional_blur_with_alpha_expansion() {
    let mut plan = resolved(&fixture());
    clip(&mut plan).effects = vec![effect(
        "fx_directional",
        Effect::VideoDirectionalBlur {
            angle_degrees: constant(90.0),
            radius: constant(24.0),
        },
    )];

    let graph = graph(&plan);
    let working = graph.find("visualworkingv").unwrap();
    let premultiply = graph.find("premultiply=inplace=1:planes=7").unwrap();
    let blur = graph.find("dblur@").unwrap();
    let unpremultiply = graph.find("unpremultiply=inplace=1:planes=7").unwrap();
    assert!(
        working < premultiply && premultiply < blur && blur < unpremultiply,
        "{graph}"
    );
    let premultiply_node = graph
        .split(';')
        .find(|node| node.contains("premultiply=inplace"))
        .unwrap();
    assert!(!premultiply_node.contains("format="), "{graph}");
    assert!(
        graph.contains("planes=15:angle=90:radius=24:enable='gte(t,0)*lt(t,1)'"),
        "{graph}"
    );
    assert!(!graph.contains("effectalphasplitv"), "{graph}");
}

#[test]
fn zero_radius_is_a_graph_identity() {
    for radius in [constant(0.0), named_curve("zero", 0.0)] {
        let mut plan = resolved(&fixture());
        clip(&mut plan).effects = vec![effect(
            "fx_directional_zero",
            Effect::VideoDirectionalBlur {
                angle_degrees: constant(90.0),
                radius,
            },
        )];

        let graph = graph(&plan);
        assert!(graph.contains("visualworkingv"), "{graph}");
        for forbidden in ["dblur@", "effectpremultiplyv", "effectunpremultiplyv"] {
            assert!(
                !graph.contains(forbidden),
                "unexpected {forbidden}: {graph}"
            );
        }
    }
}

#[test]
fn alpha_expansion_and_blur_share_the_partial_effect_window() {
    let mut plan = resolved(&fixture());
    let mut directional = effect(
        "fx_directional_window",
        Effect::VideoDirectionalBlur {
            angle_degrees: constant(0.0),
            radius: constant(12.0),
        },
    );
    directional.active_range = TimeRange::new(time(100), time(300)).unwrap();
    clip(&mut plan).effects = vec![directional];

    let graph = graph(&plan);
    let enable = "enable='gte(t,0.166666666667)*lt(t,0.666666666667)'";
    assert_eq!(graph.matches(enable).count(), 3, "{graph}");
}

#[test]
fn animated_radius_disables_every_mutating_filter_at_zero() {
    let mut plan = resolved(&fixture());
    clip(&mut plan).effects = vec![effect(
        "fx_directional_zero_crossing",
        Effect::VideoDirectionalBlur {
            angle_degrees: constant(0.0),
            radius: named_curve("crossing", 24.0),
        },
    )];

    let graph = graph(&plan);
    for marker in [
        "premultiply=inplace",
        "planes=15:angle",
        "unpremultiply=inplace",
    ] {
        let node = graph
            .split(';')
            .find(|node| node.contains(marker))
            .unwrap_or_else(|| panic!("missing {marker}: {graph}"));
        assert!(node.contains("*gt(("), "ungated {marker}: {node}");
    }
}

#[test]
fn animates_directional_blur_angle_and_radius_on_one_instance() {
    let mut plan = resolved(&fixture());
    clip(&mut plan).effects = vec![effect(
        "fx_directional_animated",
        Effect::VideoDirectionalBlur {
            angle_degrees: named_curve("angle", 180.0),
            radius: named_curve("radius", 36.0),
        },
    )];

    let graph = graph(&plan);
    let target = graph
        .split("dblur@")
        .nth(1)
        .and_then(|value| value.split([' ', '=']).next())
        .unwrap();
    assert!(graph.contains(&format!("{target} angle")), "{graph}");
    assert!(graph.contains(&format!("{target} radius")), "{graph}");
    assert_runtime_clamp(&graph, target, "angle", 0.0, 360.0);
    assert_runtime_clamp(&graph, target, "radius", 0.0, 100.0);
    assert_eq!(graph.matches("dblur@").count(), 3, "{graph}");
}

#[test]
fn publishes_every_filter_required_by_the_alpha_aware_backend_chain() {
    let mut plan = resolved(&fixture());
    clip(&mut plan).effects = vec![effect(
        "fx_directional_requirements",
        Effect::VideoDirectionalBlur {
            angle_degrees: constant(0.0),
            radius: constant(12.0),
        },
    )];

    let bundle = veac_codegen::emitter::emit_all(&plan, &bindings(&plan)).unwrap();
    for expected in ["dblur", "format", "premultiply", "unpremultiply"] {
        assert!(
            bundle.requirements().iter().any(|value| {
                value.kind() == veac_codegen::emitter::BackendCapabilityKind::Filter
                    && value.name() == expected
            }),
            "missing filter requirement {expected}"
        );
    }
}

fn named_curve(name: &str, value: f64) -> Animatable<f64> {
    Animatable::Keyframes {
        keyframes: vec![
            Keyframe {
                id: KeyframeId::new(format!("kf_{name}_start")).unwrap(),
                time: time(0),
                value: 0.0,
                interpolation: Interpolation::Linear,
            },
            Keyframe {
                id: KeyframeId::new(format!("kf_{name}_end")).unwrap(),
                time: time(600),
                value,
                interpolation: Interpolation::Linear,
            },
        ],
    }
}

fn assert_runtime_clamp(graph: &str, target: &str, option: &str, minimum: f64, maximum: f64) {
    let command = graph
        .split(&format!("{target} {option} "))
        .nth(1)
        .unwrap_or_else(|| panic!("missing {option} command: {graph}"));
    let bounds = format!("\\\\,{minimum}\\\\,{maximum}");
    assert!(
        command.starts_with("clip(("),
        "unclamped {option}: {command}"
    );
    assert!(
        command.contains(&bounds),
        "wrong {option} bounds: {command}"
    );
}
