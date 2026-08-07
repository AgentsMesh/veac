use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn text_style(
        &mut self,
        text: &str,
        style: &TextStyle,
        duration: RationalTime,
        timebase: u32,
        path: &str,
        item_id: &str,
    ) {
        if text.len() > MAX_TEXT_BYTES
            || text.chars().count() > MAX_TEXT_SCALARS
            || style.fallback_fonts.len() > MAX_FALLBACK_FONTS
            || style.spans.len() > MAX_TEXT_SPANS
            || invalid_font(&style.font)
            || style.fallback_fonts.iter().any(invalid_font)
            || !style.size_pixels.is_finite()
            || style.size_pixels <= 0.0
            || style.size_pixels > MAX_TEXT_SIZE_PIXELS
            || !style.tracking_pixels.is_finite()
            || style.tracking_pixels.abs() > MAX_TEXT_TRACKING_PIXELS
            || !style.line_height.is_finite()
            || style.line_height <= 0.0
            || style.line_height > MAX_TEXT_LINE_HEIGHT
        {
            self.value_error("TEXT_STYLE", path, item_id);
        }
        self.text_box(&style.layout, path, item_id);
        self.text_path(style, path, item_id);
        self.text_decorations(style, path, item_id);
        self.text_spans(text, &style.spans, path, item_id);
        if let Some(animation) = &style.animation {
            self.text_animation(animation, duration, timebase, path, item_id);
        }
    }

    fn text_box(&mut self, layout: &TextLayout, path: &str, item_id: &str) {
        let inline_bounded = match layout.writing_mode {
            TextWritingMode::HorizontalTb => layout.box_width_pixels.is_some(),
            TextWritingMode::VerticalRl | TextWritingMode::VerticalLr => {
                layout.box_height_pixels.is_some()
            }
        };
        let invalid = !text_box_valid(layout.box_width_pixels, layout.box_height_pixels)
            || (layout.wrap != TextWrap::None && !inline_bounded)
            || (layout.overflow != TextOverflow::Visible && !inline_bounded);
        if invalid {
            self.value_error("TEXT_LAYOUT", path, item_id);
        }
    }

    fn text_path(&mut self, style: &TextStyle, path: &str, item_id: &str) {
        let Some(text_path) = &style.path else {
            return;
        };
        let pointer = format!("{path}/source/style/path");
        let invalid = !(2..=256).contains(&text_path.points.len())
            || text_path.points.windows(2).any(|pair| pair[0] == pair[1])
            || style.layout.writing_mode != TextWritingMode::HorizontalTb
            || style.layout.wrap != TextWrap::None;
        if invalid {
            self.value_error("TEXT_PATH", &pointer, item_id);
        }
        self.length(
            text_path.start_offset,
            false,
            "TEXT_PATH",
            &format!("{pointer}/start_offset"),
            item_id,
        );
        for (index, point) in text_path.points.iter().enumerate() {
            let point_path = format!("{pointer}/points/{index}");
            self.length(point.x, false, "TEXT_PATH", &point_path, item_id);
            self.length(point.y, false, "TEXT_PATH", &point_path, item_id);
        }
    }

    fn text_decorations(&mut self, style: &TextStyle, path: &str, item_id: &str) {
        let invalid = style.background.as_ref().is_some_and(|background| {
            !background.padding_pixels.is_finite()
                || !(0.0..=MAX_TEXT_PADDING_PIXELS).contains(&background.padding_pixels)
        }) || style.outline.as_ref().is_some_and(|outline| {
            !outline.width_pixels.is_finite()
                || !(0.0..=MAX_TEXT_OUTLINE_PIXELS).contains(&outline.width_pixels)
        });
        if invalid {
            self.value_error("TEXT_STYLE", path, item_id);
        }
        if let Some(shadow) = &style.shadow {
            self.shadow(shadow, "TEXT_STYLE", path, item_id);
        }
    }

    fn text_spans(&mut self, text: &str, spans: &[TextSpan], path: &str, item_id: &str) {
        let length = text.chars().count() as u32;
        let mut previous_end = 0;
        let invalid = spans.iter().any(|span| {
            let invalid = span.start >= span.end
                || span.end > length
                || span.start < previous_end
                || span.font.as_ref().is_some_and(invalid_font)
                || span.size_pixels.is_some_and(|value| {
                    !value.is_finite() || !(0.0..=MAX_TEXT_SIZE_PIXELS).contains(&value)
                });
            previous_end = span.end;
            invalid
        });
        if invalid {
            self.value_error("TEXT_SPAN", path, item_id);
        }
    }

    fn text_animation(
        &mut self,
        animation: &TextAnimation,
        duration: RationalTime,
        timebase: u32,
        path: &str,
        item_id: &str,
    ) {
        self.time(
            animation.stagger,
            timebase,
            false,
            "TEXT_ANIMATION",
            &format!("{path}/source/style/animation/stagger"),
            item_id,
        );
        for (name, curve) in [
            ("reveal", &animation.reveal),
            ("opacity", &animation.opacity),
        ] {
            self.animatable(
                curve,
                duration,
                timebase,
                &format!("{path}/source/style/animation/{name}"),
                item_id,
                |value| value.is_finite() && (0.0..=1.0).contains(value),
            );
        }
        if let Some(highlight) = &animation.highlight {
            self.animatable(
                &highlight.progress,
                duration,
                timebase,
                &format!("{path}/source/style/animation/highlight/progress"),
                item_id,
                |value| value.is_finite() && (0.0..=1.0).contains(value),
            );
        }
        self.animatable(
            &animation.transform.position_offset,
            duration,
            timebase,
            &format!("{path}/source/style/animation/transform/position_offset"),
            item_id,
            |point| point.x.value.is_finite() && point.y.value.is_finite(),
        );
        self.animatable(
            &animation.transform.scale,
            duration,
            timebase,
            &format!("{path}/source/style/animation/transform/scale"),
            item_id,
            |scale| visual_scale_valid(*scale),
        );
        self.animatable(
            &animation.transform.rotation_degrees,
            duration,
            timebase,
            &format!("{path}/source/style/animation/transform/rotation_degrees"),
            item_id,
            |value| value.is_finite(),
        );
    }
}

fn invalid_font(font: &FontRef) -> bool {
    matches!(font, FontRef::Family { family } if family.trim().is_empty())
}
