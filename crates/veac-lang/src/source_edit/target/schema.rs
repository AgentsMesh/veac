use schemars::JsonSchema;

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(transparent, inline)]
pub(super) struct CanonicalNameSchema(
    #[schemars(
        length(min = 1, max = 128),
        regex(pattern = r"^(?!true$|false$)[A-Za-z_][A-Za-z0-9_]*(-[A-Za-z0-9_]+)*$")
    )]
    String,
);
