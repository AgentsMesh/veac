use sha2::{Digest, Sha256};

use super::{bytes, length, tag, u32_value};
use crate::program::expression::{
    CoreInputDeclarationDigest, CoreInputIdentity, CoreProgram, CoreTemporalInputIdentity,
};

const DOMAIN: &[u8] = b"veac.core-v10.input-declarations\0";

pub(crate) fn digest(program: &CoreProgram) -> CoreInputDeclarationDigest {
    let mut digest = Sha256::new();
    digest.update(DOMAIN);
    length(&mut digest, program.inputs.len());
    for input in &program.inputs {
        u32_value(&mut digest, input.id.value());
        identity(&mut digest, &input.identity);
        let value_type = program
            .value_type(input.type_id)
            .expect("verified input declaration has a value type");
        super::value_type::encode(&mut digest, value_type);
    }
    CoreInputDeclarationDigest::from_bytes(digest.finalize().into())
}

fn identity(digest: &mut Sha256, value: &CoreInputIdentity) {
    match value {
        CoreInputIdentity::Build(id) => {
            tag(digest, 0x00);
            digest.update(id.as_bytes());
        }
        CoreInputIdentity::Temporal(value) => temporal_identity(digest, value),
    }
}

fn temporal_identity(digest: &mut Sha256, value: &CoreTemporalInputIdentity) {
    use CoreTemporalInputIdentity::*;
    match value {
        SequenceTime { sequence_id } => tagged_bytes(digest, 0x10, sequence_id.as_str()),
        ClipTime { item_id } => tagged_bytes(digest, 0x11, item_id.as_str()),
        SourceTime { source_id } => tagged_bytes(digest, 0x12, source_id.as_str()),
        Frame { sequence_id } => tagged_bytes(digest, 0x13, sequence_id.as_str()),
        Progress { item_id } => tagged_bytes(digest, 0x14, item_id.as_str()),
        Parameter {
            parameter_id,
            value_type,
        } => {
            tagged_bytes(digest, 0x15, parameter_id.as_str());
            temporal_type(digest, *value_type);
        }
    }
}

fn tagged_bytes(digest: &mut Sha256, kind: u8, value: &str) {
    tag(digest, kind);
    bytes(digest, value.as_bytes());
}

fn temporal_type(digest: &mut Sha256, value: veac_ir::TemporalType) {
    let value = match value {
        veac_ir::TemporalType::Boolean => 0,
        veac_ir::TemporalType::Integer => 1,
        veac_ir::TemporalType::Scalar => 2,
        veac_ir::TemporalType::Time => 3,
        veac_ir::TemporalType::Length => 4,
        veac_ir::TemporalType::Angle => 5,
        veac_ir::TemporalType::Vec2 => 6,
        veac_ir::TemporalType::Point => 7,
        veac_ir::TemporalType::Rect => 8,
        veac_ir::TemporalType::Color => 9,
        veac_ir::TemporalType::Text => 10,
    };
    tag(digest, value);
}
