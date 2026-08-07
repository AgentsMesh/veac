use crate::program::{DomainOperationExposure, DomainOperationRegistry, DomainType};

use super::language_spec;

#[test]
fn standard_library_catalog_is_the_sorted_contract_exposure_projection() {
    let spec = language_spec().standard_library;
    let registry = DomainOperationRegistry::standard();
    for function in &spec.free_functions {
        let contract = registry.lookup_opcode(function.operation_opcode).unwrap();
        assert_eq!(
            contract.exposure(),
            &DomainOperationExposure::FreeFunction {
                name: function.name.clone().into(),
            }
        );
    }
    for method in &spec.methods {
        let contract = registry.lookup_opcode(method.operation_opcode).unwrap();
        assert_eq!(
            contract.exposure(),
            &DomainOperationExposure::Method {
                receiver: DomainType::from_opcode(method.receiver.opcode).unwrap(),
                name: method.name.clone().into(),
            }
        );
    }
    assert!(spec
        .free_functions
        .windows(2)
        .all(|pair| pair[0].name < pair[1].name));
    assert!(spec.methods.windows(2).all(|pair| {
        (pair[0].receiver.opcode, pair[0].name.as_str())
            < (pair[1].receiver.opcode, pair[1].name.as_str())
    }));
}
