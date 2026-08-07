super::impl_syntax_tokens!(veac_ir::FontWeight,
    veac_ir::FontWeight::Thin => "thin",
    veac_ir::FontWeight::ExtraLight => "extra-light",
    veac_ir::FontWeight::Light => "light",
    veac_ir::FontWeight::Normal => "normal",
    veac_ir::FontWeight::Medium => "medium",
    veac_ir::FontWeight::SemiBold => "semi-bold",
    veac_ir::FontWeight::Bold => "bold",
    veac_ir::FontWeight::ExtraBold => "extra-bold",
    veac_ir::FontWeight::Black => "black",
);

super::impl_syntax_tokens!(veac_ir::FontStyle,
    veac_ir::FontStyle::Normal => "normal",
    veac_ir::FontStyle::Italic => "italic",
    veac_ir::FontStyle::Oblique => "oblique",
);

super::define_syntax_tokens! {
    pub enum TextFontKind {
        Family => "family",
        Resource => "resource",
    }
}
