use std::path::Path;

use cosmic_text::fontdb::Database;
use cosmic_text::FontSystem;
use veac_artifact::{ArtifactErrorKind, BoundResource, ExecutionBindings};
use veac_plan::{ResolvedFont, ResolvedText, ResolvedTextStyle};

use super::{EmbeddedFont, FontBook, FontEntry, NoFallback};
use crate::emitter::text::{error::TextError, font_face};

const DEFAULT_LIMITS: FontLimits = FontLimits {
    file_bytes: veac_artifact::MAX_IN_MEMORY_ARTIFACT_BYTES,
    total_bytes: 384 * 1024 * 1024,
    embedded_bytes: 30_000,
};

#[derive(Clone, Copy)]
pub(super) struct FontLimits {
    pub file_bytes: u64,
    pub total_bytes: u64,
    pub embedded_bytes: usize,
}

struct ReadBudget {
    limits: FontLimits,
    consumed: u64,
}

impl FontBook {
    pub fn load(content: &ResolvedText, bindings: &ExecutionBindings) -> Result<Self, TextError> {
        Self::load_with_limits(content, bindings, DEFAULT_LIMITS)
    }

    pub(super) fn load_with_limits(
        content: &ResolvedText,
        bindings: &ExecutionBindings,
        limits: FontLimits,
    ) -> Result<Self, TextError> {
        let mut db = Database::new();
        let mut entries = Vec::new();
        let mut indexes = std::collections::BTreeMap::new();
        let mut directory = None;
        let mut paths = Vec::new();
        let mut embedded = Vec::new();
        let mut embedded_bytes = 0_usize;
        let mut budget = ReadBudget {
            limits,
            consumed: 0,
        };
        let style = content
            .styled()
            .ok_or_else(|| TextError::invalid("font loading requires styled presentation"))?;
        for font in font_refs(style) {
            let key = (font.input_id.clone(), font.face_index);
            if indexes.contains_key(&key) {
                continue;
            }
            let resource = resource(bindings, font)?;
            let path = resource.path();
            let parent = path.parent().ok_or_else(|| {
                TextError::binding(format!(
                    "font binding {} has no parent directory",
                    path.display()
                ))
            })?;
            let primary = directory.get_or_insert_with(|| parent.to_path_buf());
            let attach = parent != primary && !font_face::system_font(path);
            let bytes = budget.read(resource, path)?;
            if attach {
                embedded_bytes = embedded_bytes.saturating_add(bytes.len());
                if embedded_bytes > limits.embedded_bytes {
                    return Err(limit_error(
                        "TEXT_FONT_EMBED_LIMIT",
                        embedded_bytes,
                        limits.embedded_bytes,
                    ));
                }
            }
            let face = font_face::add(&mut db, bytes, font.face_index, entries.len())?;
            if attach {
                embedded.push(EmbeddedFont {
                    file_name: attachment_name(path, entries.len()),
                    bytes: face.bytes.clone(),
                });
            }
            indexes.insert(key, entries.len());
            if !paths.iter().any(|value| value == path) {
                paths.push(path.to_owned());
            }
            entries.push(FontEntry {
                alias: face.alias,
                ass_name: face.ass_name,
                id: face.id,
            });
        }
        let system =
            FontSystem::new_with_locale_and_db_and_fallback("und".to_owned(), db, NoFallback);
        Ok(Self {
            system,
            entries,
            indexes,
            directory: directory.ok_or_else(|| TextError::invalid("text has no font"))?,
            paths,
            embedded,
        })
    }
}

impl ReadBudget {
    fn read(&mut self, resource: &BoundResource, path: &Path) -> Result<Vec<u8>, TextError> {
        let remaining = self.limits.total_bytes.saturating_sub(self.consumed);
        let file_limit = self
            .limits
            .file_bytes
            .min(veac_artifact::MAX_IN_MEMORY_ARTIFACT_BYTES);
        let bound = file_limit.min(remaining);
        let bytes = resource.read_verified_bounded(bound).map_err(|error| {
            if error.kind == ArtifactErrorKind::ResourceLimit {
                if remaining < file_limit {
                    return limit_error(
                        "TEXT_FONT_TOTAL_LIMIT",
                        self.consumed.saturating_add(bound).saturating_add(1),
                        self.limits.total_bytes,
                    );
                }
                return limit_error("TEXT_FONT_FILE_LIMIT", bound.saturating_add(1), file_limit);
            }
            TextError::binding(format!(
                "cannot verify font binding {}: {error}",
                path.display()
            ))
        })?;
        self.consumed = self.consumed.saturating_add(bytes.len() as u64);
        Ok(bytes)
    }
}

fn resource<'a>(
    bindings: &'a ExecutionBindings,
    font: &ResolvedFont,
) -> Result<&'a BoundResource, TextError> {
    bindings
        .input(&font.input_id)
        .and_then(|binding| binding.resource())
        .ok_or_else(|| {
            TextError::binding(format!("font {} has no resource binding", font.input_id))
        })
}

fn attachment_name(path: &Path, order: usize) -> String {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("ttf");
    format!("veac-font-{order}.{extension}")
}

fn limit_error(
    code: &'static str,
    observed: impl std::fmt::Display,
    limit: impl std::fmt::Display,
) -> TextError {
    TextError::new(code, format!("font bytes {observed} exceed limit {limit}"))
}

fn font_refs(style: &ResolvedTextStyle) -> impl Iterator<Item = &ResolvedFont> {
    std::iter::once(&style.font)
        .chain(&style.fallback_fonts)
        .chain(style.spans.iter().filter_map(|span| span.font.as_ref()))
}
