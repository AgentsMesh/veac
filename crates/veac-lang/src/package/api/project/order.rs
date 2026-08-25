use super::super::ApiExport;

pub(super) fn key(value: &ApiExport) -> (u8, String) {
    match value {
        ApiExport::Function { name, .. } => (0, name.clone()),
        ApiExport::Method { receiver, name, .. } => {
            (1, format!("{}\0{}\0{name}", receiver.module, receiver.name))
        }
        ApiExport::Type { name, .. } => (2, name.clone()),
        ApiExport::Constant { name, .. } => (3, name.clone()),
    }
}
