use super::{NumberLiteral, PointDecl, Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeneratorDecl {
    Transparent(Span),
    Silence(Span),
    Solid { color: Spanned<String>, span: Span },
    Gradient(GradientGeneratorDecl),
    Shape(ShapeGeneratorDecl),
}

impl GeneratorDecl {
    pub fn span(&self) -> Span {
        match self {
            Self::Transparent(span) | Self::Silence(span) => *span,
            Self::Solid { span, .. } => *span,
            Self::Gradient(value) => value.span,
            Self::Shape(value) => value.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradientGeneratorDecl {
    pub geometry: GradientGeometryDecl,
    pub stops: Vec<GradientStopDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GradientGeometryDecl {
    Linear {
        from: PointDecl,
        to: PointDecl,
    },
    Radial {
        center: PointDecl,
        radius: NumberLiteral,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradientStopDecl {
    pub position: NumberLiteral,
    pub color: Spanned<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeGeneratorDecl {
    pub geometry: ShapeGeometryDecl,
    pub fill: Option<PaintDecl>,
    pub stroke: Option<StrokeDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShapeGeometryDecl {
    Rectangle(BoundsDecl),
    RoundedRectangle {
        bounds: BoundsDecl,
        radius: NumberLiteral,
    },
    Ellipse(BoundsDecl),
    Polygon(Vec<PointDecl>),
    Path(Vec<PathCommandDecl>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundsDecl {
    pub x: NumberLiteral,
    pub y: NumberLiteral,
    pub width: NumberLiteral,
    pub height: NumberLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathCommandDecl {
    Move(PointDecl),
    Line(PointDecl),
    Close(Span),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaintDecl {
    Solid(Spanned<String>),
    Gradient(Box<GradientGeneratorDecl>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrokeDecl {
    pub width: NumberLiteral,
    pub paint: PaintDecl,
    pub span: Span,
}
