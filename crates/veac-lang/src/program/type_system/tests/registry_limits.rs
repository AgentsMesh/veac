use std::sync::Arc;

use super::super::*;

#[test]
fn registry_rejects_the_first_definition_above_its_count_limit() {
    let mut builder = TypeRegistryBuilder::new();
    for index in 0..=MAX_TYPE_REGISTRY_DEFINITIONS {
        let definition = Arc::new(TypeDefinition::new(
            format!("module-{index}.veac"),
            format!("Type{index}"),
            TypeDefinitionKind::Struct(StructDefinition::new(Vec::new())),
        ));
        let result = builder.insert(definition);
        if index < MAX_TYPE_REGISTRY_DEFINITIONS {
            result.unwrap();
        } else {
            assert_eq!(result.unwrap_err().code(), "TYPE_REGISTRY_LIMIT");
        }
    }
}
