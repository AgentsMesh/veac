use sha2::{Digest, Sha256};

use super::{bytes, length, tag};
use crate::program::expression::Value;

pub(super) fn encode(digest: &mut Sha256, value: &Value) {
    match value {
        Value::Integer(value) => {
            tag(digest, 0x00);
            digest.update(value.to_be_bytes());
        }
        Value::Scalar(value) => number(digest, 0x01, *value),
        Value::Time(value) => number(digest, 0x02, *value),
        Value::Length(value) => number(digest, 0x03, *value),
        Value::Percent(value) => number(digest, 0x04, *value),
        Value::Angle(value) => number(digest, 0x05, *value),
        Value::Text(value) => tagged_bytes(digest, 0x06, value.as_bytes()),
        Value::Color(value) => tagged_bytes(digest, 0x07, value.as_bytes()),
        Value::Bool(value) => digest.update([0x08, u8::from(*value)]),
        Value::Identifier(value) => tagged_bytes(digest, 0x09, value.as_bytes()),
        Value::Range(value) => {
            tag(digest, 0x0a);
            digest.update(value.start().to_be_bytes());
            digest.update(value.end().to_be_bytes());
            digest.update(value.step().to_be_bytes());
        }
        Value::List(value) => {
            tag(digest, 0x0b);
            super::value_type::encode(digest, value.value_type());
            values(digest, value.values());
        }
        Value::Map(value) => {
            tag(digest, 0x0c);
            super::value_type::encode(digest, value.value_type());
            length(digest, value.entries().len());
            for entry in value.entries() {
                encode(digest, entry.key());
                encode(digest, entry.value());
            }
        }
        Value::Tuple(value) => {
            tag(digest, 0x0d);
            values(digest, value.values());
        }
        Value::Struct(value) => {
            tag(digest, 0x0e);
            digest.update(value.type_id().as_bytes());
            digest.update(value.definition_digest().as_bytes());
            values(digest, value.fields());
        }
        Value::Enum(value) => {
            tag(digest, 0x0f);
            digest.update(value.type_id().as_bytes());
            digest.update(value.definition_digest().as_bytes());
            digest.update(value.variant().value().to_be_bytes());
            values(digest, value.fields());
        }
        Value::Closure(_) => unreachable!("closure values cannot be Core literals"),
        Value::Domain(_) => unreachable!("domain handles cannot be Core literals"),
    }
}

fn number(digest: &mut Sha256, kind: u8, value: crate::program::expression::ExactNumber) {
    tag(digest, kind);
    digest.update(value.numerator().to_be_bytes());
    digest.update(value.denominator().to_be_bytes());
}

fn tagged_bytes(digest: &mut Sha256, kind: u8, value: &[u8]) {
    tag(digest, kind);
    bytes(digest, value);
}

fn values(digest: &mut Sha256, values: &[Value]) {
    length(digest, values.len());
    values.iter().for_each(|value| encode(digest, value));
}
