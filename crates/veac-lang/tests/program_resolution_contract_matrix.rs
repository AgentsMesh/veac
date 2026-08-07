pub use veac_lang::program;
pub use veac_lang::source_edit;

#[path = "support/function_resolution_contract_matrix.rs"]
mod function_resolution_contract_matrix;
#[path = "support/resolution_contract_matrix.rs"]
mod resolution_contract_matrix;
#[path = "support/source_index_contract_matrix.rs"]
mod source_index_contract_matrix;
#[path = "support/type_registry_contract_matrix.rs"]
mod type_registry_contract_matrix;
