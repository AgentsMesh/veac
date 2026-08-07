use crate::temporal::{TemporalNodeKind, TemporalType};

use super::Validator;

impl Validator {
    pub(super) fn composite(
        &mut self,
        kind: &TemporalNodeKind,
        types: &[TemporalType],
        pointer: &str,
    ) -> Option<TemporalType> {
        use TemporalNodeKind::*;
        use TemporalType as Type;
        let valid = match kind {
            ComposeVec2 { x, y } => self.pair(*x, *y, types, pointer, Type::Scalar, Type::Vec2),
            ProjectVec2 { value, .. } => {
                self.project(*value, types, pointer, Type::Vec2, Type::Scalar)
            }
            ComposePoint { x, y } => self.pair(*x, *y, types, pointer, Type::Length, Type::Point),
            ProjectPoint { value, .. } => {
                self.project(*value, types, pointer, Type::Point, Type::Length)
            }
            ComposeRect {
                x,
                y,
                width,
                height,
            } => self.quad(
                [*x, *y, *width, *height],
                types,
                pointer,
                Type::Scalar,
                Type::Rect,
            ),
            ProjectRect { value, .. } => {
                self.project(*value, types, pointer, Type::Rect, Type::Scalar)
            }
            ComposeColor {
                red,
                green,
                blue,
                alpha,
            } => self.quad(
                [*red, *green, *blue, *alpha],
                types,
                pointer,
                Type::Integer,
                Type::Color,
            ),
            ProjectColor { value, .. } => {
                self.project(*value, types, pointer, Type::Color, Type::Integer)
            }
            _ => None,
        };
        valid.or_else(|| self.bad_operation(pointer))
    }

    fn pair(
        &mut self,
        left: super::super::TemporalNodeId,
        right: super::super::TemporalNodeId,
        types: &[TemporalType],
        pointer: &str,
        operand: TemporalType,
        result: TemporalType,
    ) -> Option<TemporalType> {
        (self.operand(left, types, pointer)? == operand
            && self.operand(right, types, pointer)? == operand)
            .then_some(result)
    }

    fn quad(
        &mut self,
        ids: [super::super::TemporalNodeId; 4],
        types: &[TemporalType],
        pointer: &str,
        operand: TemporalType,
        result: TemporalType,
    ) -> Option<TemporalType> {
        for id in ids {
            if self.operand(id, types, pointer)? != operand {
                return None;
            }
        }
        Some(result)
    }

    fn project(
        &mut self,
        id: super::super::TemporalNodeId,
        types: &[TemporalType],
        pointer: &str,
        operand: TemporalType,
        result: TemporalType,
    ) -> Option<TemporalType> {
        (self.operand(id, types, pointer)? == operand).then_some(result)
    }
}
