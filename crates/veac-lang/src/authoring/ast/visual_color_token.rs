super::impl_syntax_tokens!(veac_ir::Anchor,
    veac_ir::Anchor::Center => "center",
    veac_ir::Anchor::TopLeft => "top-left",
    veac_ir::Anchor::Top => "top",
    veac_ir::Anchor::TopRight => "top-right",
    veac_ir::Anchor::Left => "left",
    veac_ir::Anchor::Right => "right",
    veac_ir::Anchor::BottomLeft => "bottom-left",
    veac_ir::Anchor::Bottom => "bottom",
    veac_ir::Anchor::BottomRight => "bottom-right",
);

super::impl_syntax_tokens!(veac_ir::FitMode,
    veac_ir::FitMode::Fill => "fill",
    veac_ir::FitMode::Contain => "contain",
    veac_ir::FitMode::Cover => "cover",
);

super::impl_syntax_tokens!(veac_ir::BlendMode,
    veac_ir::BlendMode::Normal => "normal",
    veac_ir::BlendMode::Multiply => "multiply",
    veac_ir::BlendMode::Screen => "screen",
    veac_ir::BlendMode::Overlay => "overlay",
    veac_ir::BlendMode::Darken => "darken",
    veac_ir::BlendMode::Lighten => "lighten",
    veac_ir::BlendMode::ColorDodge => "color-dodge",
    veac_ir::BlendMode::ColorBurn => "color-burn",
    veac_ir::BlendMode::HardLight => "hard-light",
    veac_ir::BlendMode::SoftLight => "soft-light",
    veac_ir::BlendMode::Difference => "difference",
    veac_ir::BlendMode::Exclusion => "exclusion",
);

super::impl_syntax_tokens!(veac_ir::HueRange,
    veac_ir::HueRange::Red => "red",
    veac_ir::HueRange::Yellow => "yellow",
    veac_ir::HueRange::Green => "green",
    veac_ir::HueRange::Cyan => "cyan",
    veac_ir::HueRange::Blue => "blue",
    veac_ir::HueRange::Magenta => "magenta",
);

super::impl_syntax_tokens!(veac_ir::ToneCurveInterpolation,
    veac_ir::ToneCurveInterpolation::Natural => "natural",
    veac_ir::ToneCurveInterpolation::Monotonic => "monotonic",
);

super::impl_syntax_tokens!(veac_ir::LutInterpolation,
    veac_ir::LutInterpolation::Nearest => "nearest",
    veac_ir::LutInterpolation::Linear => "linear",
    veac_ir::LutInterpolation::Cosine => "cosine",
    veac_ir::LutInterpolation::Cubic => "cubic",
    veac_ir::LutInterpolation::Spline => "spline",
    veac_ir::LutInterpolation::Trilinear => "trilinear",
    veac_ir::LutInterpolation::Tetrahedral => "tetrahedral",
    veac_ir::LutInterpolation::Pyramid => "pyramid",
    veac_ir::LutInterpolation::Prism => "prism",
);
