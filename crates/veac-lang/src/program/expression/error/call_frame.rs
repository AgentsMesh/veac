use std::ops::Range;

use super::super::FunctionOrigin;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionCallFrame {
    function_name: String,
    origin: Option<FunctionOrigin>,
    span: Range<usize>,
}

impl ExpressionCallFrame {
    pub(crate) fn new(
        function_name: impl Into<String>,
        origin: Option<&FunctionOrigin>,
        span: Range<usize>,
    ) -> Self {
        Self {
            function_name: function_name.into(),
            origin: origin.cloned(),
            span,
        }
    }

    pub fn function_name(&self) -> &str {
        &self.function_name
    }

    pub fn origin(&self) -> Option<&FunctionOrigin> {
        self.origin.as_ref()
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    pub fn authored_span(&self) -> Option<Range<usize>> {
        self.origin
            .as_ref()
            .map(|origin| origin.absolute_span(self.span()))
    }
}
