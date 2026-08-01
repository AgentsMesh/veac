use std::time::Instant;

use super::cleanup;
use crate::executor::staging::directory::Directory;
use crate::executor::staging::ownership;
use crate::RuntimeError;

pub(super) fn discard_if_owned(
    output: &Directory,
    stage_name: &str,
    stage: &Directory,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    let Some(marker) = ownership::identity(stage, stage_name) else {
        return Ok(());
    };
    stage.require(ownership::MARKER_NAME, marker)?;
    cleanup::uncommitted(output, stage_name, stage, deadline)
}
