pub use veac_ir::FillerSuggestion;

super::impl_syntax_tokens!(FillerSuggestion,
    FillerSuggestion::Keep => "keep",
    FillerSuggestion::Delete => "delete",
    FillerSuggestion::Tighten => "tighten",
);

super::define_syntax_tokens! {
    pub enum ReviewAction {
        Keep => "keep",
        Remove => "remove",
    }
}

super::define_syntax_tokens! {
    pub enum AnnotationTimingKind {
        Untimed => "untimed",
        Point => "point",
        Range => "range",
    }
}
