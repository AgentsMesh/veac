use super::{DomainMaxStageSpec, DomainOpsetSpec, TemporalLoweringOpcodeSpec};

#[test]
fn machine_contract_publishes_every_temporal_availability() {
    let value = DomainOpsetSpec::current();
    let temporal = value
        .operations
        .iter()
        .filter(|operation| operation.contract.max_stage == DomainMaxStageSpec::Temporal)
        .map(|operation| {
            (
                operation.name.as_str(),
                operation.contract.temporal_lowering,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        temporal,
        vec![
            ("point", Some(TemporalLoweringOpcodeSpec::Point)),
            ("vector", Some(TemporalLoweringOpcodeSpec::Vector)),
            ("rect", Some(TemporalLoweringOpcodeSpec::Rect)),
        ]
    );
    assert_eq!(
        value
            .operations
            .iter()
            .filter(
                |operation| operation.contract.max_stage == DomainMaxStageSpec::Build
                    && operation.contract.temporal_lowering.is_none()
            )
            .count(),
        579
    );
}

#[test]
fn corrupt_temporal_availability_fails_closed() {
    let mut value = DomainOpsetSpec::current();
    let vector = value
        .operations
        .iter_mut()
        .find(|operation| operation.name == "vector")
        .unwrap();
    vector.contract.max_stage = DomainMaxStageSpec::Build;
    assert!(value.validate().is_err());
}
