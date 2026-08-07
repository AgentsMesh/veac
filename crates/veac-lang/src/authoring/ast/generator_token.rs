super::define_syntax_tokens! {
    pub enum GradientGeometryKind {
        Linear => "linear",
        Radial => "radial",
    }
}

super::define_syntax_tokens! {
    pub enum ShapeGeometryKind {
        Rectangle => "rectangle",
        RoundedRectangle => "rounded-rectangle",
        Ellipse => "ellipse",
        Polygon => "polygon",
        Path => "path",
    }
}

super::define_syntax_tokens! {
    pub enum PaintKind {
        Solid => "solid",
        Gradient => "gradient",
    }
}
