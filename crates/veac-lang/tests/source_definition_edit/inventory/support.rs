use veac_lang::source_edit::{
    ExpressionSite, SourceAudioProcessorField, SourceAudioProcessorKind, SourceColorField,
    SourceDeliveryField, SourceNodeRef, SourcePresetKind, SourceTextLayoutField,
    SourceTextStyleField,
};

use super::super::fixture::{expression, SOURCE};

pub(super) type Case = (SourceNodeRef, ExpressionSite, &'static str);

pub(super) fn preset(
    kind: SourcePresetKind,
    name: &str,
    site: ExpressionSite,
    value: &'static str,
) -> Case {
    node(SourceNodeRef::preset("main.veac", kind, name), site, value)
}

pub(super) fn node(target: SourceNodeRef, site: ExpressionSite, value: &'static str) -> Case {
    (target, site, value)
}

pub(super) fn style(field: SourceTextStyleField) -> ExpressionSite {
    ExpressionSite::PresetTextStyleField { field }
}

pub(super) fn layout(field: SourceTextLayoutField) -> ExpressionSite {
    ExpressionSite::PresetTextLayoutField { field }
}

pub(super) fn color(field: SourceColorField) -> ExpressionSite {
    ExpressionSite::PresetColorField { field }
}

pub(super) fn audio(
    processor_kind: SourceAudioProcessorKind,
    field: SourceAudioProcessorField,
) -> ExpressionSite {
    ExpressionSite::PresetAudioProcessorField {
        processor_kind,
        field,
    }
}

pub(super) fn delivery(field: SourceDeliveryField) -> ExpressionSite {
    ExpressionSite::PresetDeliveryField { field }
}

pub(super) fn parameter(name: &str) -> ExpressionSite {
    ExpressionSite::ModifierParameter {
        parameter: name.into(),
    }
}

pub(super) fn assert_source(
    inventory: &veac_lang::program::SourceIndexInventory,
    target: SourceNodeRef,
    site: ExpressionSite,
    expected: &str,
) {
    let value = expression(inventory, &target, &site);
    assert_eq!(value.source, expected);
    assert_eq!(&SOURCE[value.range.start..value.range.end], expected);
}
