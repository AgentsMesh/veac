use crate::*;

use super::support::{codes, library};

#[test]
fn library_program_limit_has_an_exact_boundary() {
    let mut value = library();
    value.bindings.clear();
    let program = value.programs[0].clone();
    value.programs = vec![program; MAX_TEMPORAL_PROGRAMS + 1];
    let actual = codes(validate_temporal_library(&value).unwrap_err());
    assert!(actual.iter().any(|code| code == "TEMPORAL_PROGRAM_LIMIT"));
}

#[test]
fn program_input_limit_is_enforced_before_evaluation() {
    let inputs = (0..=MAX_TEMPORAL_INPUTS)
        .map(|index| TemporalInputDeclaration {
            id: TemporalInputId::new(u32::try_from(index).unwrap()),
            value_type: TemporalType::Scalar,
            source: TemporalInputSource::Parameter {
                parameter_id: TemporalParameterId::new(format!("tpm_{index}")).unwrap(),
            },
        })
        .collect();
    let program = super::support::program(
        inputs,
        vec![super::support::literal(0, super::support::scalar(0.0))],
        0,
        TemporalType::Scalar,
    );
    let actual = codes(validate_temporal_program(&program).unwrap_err());
    assert!(actual.iter().any(|code| code == "TEMPORAL_INPUT_LIMIT"));
}
