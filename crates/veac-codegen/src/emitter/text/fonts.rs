use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use cosmic_text::fontdb::ID;
use cosmic_text::{Fallback, FontSystem};
use veac_plan::{PlanInputId, ResolvedFont};

use super::error::TextError;

mod load;

struct NoFallback;

impl Fallback for NoFallback {
    fn common_fallback(&self) -> &[&'static str] {
        &[]
    }

    fn forbidden_fallback(&self) -> &[&'static str] {
        &[]
    }

    fn script_fallback(&self, _script: unicode_script::Script, _locale: &str) -> &[&'static str] {
        &[]
    }
}

pub(super) struct FontEntry {
    pub alias: String,
    pub ass_name: String,
    id: ID,
}

pub(super) struct EmbeddedFont {
    pub file_name: String,
    pub bytes: Arc<Vec<u8>>,
}

pub(super) struct FontBook {
    pub system: FontSystem,
    pub entries: Vec<FontEntry>,
    indexes: BTreeMap<(PlanInputId, u32), usize>,
    pub directory: PathBuf,
    pub paths: Vec<PathBuf>,
    pub embedded: Vec<EmbeddedFont>,
}

impl FontBook {
    pub fn index(&self, font: &ResolvedFont) -> Result<usize, TextError> {
        self.indexes
            .get(&(font.input_id.clone(), font.face_index))
            .copied()
            .ok_or_else(|| TextError::invalid("resolved font is absent from the font book"))
    }

    pub fn covers(&self, index: usize, text: &str) -> bool {
        let Some(entry) = self.entries.get(index) else {
            return false;
        };
        self.system
            .db()
            .with_face_data(entry.id, |data, face_index| {
                let Ok(face) = ttf_parser::Face::parse(data, face_index) else {
                    return false;
                };
                text.chars()
                    .filter(|character| required_scalar(*character))
                    .all(|character| face.glyph_index(character).is_some())
            })
            .unwrap_or(false)
    }

    pub fn first_covering(&self, text: &str) -> Option<usize> {
        (0..self.entries.len()).find(|index| self.covers(*index, text))
    }
}

fn required_scalar(character: char) -> bool {
    !character.is_whitespace()
        && !matches!(character, '\u{200c}' | '\u{200d}' | '\u{200e}' | '\u{200f}')
        && !(('\u{202a}'..='\u{202e}').contains(&character))
        && !(('\u{2066}'..='\u{2069}').contains(&character))
        && !(('\u{fe00}'..='\u{fe0f}').contains(&character))
        && !(('\u{e0100}'..='\u{e01ef}').contains(&character))
}

#[cfg(test)]
mod tests;
