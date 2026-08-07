use super::evaluator::Evaluator;
use super::{ResidualRuntimeValue, ResidualizationError};
use crate::program::expression::{CoreInstruction, ValueId};
use crate::program::{OperandAxis, TemporalLoweringOpcode};
use veac_ir::{Point, Rect, TemporalNodeKind, TemporalType, TemporalValue, Vec2};

impl Evaluator<'_> {
    pub(super) fn domain_construct(
        &mut self,
        opcode: u16,
        operands: &[ValueId],
        instruction: &CoreInstruction,
        slots: &[Option<ResidualRuntimeValue>],
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let contract = self.domain.lookup_opcode(opcode).ok_or_else(|| {
            ResidualizationError::new(
                "RESIDUAL_CORE_CONTRACT",
                format!("unknown domain opcode 0x{opcode:04x}"),
                instruction.span(),
            )
        })?;
        let values = operands
            .iter()
            .map(|id| self.value(slots, *id, instruction.span()))
            .collect::<Result<Vec<_>, _>>()?;
        for (specification, value) in contract.operands().iter().zip(&values) {
            if specification.axis() == OperandAxis::Topology
                && matches!(value, ResidualRuntimeValue::Residual(_))
            {
                return Err(ResidualizationError::new(
                    "RESIDUAL_TEMPORAL_TOPOLOGY",
                    format!(
                        "domain topology operand `{}` is Temporal",
                        specification.name()
                    ),
                    instruction.span(),
                ));
            }
        }
        let lowering = contract.temporal_lowering().ok_or_else(|| {
            ResidualizationError::new(
                "RESIDUAL_CORE_CONTRACT",
                format!(
                    "verified domain operation `{}` lacks Temporal lowering",
                    contract.name()
                ),
                instruction.span(),
            )
        })?;
        if let Some(value) = static_composite(lowering, &values, instruction.span())? {
            return Ok(ResidualRuntimeValue::TemporalConstant(value));
        }
        let ids = values
            .into_iter()
            .map(|value| self.builder.as_node(value, instruction.span()))
            .collect::<Result<Vec<_>, _>>()?;
        let (value_type, kind) = match lowering {
            TemporalLoweringOpcode::ComposeVector => (
                TemporalType::Vec2,
                TemporalNodeKind::ComposeVec2 {
                    x: ids[0].node_id,
                    y: ids[1].node_id,
                },
            ),
            TemporalLoweringOpcode::ComposePoint => (
                TemporalType::Point,
                TemporalNodeKind::ComposePoint {
                    x: ids[0].node_id,
                    y: ids[1].node_id,
                },
            ),
            TemporalLoweringOpcode::ComposeRect => (
                TemporalType::Rect,
                TemporalNodeKind::ComposeRect {
                    x: ids[0].node_id,
                    y: ids[1].node_id,
                    width: ids[2].node_id,
                    height: ids[3].node_id,
                },
            ),
        };
        self.builder
            .node(value_type, kind, instruction.span())
            .map(ResidualRuntimeValue::Residual)
    }
}

fn static_composite(
    operation: TemporalLoweringOpcode,
    values: &[ResidualRuntimeValue],
    span: std::ops::Range<usize>,
) -> Result<Option<TemporalValue>, ResidualizationError> {
    if values
        .iter()
        .any(|value| !matches!(value, ResidualRuntimeValue::Concrete(_)))
    {
        return Ok(None);
    }
    let values = values
        .iter()
        .map(|value| match value {
            ResidualRuntimeValue::Concrete(value) => {
                super::convert::temporal_value(value, span.clone()).map(|value| value.1)
            }
            _ => unreachable!("static composite was checked"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let value = match (operation, values.as_slice()) {
        (
            TemporalLoweringOpcode::ComposeVector,
            [TemporalValue::Scalar { value: x }, TemporalValue::Scalar { value: y }],
        ) => TemporalValue::Vec2 {
            value: Vec2 { x: *x, y: *y },
        },
        (
            TemporalLoweringOpcode::ComposePoint,
            [TemporalValue::Length { value: x }, TemporalValue::Length { value: y }],
        ) => TemporalValue::Point {
            value: Point { x: *x, y: *y },
        },
        (
            TemporalLoweringOpcode::ComposeRect,
            [TemporalValue::Scalar { value: x }, TemporalValue::Scalar { value: y }, TemporalValue::Scalar { value: width }, TemporalValue::Scalar { value: height }],
        ) => TemporalValue::Rect {
            value: Rect {
                x: *x,
                y: *y,
                width: *width,
                height: *height,
            },
        },
        (
            TemporalLoweringOpcode::ComposeVector
            | TemporalLoweringOpcode::ComposePoint
            | TemporalLoweringOpcode::ComposeRect,
            _,
        ) => {
            return Err(ResidualizationError::new(
                "RESIDUAL_CORE_CONTRACT",
                "static composite operands do not match their verified type",
                span,
            ))
        }
    };
    Ok(Some(value))
}
