use veac_ir::{Length, LengthUnit, Point, TemporalNodeKind, TemporalValue};
use veac_lang::program::build_source;

use super::support::{evaluate, MAIN, SINK_MATRIX};

#[test]
fn authored_curve_reaches_canonical_ir_and_reference_evaluator() {
    let source = MAIN.replace(
        "pulse(progress)",
        "sample_curve_ease_in_out(progress, [(0.0, 0.0), (0.5, 1.0), (1.0, 0.0)])",
    );
    let built = build_source(&source).unwrap();
    let envelope = built.envelope();
    assert!(envelope.temporal.programs[0]
        .nodes
        .iter()
        .any(|node| matches!(node.kind, TemporalNodeKind::CurveSample { .. })));
    assert_eq!(
        evaluate(envelope, TemporalValue::Scalar { value: 0.25 }),
        TemporalValue::Scalar { value: 0.5 }
    );
    assert!(veac_ir::validate(envelope).is_ok());
}

#[test]
fn authored_point_curve_keeps_typed_composite_keys() {
    let source = format!(
        "animate visual-position on clip(@matrix, @main, @visual, @visual-clip) {{\n  \
         sample_curve_linear(progress, [(0.0, point(0px, 10px)), \
         (1.0, point(20px, 30px))])\n}}\n{SINK_MATRIX}"
    );
    let built = build_source(&source).unwrap();
    let envelope = built.envelope();
    assert_eq!(
        evaluate(envelope, TemporalValue::Scalar { value: 0.5 }),
        TemporalValue::Point {
            value: Point {
                x: Length {
                    value: 10.0,
                    unit: LengthUnit::Pixels,
                },
                y: Length {
                    value: 20.0,
                    unit: LengthUnit::Pixels,
                },
            }
        }
    );
    assert!(veac_ir::validate(envelope).is_ok());
}
