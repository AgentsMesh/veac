use std::collections::BTreeMap;

use veac_artifact::ExecutionBindings;
use veac_plan::{PlanInputId, ResolvedFont};

use super::super::failure::Failure;

pub(super) struct FontCatalog<'a> {
    bindings: &'a ExecutionBindings,
    names: BTreeMap<(PlanInputId, u32), String>,
}

impl<'a> FontCatalog<'a> {
    pub fn new(bindings: &'a ExecutionBindings) -> Self {
        Self {
            bindings,
            names: BTreeMap::new(),
        }
    }

    pub fn name(&mut self, font: &ResolvedFont) -> Result<String, Failure> {
        let key = (font.input_id.clone(), font.face_index);
        if let Some(name) = self.names.get(&key) {
            return Ok(name.clone());
        }
        let name = crate::emitter::text::bound_ass_font_name(font, self.bindings)
            .map_err(|error| Failure::invalid("CAPTION_ASS_FONT_BINDING", error))?;
        if name.trim().is_empty()
            || name
                .chars()
                .any(|value| matches!(value, ',' | '\\' | '{' | '}' | '\r' | '\n'))
        {
            return Err(Failure::unsupported(
                "CAPTION_ASS_FONT_NAME",
                "bound font name cannot be represented safely in ASS",
            ));
        }
        self.names.insert(key, name.clone());
        Ok(name)
    }
}
