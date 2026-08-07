use super::evaluator::Evaluator;
use super::{ResidualRuntimeValue, ResidualizationError};
use crate::program::expression::{
    CoreInstruction, CoreTemporalComposeOperation as Compose,
    CoreTemporalProjectOperation as Project, ValueId,
};
use veac_ir::{
    TemporalColorChannel as Color, TemporalNodeKind, TemporalPointAxis as Point,
    TemporalRectField as Rect, TemporalType, TemporalVectorAxis as Vec2,
};

impl Evaluator<'_> {
    pub(super) fn temporal_compose(
        &mut self,
        operation: Compose,
        operands: &[ValueId],
        instruction: &CoreInstruction,
        slots: &[Option<ResidualRuntimeValue>],
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let values = operands
            .iter()
            .map(|id| self.value(slots, *id, instruction.span()))
            .collect::<Result<Vec<_>, _>>()?;
        let ids = values
            .into_iter()
            .map(|value| self.builder.as_node(value, instruction.span()))
            .collect::<Result<Vec<_>, _>>()?;
        let (value_type, kind) = match operation {
            Compose::Vec2 => (
                TemporalType::Vec2,
                TemporalNodeKind::ComposeVec2 {
                    x: ids[0].node_id,
                    y: ids[1].node_id,
                },
            ),
            Compose::Point => (
                TemporalType::Point,
                TemporalNodeKind::ComposePoint {
                    x: ids[0].node_id,
                    y: ids[1].node_id,
                },
            ),
            Compose::Rect => (
                TemporalType::Rect,
                TemporalNodeKind::ComposeRect {
                    x: ids[0].node_id,
                    y: ids[1].node_id,
                    width: ids[2].node_id,
                    height: ids[3].node_id,
                },
            ),
            Compose::Color => (
                TemporalType::Color,
                TemporalNodeKind::ComposeColor {
                    red: ids[0].node_id,
                    green: ids[1].node_id,
                    blue: ids[2].node_id,
                    alpha: ids[3].node_id,
                },
            ),
        };
        self.builder
            .node(value_type, kind, instruction.span())
            .map(ResidualRuntimeValue::Residual)
    }

    pub(super) fn temporal_project(
        &mut self,
        operation: Project,
        value: ResidualRuntimeValue,
        instruction: &CoreInstruction,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let value = self.builder.as_node(value, instruction.span())?;
        let (value_type, kind) = match operation {
            Project::Vec2X | Project::Vec2Y => (
                TemporalType::Scalar,
                TemporalNodeKind::ProjectVec2 {
                    value: value.node_id,
                    axis: if operation == Project::Vec2X {
                        Vec2::X
                    } else {
                        Vec2::Y
                    },
                },
            ),
            Project::PointX | Project::PointY => (
                TemporalType::Length,
                TemporalNodeKind::ProjectPoint {
                    value: value.node_id,
                    axis: if operation == Project::PointX {
                        Point::X
                    } else {
                        Point::Y
                    },
                },
            ),
            Project::RectX | Project::RectY | Project::RectWidth | Project::RectHeight => (
                TemporalType::Scalar,
                TemporalNodeKind::ProjectRect {
                    value: value.node_id,
                    field: rect_field(operation),
                },
            ),
            Project::ColorRed | Project::ColorGreen | Project::ColorBlue | Project::ColorAlpha => (
                TemporalType::Integer,
                TemporalNodeKind::ProjectColor {
                    value: value.node_id,
                    channel: color_channel(operation),
                },
            ),
        };
        self.builder
            .node(value_type, kind, instruction.span())
            .map(ResidualRuntimeValue::Residual)
    }
}

fn rect_field(value: Project) -> Rect {
    match value {
        Project::RectX => Rect::X,
        Project::RectY => Rect::Y,
        Project::RectWidth => Rect::Width,
        Project::RectHeight => Rect::Height,
        _ => unreachable!(),
    }
}

fn color_channel(value: Project) -> Color {
    match value {
        Project::ColorRed => Color::Red,
        Project::ColorGreen => Color::Green,
        Project::ColorBlue => Color::Blue,
        Project::ColorAlpha => Color::Alpha,
        _ => unreachable!(),
    }
}
