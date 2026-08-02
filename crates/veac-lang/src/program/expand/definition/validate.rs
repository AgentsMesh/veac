use crate::authoring;

pub(super) fn source(source: &str) -> Result<(), String> {
    let document = authoring::parse(source).map_err(first)?;
    authoring::lower_document(&document).map_err(first)?;
    Ok(())
}

fn first(errors: authoring::Diagnostics) -> String {
    let diagnostic = &errors.as_slice()[0];
    format!("{}: {}", diagnostic.code, diagnostic.message)
}
