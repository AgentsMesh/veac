super::define_syntax_tokens! {
    pub enum AnnotationKind {
        Marker => "marker",
        Language => "language",
        SceneBoundary => "scene-boundary",
        Scene => "scene",
        Beat => "beat",
        Silence => "silence",
        Filler => "filler",
        Highlight => "highlight",
        Review => "review",
    }
}
