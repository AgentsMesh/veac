super::define_syntax_tokens! {
    pub enum ColorStageKind {
        Basic => "basic",
        Matrix => "matrix",
        Hsl => "hsl",
        Curves => "curves",
        Wheels => "wheels",
        Lut => "lut",
    }
}

super::define_syntax_tokens! {
    pub enum ColorCurveChannel {
        Luma => "luma",
        Red => "red",
        Green => "green",
        Blue => "blue",
    }
}
