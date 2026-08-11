mod advanced;
mod basic;
mod types;

use veac_lang::program::expression::Value;

use crate::AssertionSpec;

use super::super::EvidenceDecodeError;
use super::value::{unknown_variant, Decoder};

pub(super) fn assertion(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<AssertionSpec, EvidenceDecodeError> {
    let variant = decoder.variant(value, "EvidenceAssertion", path)?;
    match variant.name {
        "DecodeComplete" => basic::decode_complete(decoder, &variant.fields),
        "Alpha" => basic::alpha(decoder, &variant.fields),
        "PixelDiff" => basic::pixel_diff(decoder, &variant.fields),
        "Bounds" => basic::bounds(decoder, &variant.fields),
        "LayerOrder" => basic::layer_order(decoder, &variant.fields),
        "CompositeOver" => advanced::composite(decoder, &variant.fields),
        "RevealOrder" => advanced::reveal(decoder, &variant.fields),
        "MotionProfile" => advanced::motion(decoder, &variant.fields),
        name => Err(unknown_variant(path, "EvidenceAssertion", name)),
    }
}
