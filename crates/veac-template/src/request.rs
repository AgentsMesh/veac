use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use veac_ir::{ItemId, Material, OperationId};

pub const TEMPLATE_FILL_SCHEMA_ID: &str = "https://veac.dev/schemas/template-fill-request";
pub const TEMPLATE_FILL_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemplateFillRequest {
    pub schema: String,
    pub schema_version: u32,
    pub operation_id: OperationId,
    #[schemars(range(max = 9007199254740991u64))]
    pub base_revision: u64,
    pub media_bindings: Vec<MediaBinding>,
    pub text_bindings: Vec<TextBinding>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MediaBinding {
    pub clip_id: ItemId,
    pub material: Material,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextBinding {
    pub clip_id: ItemId,
    pub text: String,
}

pub fn template_fill_request_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(TemplateFillRequest))
}

pub fn decode_template_fill_request_json(
    input: &str,
) -> Result<TemplateFillRequest, crate::TemplateError> {
    veac_ir::reject_duplicate_json_keys(input).map_err(invalid_json)?;
    let request: TemplateFillRequest = serde_json::from_str(input).map_err(invalid_json)?;
    validate_header(&request)?;
    Ok(request)
}

pub(crate) fn validate_header(request: &TemplateFillRequest) -> Result<(), crate::TemplateError> {
    if request.schema != TEMPLATE_FILL_SCHEMA_ID
        || request.schema_version != TEMPLATE_FILL_SCHEMA_VERSION
        || OperationId::new(request.operation_id.as_str()).is_err()
        || request.base_revision > veac_ir::MAX_SAFE_INTEGER
    {
        return Err(crate::TemplateError::new(
            crate::TemplateErrorKind::InvalidRequest,
            "template fill request header is invalid or unsupported",
        ));
    }
    Ok(())
}

fn invalid_json(error: serde_json::Error) -> crate::TemplateError {
    crate::TemplateError::new(
        crate::TemplateErrorKind::InvalidRequest,
        format!("invalid template fill request JSON: {error}"),
    )
}
