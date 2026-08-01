mod hls;
mod scanner;
mod stage;

use std::path::Path;
use std::time::Instant;

use veac_artifact::DeliveryPackageInventory;

use super::directory::Directory;
use crate::RuntimeError;

pub(super) use stage::{stage, StageContext};

pub(in crate::executor) fn inspect_hls(
    root: &Path,
    entrypoint: &Path,
    deadline: Instant,
) -> Result<DeliveryPackageInventory, RuntimeError> {
    let directory = Directory::open(root)?;
    let first = scanner::inventory(&directory, entrypoint, deadline)?;
    hls::validate(&directory, &first, deadline)?;
    let second = scanner::inventory(&directory, entrypoint, deadline)?;
    if first != second {
        return Err(RuntimeError::new(
            "delivery package changed while it was validated",
        ));
    }
    Ok(second)
}
