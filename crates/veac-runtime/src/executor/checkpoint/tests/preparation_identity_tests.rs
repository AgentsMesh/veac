use std::path::PathBuf;

use veac_codegen::emitter::{
    BackendAction, BackendFilterBinding, BackendFilterContract, BackendFilterEscape,
    BackendInternalAccess, BackendOutput, BackendPreparation, BackendProduct,
};

use super::support::{identity_for, task};

#[test]
fn preparation_command_arguments_change_the_checkpoint_identity() {
    let first = prepared(
        "prepare-one",
        "analysis.trf",
        "motion.trf",
        BackendInternalAccess::Produce,
    );
    let second = prepared(
        "prepare-two",
        "analysis.trf",
        "motion.trf",
        BackendInternalAccess::Produce,
    );
    assert_ne!(identity_for(&first).key, identity_for(&second).key);
}

#[test]
fn preparation_declared_outputs_change_the_checkpoint_identity() {
    let first = prepared(
        "prepare",
        "analysis-a.trf",
        "motion.trf",
        BackendInternalAccess::Produce,
    );
    let second = prepared(
        "prepare",
        "analysis-b.trf",
        "motion.trf",
        BackendInternalAccess::Produce,
    );
    assert_ne!(identity_for(&first).key, identity_for(&second).key);
}

#[test]
fn preparation_internal_binding_paths_change_the_checkpoint_identity() {
    let first = prepared(
        "prepare",
        "analysis.trf",
        "motion-a.trf",
        BackendInternalAccess::Produce,
    );
    let second = prepared(
        "prepare",
        "analysis.trf",
        "motion-b.trf",
        BackendInternalAccess::Produce,
    );
    assert_ne!(identity_for(&first).key, identity_for(&second).key);
}

#[test]
fn preparation_internal_binding_access_changes_the_checkpoint_identity() {
    let first = prepared(
        "prepare",
        "analysis.trf",
        "motion.trf",
        BackendInternalAccess::Produce,
    );
    let second = prepared(
        "prepare",
        "analysis.trf",
        "motion.trf",
        BackendInternalAccess::Consume,
    );
    assert_ne!(identity_for(&first).key, identity_for(&second).key);
}

fn prepared(
    argument: &str,
    output: &str,
    binding: &str,
    access: BackendInternalAccess,
) -> veac_codegen::emitter::BackendTask {
    let mut task = task(
        BackendOutput::File(PathBuf::from("master.mp4")),
        BackendProduct::VideoMaster,
    );
    let BackendAction::Ffmpeg(command) = &mut task.action else {
        unreachable!()
    };
    let mut preparation = command.clone();
    preparation.preparations.clear();
    preparation.output_args.push(argument.to_owned());
    preparation.filter_contract = Some(
        BackendFilterContract::new(
            "__VEAC_FILTER_RESOURCE_0000__".to_owned(),
            vec![BackendFilterBinding::internal_file(
                "__VEAC_FILTER_RESOURCE_0000__".to_owned(),
                PathBuf::from(binding),
                access,
                BackendFilterEscape::Quoted,
            )],
        )
        .unwrap(),
    );
    command.preparations = vec![BackendPreparation {
        command: preparation,
        outputs: vec![PathBuf::from(output)],
    }];
    task
}
