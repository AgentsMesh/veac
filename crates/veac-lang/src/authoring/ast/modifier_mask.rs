super::define_syntax_tokens! {
    pub enum MaskShapeKind {
        Linear => "linear",
        Mirror => "mirror",
        Circle => "circle",
        Rectangle => "rectangle",
        RoundedRectangle => "rounded-rectangle",
        Ellipse => "ellipse",
        Polygon => "polygon",
        Heart => "heart",
        Star => "star",
        Path => "path",
    }
}
