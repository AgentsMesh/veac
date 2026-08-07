use super::super::use_macro::define_control_uses;

define_control_uses! {
    RECORD_AT_FIELD => "at" @ RecordSpanMember : FieldIntroducer;
    RECORD_DURATION_FIELD => "duration" @ RecordSpanMember : FieldIntroducer;
    POINT_X_FIELD => "x" @ PointMember : FieldIntroducer;
    POINT_Y_FIELD => "y" @ PointMember : FieldIntroducer;
    VECTOR_X_FIELD => "x" @ VectorMember : FieldIntroducer;
    VECTOR_Y_FIELD => "y" @ VectorMember : FieldIntroducer;
    RECT_X_FIELD => "x" @ RectMember : FieldIntroducer;
    RECT_Y_FIELD => "y" @ RectMember : FieldIntroducer;
    RECT_WIDTH_FIELD => "width" @ RectMember : FieldIntroducer;
    RECT_HEIGHT_FIELD => "height" @ RectMember : FieldIntroducer;
    SHADOW_COLOR_FIELD => "color" @ ShadowMember : FieldIntroducer;
    SHADOW_OPACITY_FIELD => "opacity" @ ShadowMember : FieldIntroducer;
    SHADOW_BLUR_FIELD => "blur" @ ShadowMember : FieldIntroducer;
    SHADOW_OFFSET_FIELD => "offset" @ ShadowMember : FieldIntroducer;
}
