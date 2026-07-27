use veac_ir::{FontRef, TextStyle};

use super::{material::InputUsage, PlanResolver};
use crate::{
    ResolutionDiagnostic, ResolutionErrorKind, ResolvedFont, ResolvedInputKind, ResolvedText,
    ResolvedTextSpan, ResolvedTextStyle,
};

impl PlanResolver<'_> {
    pub(super) fn resolve_text(
        &mut self,
        text: &str,
        style: &TextStyle,
        path: &str,
    ) -> Option<ResolvedText> {
        let font = self.resolve_font(&style.font, &format!("{path}/source/style/font"))?;
        let mut fallback_fonts = Vec::with_capacity(style.fallback_fonts.len());
        for (index, fallback) in style.fallback_fonts.iter().enumerate() {
            fallback_fonts.push(self.resolve_font(
                fallback,
                &format!("{path}/source/style/fallback_fonts/{index}"),
            )?);
        }
        let mut spans = Vec::with_capacity(style.spans.len());
        for (index, span) in style.spans.iter().enumerate() {
            let font = match &span.font {
                Some(font) => Some(
                    self.resolve_font(font, &format!("{path}/source/style/spans/{index}/font"))?,
                ),
                None => None,
            };
            spans.push(ResolvedTextSpan {
                start: span.start,
                end: span.end,
                font,
                font_weight: span.font_weight,
                font_style: span.font_style,
                size_pixels: span.size_pixels,
                color: span.color,
            });
        }
        Some(ResolvedText {
            text: text.to_owned(),
            style: ResolvedTextStyle {
                font,
                fallback_fonts,
                font_weight: style.font_weight,
                font_style: style.font_style,
                size_pixels: style.size_pixels,
                color: style.color,
                tracking_pixels: style.tracking_pixels,
                line_height: style.line_height,
                layout: style.layout,
                path: style.path.clone(),
                background: style.background.clone(),
                outline: style.outline.clone(),
                shadow: style.shadow.clone(),
                spans,
                animation: style.animation.clone(),
            },
        })
    }

    fn resolve_font(&mut self, font: &FontRef, path: &str) -> Option<ResolvedFont> {
        match font {
            FontRef::Material { material_id } => {
                let input = self.material_input(
                    material_id,
                    InputUsage {
                        video: false,
                        audio: false,
                        font: true,
                    },
                )?;
                let ResolvedInputKind::Font {
                    family,
                    postscript_name,
                    face_index,
                } = &input.kind
                else {
                    return None;
                };
                Some(ResolvedFont {
                    requested: font.clone(),
                    input_id: input.id,
                    family: family.clone(),
                    postscript_name: postscript_name.clone(),
                    face_index: *face_index,
                })
            }
            FontRef::Family { family } => {
                self.push(
                    ResolutionDiagnostic::new(
                        ResolutionErrorKind::FontFamilyUnresolved,
                        "FONT_FAMILY_UNRESOLVED",
                        None,
                        path,
                        format!("font family {family:?} has no deterministic font input"),
                    )
                    .repair("use a font material with an authored content identity"),
                );
                None
            }
        }
    }
}
