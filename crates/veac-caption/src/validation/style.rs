use crate::{validation::duplicates, CaptionDocument, ValidationIssue};

use super::issue;

pub(super) fn validate_styles(document: &CaptionDocument, issues: &mut Vec<ValidationIssue>) {
    if duplicates(document.styles.iter().map(|style| style.id.as_str())) {
        issue(
            issues,
            "$.document.styles",
            "DUPLICATE_ID",
            "style IDs must be unique",
        );
    }
    for (index, style) in document.styles.iter().enumerate() {
        let path = format!("$.document.styles[{index}]");
        if style.id.trim().is_empty() || style.font_family.trim().is_empty() {
            issue(
                issues,
                &path,
                "EMPTY",
                "style ID and font family must be nonempty",
            );
        }
        if style.font_size_pixels == 0 || style.alignment == 0 || style.alignment > 9 {
            issue(
                issues,
                &path,
                "STYLE_RANGE",
                "font size and alignment are out of range",
            );
        }
        if style.border_style != 1 && style.border_style != 3 {
            issue(issues, &path, "STYLE_RANGE", "border style must be 1 or 3");
        }
        let numbers = [
            style.scale_x_percent,
            style.scale_y_percent,
            style.letter_spacing_pixels,
            style.rotation_degrees,
            style.outline_pixels,
            style.shadow_pixels,
        ];
        if numbers.iter().any(|value| !value.is_finite())
            || style.scale_x_percent <= 0.0
            || style.scale_y_percent <= 0.0
            || style.outline_pixels < 0.0
            || style.shadow_pixels < 0.0
        {
            issue(
                issues,
                &path,
                "STYLE_NUMBER",
                "style numbers must be finite and valid",
            );
        }
    }
}
