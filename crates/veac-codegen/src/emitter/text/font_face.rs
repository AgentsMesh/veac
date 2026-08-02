use std::sync::Arc;

use cosmic_text::fontdb::{Database, Source, ID};

use super::error::TextError;

pub(super) struct LoadedFace {
    pub id: ID,
    pub alias: String,
    pub ass_name: String,
    pub bytes: Arc<Vec<u8>>,
}

pub(super) fn add(
    database: &mut Database,
    bytes: Vec<u8>,
    face_index: u32,
    order: usize,
) -> Result<LoadedFace, TextError> {
    let mut parsed = Database::new();
    let bytes = Arc::new(bytes);
    let ids = parsed.load_font_source(Source::Binary(bytes.clone()));
    let face = ids
        .iter()
        .filter_map(|id| parsed.face(*id))
        .find(|face| face.index == face_index)
        .cloned()
        .ok_or_else(|| {
            TextError::new(
                "TEXT_FONT_FACE_MISSING",
                format!("font collection has no face index {face_index}"),
            )
        })?;
    let alias = format!("VEACFont{order}");
    let ass_name = if face.post_script_name.is_empty() {
        face.families
            .first()
            .map(|family| family.0.clone())
            .ok_or_else(|| TextError::binding("font face has no usable name"))?
    } else {
        face.post_script_name.clone()
    };
    let mut selected = face;
    selected.id = ID::dummy();
    let language = selected
        .families
        .first()
        .map(|family| family.1)
        .unwrap_or(cosmic_text::fontdb::Language::English_UnitedStates);
    selected.families = vec![(alias.clone(), language)];
    Ok(LoadedFace {
        id: database.push_face_info(selected),
        alias,
        ass_name,
        bytes,
    })
}

pub(super) fn system_font(path: &std::path::Path) -> bool {
    [
        "/System/Library/Fonts",
        "/Library/Fonts",
        "/usr/share/fonts",
        "/usr/local/share/fonts",
        "C:\\Windows\\Fonts",
    ]
    .iter()
    .any(|root| path.starts_with(root))
}
