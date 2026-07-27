use std::collections::{BTreeMap, BTreeSet};

use veac_ir::{ItemId, Material};

use crate::{inventory::Inventory, TemplateError, TemplateErrorKind, TemplateFillRequest};

pub(crate) struct ResolvedBindings<'a> {
    pub media: BTreeMap<ItemId, &'a Material>,
    pub texts: BTreeMap<ItemId, &'a str>,
}

pub(crate) fn resolve<'a>(
    inventory: &Inventory,
    request: &'a TemplateFillRequest,
) -> Result<ResolvedBindings<'a>, TemplateError> {
    let slot_ids: BTreeSet<_> = inventory
        .slots
        .iter()
        .map(|slot| slot.clip_id.clone())
        .collect();
    let text_ids: BTreeSet<_> = inventory
        .texts
        .iter()
        .map(|text| text.clip_id.clone())
        .collect();
    let media = resolve_media(&slot_ids, request)?;
    let texts = resolve_texts(&text_ids, request)?;
    Ok(ResolvedBindings { media, texts })
}

fn resolve_media<'a>(
    expected: &BTreeSet<ItemId>,
    request: &'a TemplateFillRequest,
) -> Result<BTreeMap<ItemId, &'a Material>, TemplateError> {
    let mut result = BTreeMap::new();
    for binding in &request.media_bindings {
        if !expected.contains(&binding.clip_id) {
            return Err(TemplateError::clip(
                TemplateErrorKind::UnexpectedMediaBinding,
                &binding.clip_id,
                "media binding does not target a replaceable slot",
            ));
        }
        if result
            .insert(binding.clip_id.clone(), &binding.material)
            .is_some()
        {
            return Err(TemplateError::clip(
                TemplateErrorKind::DuplicateMediaBinding,
                &binding.clip_id,
                "media slot is bound more than once",
            ));
        }
    }
    if let Some(missing) = expected.iter().find(|id| !result.contains_key(*id)) {
        return Err(TemplateError::clip(
            TemplateErrorKind::MissingMediaBinding,
            missing,
            "replaceable slot has no media binding",
        ));
    }
    Ok(result)
}

fn resolve_texts<'a>(
    expected: &BTreeSet<ItemId>,
    request: &'a TemplateFillRequest,
) -> Result<BTreeMap<ItemId, &'a str>, TemplateError> {
    let mut result = BTreeMap::new();
    for binding in &request.text_bindings {
        if !expected.contains(&binding.clip_id) {
            return Err(TemplateError::clip(
                TemplateErrorKind::UnexpectedTextBinding,
                &binding.clip_id,
                "text binding does not target editable template text",
            ));
        }
        if binding.text.is_empty() {
            return Err(TemplateError::clip(
                TemplateErrorKind::InvalidText,
                &binding.clip_id,
                "template text binding must be non-empty",
            ));
        }
        if result
            .insert(binding.clip_id.clone(), binding.text.as_str())
            .is_some()
        {
            return Err(TemplateError::clip(
                TemplateErrorKind::DuplicateTextBinding,
                &binding.clip_id,
                "editable text is bound more than once",
            ));
        }
    }
    Ok(result)
}
