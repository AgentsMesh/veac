use serde_json::{json, Value};
use veac_codegen::emitter::{
    BackendCommand, BackendFilterBinding, BackendFilterContract, BackendFilterEscape,
    BackendInternalAccess,
};

use crate::executor::output;

pub(super) fn value(command: &BackendCommand) -> Value {
    command
        .filter_contract
        .as_ref()
        .map(contract)
        .unwrap_or(Value::Null)
}

fn contract(value: &BackendFilterContract) -> Value {
    json!({
        "template": value.template(),
        "bindings": value.bindings().iter().map(binding).collect::<Vec<_>>(),
    })
}

fn binding(value: &BackendFilterBinding) -> Value {
    match value {
        BackendFilterBinding::File {
            token,
            path,
            escape,
        } => json!({
            "type": "file",
            "token": token,
            "path": output::path_string(path),
            "escape": escape_name(*escape),
        }),
        BackendFilterBinding::Directory {
            token,
            directory,
            files,
            escape,
        } => json!({
            "type": "directory",
            "token": token,
            "directory": output::path_string(directory),
            "files": files.iter().map(|path| output::path_string(path)).collect::<Vec<_>>(),
            "escape": escape_name(*escape),
        }),
        BackendFilterBinding::InternalFile {
            token,
            path,
            access,
            escape,
        } => json!({
            "type": "internal_file",
            "token": token,
            "path": output::path_string(path),
            "access": access_name(*access),
            "escape": escape_name(*escape),
        }),
    }
}

fn access_name(value: BackendInternalAccess) -> &'static str {
    match value {
        BackendInternalAccess::Produce => "produce",
        BackendInternalAccess::Consume => "consume",
    }
}

fn escape_name(value: BackendFilterEscape) -> &'static str {
    match value {
        BackendFilterEscape::FilterValue => "filter_value",
        BackendFilterEscape::Quoted => "quoted",
    }
}
