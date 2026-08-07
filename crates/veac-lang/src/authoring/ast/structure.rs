super::define_syntax_tokens! {
    pub enum ApplyScopeKind {
        CompositeBand => "composite-band",
        Layer => "layer",
        Items => "items",
    }
}

super::define_syntax_tokens! {
    pub enum ApplyStageKind {
        Color => "color",
        Effect => "effect",
    }
}
