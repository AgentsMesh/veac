use veac_lang::source_edit::{
    ExpressionSite, SourceNodeRef, SourcePresetKind, SourceTextLayoutField, SourceTextStyleField,
};

use super::fixture::{expression, source_index};

#[test]
fn nested_text_style_fields_are_independently_addressable() {
    use SourceTextStyleField as Field;
    let inventory = source_index().inventory();
    let target = SourceNodeRef::preset("main.veac", SourcePresetKind::TextStyle, "shared");
    for (field, expected) in [
        (Field::FontFamily, "\"Arial\""),
        (Field::Size, "24px"),
        (Field::Weight, "bold"),
        (Field::FontStyle, "italic"),
        (Field::Fill, "#ffffffff"),
        (Field::Tracking, "1px"),
        (Field::LineHeight, "1.2"),
        (Field::BackgroundColor, "#00000099"),
        (Field::BackgroundPadding, "8px"),
        (Field::OutlineColor, "#000000ff"),
        (Field::OutlineWidth, "1px"),
        (Field::ShadowColor, "#000000ff"),
        (Field::ShadowOpacity, "50%"),
        (Field::ShadowBlur, "4px"),
        (Field::ShadowOffsetX, "1px"),
        (Field::ShadowOffsetY, "2px"),
    ] {
        let site = ExpressionSite::PresetTextStyleField { field };
        assert_eq!(expression(&inventory, &target, &site).source, expected);
    }
}

#[test]
fn text_layout_and_path_fields_are_independently_addressable() {
    use SourceTextLayoutField as Field;
    let inventory = source_index().inventory();
    let target = SourceNodeRef::preset("main.veac", SourcePresetKind::TextLayout, "shared");
    for (field, expected) in [
        (Field::BoxWidth, "320px"),
        (Field::BoxHeight, "80px"),
        (Field::Wrap, "none"),
        (Field::Overflow, "clip"),
        (Field::HorizontalAlign, "center"),
        (Field::VerticalAlign, "middle"),
        (Field::WritingMode, "horizontal-tb"),
        (Field::Orientation, "upright"),
        (Field::PathStartOffset, "0px"),
        (Field::PathReverse, "false"),
        (Field::PathAlign, "start"),
    ] {
        let site = ExpressionSite::PresetTextLayoutField { field };
        assert_eq!(expression(&inventory, &target, &site).source, expected);
    }
}
