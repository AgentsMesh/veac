use crate::*;

use super::support::{literal, program};

#[test]
fn composite_node_axes_are_part_of_the_content_digest() {
    let color = TemporalValue::Color {
        value: Color {
            red: 1,
            green: 2,
            blue: 3,
            alpha: 4,
        },
    };
    let mut value = program(
        Vec::new(),
        vec![
            literal(0, color),
            TemporalNode {
                id: TemporalNodeId::new(1),
                value_type: TemporalType::Integer,
                kind: TemporalNodeKind::ProjectColor {
                    value: TemporalNodeId::new(0),
                    channel: TemporalColorChannel::Red,
                },
                provenance_id: None,
            },
        ],
        1,
        TemporalType::Integer,
    );
    let digest = value.content_sha256.clone();
    let TemporalNodeKind::ProjectColor { channel, .. } = &mut value.nodes[1].kind else {
        unreachable!()
    };
    *channel = TemporalColorChannel::Blue;
    assert_ne!(temporal_program_digest(&value).unwrap(), digest);
}

#[test]
fn geometry_numbers_participate_in_digest_canonicalization() {
    for value in [
        TemporalValue::Point {
            value: Point {
                x: Length {
                    value: -0.0,
                    unit: LengthUnit::Pixels,
                },
                y: Length {
                    value: 0.0,
                    unit: LengthUnit::Pixels,
                },
            },
        },
        TemporalValue::Rect {
            value: Rect {
                x: 0.0,
                y: 0.0,
                width: -0.0,
                height: 1.0,
            },
        },
    ] {
        let mut candidate = program(
            Vec::new(),
            vec![literal(0, TemporalValue::Boolean { value: true })],
            0,
            TemporalType::Boolean,
        );
        candidate.result_type = value.value_type();
        candidate.nodes[0] = literal(0, value);
        assert!(matches!(
            temporal_program_digest(&candidate),
            Err(TemporalDigestError::NonCanonicalNumber)
        ));
    }
}
