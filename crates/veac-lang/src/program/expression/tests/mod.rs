mod domain_surface;
mod errors;
mod exact;
mod execution_budget;
mod functions;
mod functions_architecture;
mod functions_budget;
mod functions_errors;
mod integers;
mod literals;
mod matching_metadata;
mod nominal_closures;
mod operators;
mod references;
mod statement_validation;
mod surface;
mod temporal_parser;
mod vocabulary;

use super::{evaluate, Environment, Value};

fn value(source: &str) -> Value {
    evaluate(source, &Environment::new()).unwrap()
}

fn error(source: &str) -> super::ExpressionError {
    evaluate(source, &Environment::new()).unwrap_err()
}
