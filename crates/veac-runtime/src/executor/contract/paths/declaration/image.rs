use std::path::Path;

use veac_ir::ImageSequencePattern;

use super::{normalized, Declaration};
use crate::executor::contract::paths::{existing, utf8};
use crate::RuntimeError;

pub(super) fn from_path(path: &Path) -> Result<Declaration, RuntimeError> {
    let normalized = normalized(path)?;
    existing::validate_pattern(&normalized)?;
    let name = utf8(normalized.file_name().unwrap().as_ref())?;
    let Some(pattern) = ImageSequencePattern::parse(name) else {
        return super::super::super::invalid("image sequence pattern has no valid placeholder");
    };
    Ok(Declaration::Pattern {
        parent: normalized.parent().unwrap().to_path_buf(),
        prefix: pattern.prefix.as_bytes().to_vec(),
        suffix: pattern.suffix.as_bytes().to_vec(),
        minimum_width: pattern.minimum_width,
    })
}
