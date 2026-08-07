use sha2::Sha256;

use super::super::{length, tag, u16_value, u32_value};
use crate::program::expression::ValueId;

pub(super) fn encode(digest: &mut Sha256, kind: u8, opcode: u16, operands: &[ValueId]) {
    tag(digest, kind);
    u16_value(digest, opcode);
    length(digest, operands.len());
    for operand in operands {
        u32_value(digest, operand.value());
    }
}
