use super::super::use_macro::define_control_uses;

define_control_uses! {
    SOLID_COLOR_FIELD => "color" @ SolidGeneratorMember : FieldIntroducer;
    SHAPE_GEOMETRY_FIELD => "geometry" @ ShapeGeneratorMember : FieldIntroducer;
    SHAPE_FILL_FIELD => "fill" @ ShapeGeneratorMember : FieldIntroducer;
    SHAPE_STROKE_FIELD => "stroke" @ ShapeGeneratorMember : FieldIntroducer;
    RECTANGLE_BOUNDS_FIELD => "bounds" @ RectangleGeometryMember : FieldIntroducer;
    ROUNDED_RECTANGLE_BOUNDS_FIELD => "bounds" @ RoundedRectangleGeometryMember : FieldIntroducer;
    ROUNDED_RECTANGLE_RADIUS_FIELD => "radius" @ RoundedRectangleGeometryMember : FieldIntroducer;
    ELLIPSE_BOUNDS_FIELD => "bounds" @ EllipseGeometryMember : FieldIntroducer;
    POLYGON_POINT_MEMBER => "point" @ PolygonGeometryMember : DeclarationIntroducer;
    PATH_MOVE_MEMBER => "move" @ PathGeometryMember : DeclarationIntroducer;
    PATH_LINE_MEMBER => "line" @ PathGeometryMember : DeclarationIntroducer;
    PATH_CLOSE_MEMBER => "close" @ PathGeometryMember : DeclarationIntroducer;
    LINEAR_FROM_FIELD => "from" @ LinearGradientMember : FieldIntroducer;
    LINEAR_TO_FIELD => "to" @ LinearGradientMember : FieldIntroducer;
    LINEAR_GRADIENT_STOP_MEMBER => "stop" @ LinearGradientMember : DeclarationIntroducer;
    RADIAL_GRADIENT_CENTER_FIELD => "center" @ RadialGradientMember : FieldIntroducer;
    RADIAL_GRADIENT_RADIUS_FIELD => "radius" @ RadialGradientMember : FieldIntroducer;
    RADIAL_GRADIENT_STOP_MEMBER => "stop" @ RadialGradientMember : DeclarationIntroducer;
}
