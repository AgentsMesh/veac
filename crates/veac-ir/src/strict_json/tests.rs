use std::fmt;

use serde::de::Visitor;

use super::StrictVisitor;

struct Expectation;

impl fmt::Display for Expectation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        <StrictVisitor as Visitor<'static>>::expecting(&StrictVisitor, formatter)
    }
}

#[test]
fn strict_visitor_documents_ijson_and_accepts_option_none() {
    assert_eq!(
        Expectation.to_string(),
        "I-JSON without duplicate object names"
    );
    let none =
        <StrictVisitor as Visitor<'static>>::visit_none::<serde::de::value::Error>(StrictVisitor);
    assert_eq!(none, Ok(()));
}
