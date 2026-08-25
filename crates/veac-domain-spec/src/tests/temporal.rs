use super::*;

#[test]
fn temporal_availability_is_closed_and_exhaustive() {
    let registry = DomainOperationRegistry::standard();
    let temporal = registry
        .contracts()
        .filter(|contract| contract.max_stage() == Stage::Temporal)
        .map(|contract| (contract.id(), contract.temporal_lowering()))
        .collect::<Vec<_>>();
    assert_eq!(
        temporal,
        vec![
            (
                DomainOperationId::Point,
                Some(TemporalLoweringOpcode::ComposePoint),
            ),
            (
                DomainOperationId::Vector,
                Some(TemporalLoweringOpcode::ComposeVector),
            ),
            (
                DomainOperationId::Rect,
                Some(TemporalLoweringOpcode::ComposeRect),
            ),
        ]
    );
    assert_eq!(
        registry
            .contracts()
            .filter(|contract| {
                contract.max_stage() == Stage::Build && contract.temporal_lowering().is_none()
            })
            .count(),
        579
    );
}
