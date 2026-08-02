mod errors;
mod exact;
mod literals;
mod operators;
mod references;

use super::{evaluate, Environment, Value};

fn value(source: &str) -> Value {
    evaluate(source, &Environment::new()).unwrap()
}

fn error(source: &str) -> super::ExpressionError {
    evaluate(source, &Environment::new()).unwrap_err()
}
