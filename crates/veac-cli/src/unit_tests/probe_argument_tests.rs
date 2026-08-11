use std::path::Path;

use clap::error::ErrorKind;

use crate::arguments::{Cli, Command};

#[test]
fn probe_arguments_distinguish_media_and_canonical_material_modes() {
    let direct = Cli::try_parse_from(["veac", "probe", "clip.mp4"]).unwrap();
    assert!(matches!(
        direct.command,
        Command::Probe {
            input,
            material: None,
            ..
        } if input.as_path() == Path::new("clip.mp4")
    ));

    let material =
        Cli::try_parse_from(["veac", "probe", "project.json", "--material", "footage"]).unwrap();
    assert!(matches!(
        material.command,
        Command::Probe {
            input,
            material: Some(id),
            ..
        } if input.as_path() == Path::new("project.json") && id == "footage"
    ));
}

#[test]
fn probe_arguments_reject_missing_inputs_values_and_duplicates() {
    assert_eq!(
        Cli::try_parse_from(["veac", "probe"]).unwrap_err().kind(),
        ErrorKind::MissingRequiredArgument
    );
    assert_eq!(
        Cli::try_parse_from(["veac", "probe", "project.json", "--material"])
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidValue
    );
    assert_eq!(
        Cli::try_parse_from([
            "veac",
            "probe",
            "project.json",
            "--material",
            "first",
            "--material",
            "second",
        ])
        .unwrap_err()
        .kind(),
        ErrorKind::ArgumentConflict
    );
}
