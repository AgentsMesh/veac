super::impl_syntax_tokens!(veac_ir::PlacementMode,
    veac_ir::PlacementMode::Free => "free",
    veac_ir::PlacementMode::Magnetic => "magnetic",
);
super::define_syntax_tokens! {
    pub enum PlaybackStateDecl { Enabled => "enabled", Disabled => "disabled" }
}
super::define_syntax_tokens! {
    pub enum AudioStateDecl { Audible => "audible", Muted => "muted" }
}
super::define_syntax_tokens! {
    pub enum IsolationStateDecl { Normal => "normal", Solo => "solo" }
}
super::define_syntax_tokens! {
    pub enum EditingStateDecl { Editable => "editable", Locked => "locked" }
}
