use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::Interpolation;

use super::super::value;
use super::{invalid, ExecutableLowerError};

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Interpolation, ExecutableLowerError> {
    let (operation, operands) = value::description(graph, source)?;
    match (operation, operands) {
        (Op::InterpolationHold, []) => Ok(Interpolation::Hold),
        (Op::InterpolationLinear, []) => Ok(Interpolation::Linear),
        (Op::InterpolationEaseIn, []) => Ok(Interpolation::EaseIn),
        (Op::InterpolationEaseOut, []) => Ok(Interpolation::EaseOut),
        (Op::InterpolationEaseInOut, []) => Ok(Interpolation::EaseInOut),
        (Op::InterpolationSpring, [frequency, decay, velocity]) => {
            let frequency = scalar(Some(frequency))?;
            let decay = scalar(Some(decay))?;
            let initial_velocity = scalar(Some(velocity))?;
            if frequency <= 0.0 || decay <= 0.0 {
                return Err(invalid());
            }
            Ok(Interpolation::Spring {
                frequency,
                decay,
                initial_velocity,
            })
        }
        (Op::InterpolationCubicBezier, [x1, y1, x2, y2]) => {
            let [x1, y1, x2, y2] = [x1, y1, x2, y2].map(|value| scalar(Some(value)));
            let (x1, y1, x2, y2) = (x1?, y1?, x2?, y2?);
            if !(0.0..=1.0).contains(&x1) || !(0.0..=1.0).contains(&x2) {
                return Err(invalid());
            }
            Ok(Interpolation::CubicBezier { x1, y1, x2, y2 })
        }
        _ => Err(invalid()),
    }
}

fn scalar(source: Option<&Value>) -> Result<f64, ExecutableLowerError> {
    let Some(Value::Scalar(value)) = source else {
        return Err(invalid());
    };
    let value = value.numerator() as f64 / value.denominator() as f64;
    value.is_finite().then_some(value).ok_or_else(invalid)
}
