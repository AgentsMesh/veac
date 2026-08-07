super::define_syntax_tokens! {
    pub enum ModifierKind {
        Layout => "layout",
        Transform => "transform",
        Composite => "composite",
        Surface => "surface",
        Mask => "mask",
        Audio => "audio",
        Color => "color",
        Effect => "effect",
    }
}

super::define_syntax_tokens! {
    pub enum PlacementKind {
        Anchor => "anchor",
        Absolute => "absolute",
    }
}
