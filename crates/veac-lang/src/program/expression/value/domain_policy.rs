use super::{domain_affinity, Value, ValueConstructionError};
use crate::program::TypeRegistry;

impl Value {
    pub fn validate_declared_input(
        &self,
        registry: &TypeRegistry,
    ) -> Result<(), ValueConstructionError> {
        self.validate_nominal_registry(registry)?;
        domain_affinity::affinity(self)?;
        if self.value_type().is_public_input_in(registry) == Some(true) {
            Ok(())
        } else {
            Err(ValueConstructionError::new(
                "VALUE_DOMAIN_PUBLIC_INPUT",
                "domain handles cannot enter declared inputs",
            ))
        }
    }
}
