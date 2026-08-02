use std::collections::BTreeSet;
use std::path::PathBuf;

use super::StagedOutput;
use crate::executor::output;
use crate::RuntimeError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::executor) enum StaleFamily {
    ImageSequence(PathBuf),
    Passlog(PathBuf),
}

pub(super) fn enumerate(
    families: &[StaleFamily],
    outputs: &[StagedOutput],
) -> Result<Vec<PathBuf>, RuntimeError> {
    let targets = outputs
        .iter()
        .map(|output| output.target().clone())
        .collect::<BTreeSet<_>>();
    let mut stale = BTreeSet::new();
    for family in families {
        let existing = match family {
            StaleFamily::ImageSequence(pattern) => output::enumerate_pattern(pattern)?,
            StaleFamily::Passlog(prefix) => output::enumerate_passlogs(prefix)?,
        };
        stale.extend(existing.into_iter().filter(|path| !targets.contains(path)));
    }
    Ok(stale.into_iter().collect())
}
