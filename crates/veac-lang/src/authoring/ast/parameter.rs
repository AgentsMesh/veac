super::define_syntax_tokens! {
    pub enum InterpolationKind {
        Hold => "hold",
        Linear => "linear",
        EaseIn => "ease-in",
        EaseOut => "ease-out",
        EaseInOut => "ease-in-out",
        CubicBezier => "cubic-bezier",
        Spring => "spring",
    }
}

impl InterpolationKind {
    pub const LEAF_TOKENS: &'static [&'static str] = &[
        Self::Hold.as_str(),
        Self::Linear.as_str(),
        Self::EaseIn.as_str(),
        Self::EaseOut.as_str(),
        Self::EaseInOut.as_str(),
    ];
}
