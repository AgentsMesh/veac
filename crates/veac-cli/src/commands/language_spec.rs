use crate::error::{CliError, CliResult};
use crate::{SchemaContract, SchemaFormat};

pub(crate) fn run(schema: bool) -> CliResult {
    crate::fs::write_stdout(&encode(schema)?)
}

pub(crate) fn encode(schema: bool) -> CliResult<String> {
    if schema {
        return super::schema::encode(SchemaContract::LanguageSpec, SchemaFormat::JsonSchema);
    }
    let spec = veac_lang::vocabulary::language_spec();
    let mut json = match serde_json_canonicalizer::to_string(&spec) {
        Ok(json) => json,
        Err(error) => return Err(CliError::new("LANGUAGE_SPEC_ENCODE", error.to_string())),
    };
    json.push('\n');
    Ok(json)
}
