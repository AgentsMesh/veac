use super::{Identifier, NumberLiteral, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorStageDecl {
    Basic(BasicColorDecl),
    Matrix(RgbMatrixDecl),
    Hsl(HslColorDecl),
    Curves(ColorCurvesDecl),
    Wheels(ColorWheelsDecl),
    Lut(LutColorDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicColorDecl {
    pub exposure: NumberLiteral,
    pub highlights: NumberLiteral,
    pub shadows: NumberLiteral,
    pub temperature: NumberLiteral,
    pub tint: NumberLiteral,
    pub fade: NumberLiteral,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbMatrixDecl {
    pub rows: [[NumberLiteral; 3]; 3],
    pub offset: [NumberLiteral; 3],
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HslColorDecl {
    pub range: Identifier,
    pub hue: NumberLiteral,
    pub saturation: NumberLiteral,
    pub lightness: NumberLiteral,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorCurvesDecl {
    pub interpolation: Identifier,
    pub luma: Option<Vec<ColorCurvePointDecl>>,
    pub red: Option<Vec<ColorCurvePointDecl>>,
    pub green: Option<Vec<ColorCurvePointDecl>>,
    pub blue: Option<Vec<ColorCurvePointDecl>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorCurvePointDecl {
    pub input: NumberLiteral,
    pub output: NumberLiteral,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorWheelsDecl {
    pub lift: ColorWheelDecl,
    pub gamma: ColorWheelDecl,
    pub gain: ColorWheelDecl,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorWheelDecl {
    pub red: NumberLiteral,
    pub green: NumberLiteral,
    pub blue: NumberLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LutColorDecl {
    pub resource: Identifier,
    pub interpolation: Identifier,
    pub span: Span,
}
