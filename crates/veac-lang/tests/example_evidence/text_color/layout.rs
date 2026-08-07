use std::collections::BTreeSet;

use veac_ir::{
    HorizontalTextAlignment, TextLayout, TextOrientation, TextOverflow, TextWrap, TextWritingMode,
    VerticalTextAlignment,
};

pub(super) fn evidence(layout: TextLayout, found: &mut BTreeSet<String>) {
    let horizontal = match layout.horizontal_alignment {
        HorizontalTextAlignment::Left => "left",
        HorizontalTextAlignment::Center => "center",
        HorizontalTextAlignment::Right => "right",
    };
    let vertical = match layout.vertical_alignment {
        VerticalTextAlignment::Top => "top",
        VerticalTextAlignment::Middle => "middle",
        VerticalTextAlignment::Bottom => "bottom",
    };
    found.insert(format!("text.layout.align-horizontal-{horizontal}"));
    found.insert(format!("text.layout.align-vertical-{vertical}"));
    if horizontal != "center" || vertical != "middle" {
        found.insert("text.layout.alignment".to_owned());
    }
    let wrap = match layout.wrap {
        TextWrap::None => "none",
        TextWrap::Word => "word",
        TextWrap::Character => "character",
    };
    found.insert(format!("text.layout.wrap-{wrap}"));
    if layout.wrap != TextWrap::None {
        found.insert("text.layout.wrap".to_owned());
    }
    let overflow = match layout.overflow {
        TextOverflow::Visible => "visible",
        TextOverflow::Clip => "clip",
        TextOverflow::Ellipsis => "ellipsis",
    };
    found.insert(format!("text.layout.overflow-{overflow}"));
    if layout.overflow != TextOverflow::Visible {
        found.insert("text.layout.overflow".to_owned());
    }
    let orientation = match layout.orientation {
        TextOrientation::Mixed => "mixed",
        TextOrientation::Upright => "upright",
        TextOrientation::Sideways => "sideways",
    };
    found.insert(format!("text.layout.orientation-{orientation}"));
    match layout.writing_mode {
        TextWritingMode::HorizontalTb => {
            found.insert("text.layout.writing-horizontal".to_owned());
            found.insert("text.layout.writing-horizontal-tb".to_owned());
        }
        TextWritingMode::VerticalRl => {
            found.insert("text.layout.writing-vertical".to_owned());
            found.insert("text.layout.writing-vertical-rl".to_owned());
        }
        TextWritingMode::VerticalLr => {
            found.insert("text.layout.writing-vertical".to_owned());
            found.insert("text.layout.writing-vertical-lr".to_owned());
        }
    }
}
