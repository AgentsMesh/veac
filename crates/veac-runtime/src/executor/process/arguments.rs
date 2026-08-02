use veac_codegen::emitter::BackendCommand;

use crate::input_policy;

pub(in crate::executor) fn for_command(command: &BackendCommand) -> Vec<String> {
    with_input_policy(command, command.to_args())
}

pub(in crate::executor) fn for_filter_script(
    command: &BackendCommand,
    path: &std::path::Path,
) -> Vec<String> {
    with_input_policy(command, command.to_args_with_filter_script(path))
}

fn with_input_policy(command: &BackendCommand, mut arguments: Vec<String>) -> Vec<String> {
    let policy = input_policy::string_arguments();
    let mut cursor = 0;
    for _ in &command.inputs {
        let relative = arguments[cursor..]
            .iter()
            .position(|value| value == "-i")
            .expect("BackendCommand::to_args emits one -i per structured input");
        let index = cursor + relative;
        arguments.splice(index..index, policy.clone());
        cursor = index + policy.len() + 2;
    }
    arguments
}

#[cfg(test)]
mod tests;
