super::impl_syntax_tokens!(veac_ir::TextWrap,
    veac_ir::TextWrap::None => "none",
    veac_ir::TextWrap::Word => "word",
    veac_ir::TextWrap::Character => "character",
);
super::impl_syntax_tokens!(veac_ir::TextOverflow,
    veac_ir::TextOverflow::Visible => "visible",
    veac_ir::TextOverflow::Clip => "clip",
    veac_ir::TextOverflow::Ellipsis => "ellipsis",
);
super::impl_syntax_tokens!(veac_ir::HorizontalTextAlignment,
    veac_ir::HorizontalTextAlignment::Left => "left",
    veac_ir::HorizontalTextAlignment::Center => "center",
    veac_ir::HorizontalTextAlignment::Right => "right",
);
super::impl_syntax_tokens!(veac_ir::VerticalTextAlignment,
    veac_ir::VerticalTextAlignment::Top => "top",
    veac_ir::VerticalTextAlignment::Middle => "middle",
    veac_ir::VerticalTextAlignment::Bottom => "bottom",
);
super::impl_syntax_tokens!(veac_ir::TextWritingMode,
    veac_ir::TextWritingMode::HorizontalTb => "horizontal-tb",
    veac_ir::TextWritingMode::VerticalRl => "vertical-rl",
    veac_ir::TextWritingMode::VerticalLr => "vertical-lr",
);
super::impl_syntax_tokens!(veac_ir::TextOrientation,
    veac_ir::TextOrientation::Upright => "upright",
    veac_ir::TextOrientation::Sideways => "sideways",
    veac_ir::TextOrientation::Mixed => "mixed",
);
super::impl_syntax_tokens!(veac_ir::TextPathAlignment,
    veac_ir::TextPathAlignment::Start => "start",
    veac_ir::TextPathAlignment::Center => "center",
    veac_ir::TextPathAlignment::End => "end",
);
