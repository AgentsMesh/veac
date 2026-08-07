use super::diagnostic::Diagnostics;
use super::model::FileKind;
use super::parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceUnitKind {
    Entry,
    Module,
}

pub fn classify_source_unit(path: &str, source: &str) -> Result<SourceUnitKind, Diagnostics> {
    parser::parse_executable(path, source)
        .map(|file| match file.kind {
            FileKind::Entry => SourceUnitKind::Entry,
            FileKind::Module => SourceUnitKind::Module,
        })
        .map_err(Diagnostics)
}

#[cfg(test)]
#[path = "source_unit/tests.rs"]
mod tests;
