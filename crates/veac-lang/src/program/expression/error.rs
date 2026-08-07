use std::fmt;
use std::ops::Range;

use super::{ExpressionLoopFrame, FunctionOrigin};

mod call_frame;
pub use call_frame::ExpressionCallFrame;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionError {
    code: &'static str,
    message: String,
    span: Range<usize>,
    function: Option<Box<FunctionMetadata>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct FunctionMetadata {
    function_name: Option<String>,
    authored_origin: Option<FunctionOrigin>,
    call_frames: Vec<ExpressionCallFrame>,
    loop_frames: Vec<ExpressionLoopFrame>,
}

impl ExpressionError {
    pub(crate) fn new(code: &'static str, message: impl Into<String>, span: Range<usize>) -> Self {
        Self {
            code,
            message: message.into(),
            span,
            function: None,
        }
    }

    pub(crate) fn in_function(mut self, name: impl Into<String>) -> Self {
        if self.function_name().is_none() {
            self.function_mut().function_name = Some(name.into());
        }
        self
    }

    pub(crate) fn in_runtime_function(
        mut self,
        name: &str,
        origin: Option<&FunctionOrigin>,
    ) -> Self {
        if self.function_name().is_none() {
            let function = self.function_mut();
            function.function_name = Some(name.to_owned());
            function.authored_origin = origin.cloned();
        }
        self
    }

    pub(crate) fn called_from(mut self, frame: ExpressionCallFrame) -> Self {
        self.function_mut().call_frames.push(frame);
        self
    }

    pub(crate) fn in_loop_frames(mut self, frames: &[ExpressionLoopFrame]) -> Self {
        if self.loop_frames().is_empty() {
            self.function_mut().loop_frames.extend_from_slice(frames);
        }
        self
    }

    fn function_mut(&mut self) -> &mut FunctionMetadata {
        self.function
            .get_or_insert_with(|| Box::new(FunctionMetadata::default()))
    }

    pub fn code(&self) -> &'static str {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    pub fn function_name(&self) -> Option<&str> {
        self.function
            .as_ref()
            .and_then(|function| function.function_name.as_deref())
    }

    pub fn authored_origin(&self) -> Option<&FunctionOrigin> {
        self.function
            .as_ref()
            .and_then(|function| function.authored_origin.as_ref())
    }

    pub fn authored_span(&self) -> Option<Range<usize>> {
        self.authored_origin()
            .map(|origin| origin.absolute_span(self.span()))
    }

    pub fn call_frames(&self) -> &[ExpressionCallFrame] {
        self.function
            .as_ref()
            .map_or(&[], |function| function.call_frames.as_slice())
    }

    pub fn loop_frames(&self) -> &[ExpressionLoopFrame] {
        self.function
            .as_ref()
            .map_or(&[], |function| function.loop_frames.as_slice())
    }
}

impl fmt::Display for ExpressionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let (Some(origin), Some(span)) = (self.authored_origin(), self.authored_span()) {
            write!(
                formatter,
                "{} at {}:{}..{}: {}",
                self.code,
                origin.source_id(),
                span.start,
                span.end,
                self.message
            )?;
        } else {
            write!(
                formatter,
                "{} at {}..{}: {}",
                self.code, self.span.start, self.span.end, self.message
            )?;
        }
        if let Some(function) = self.function_name() {
            write!(formatter, " (in function `{function}`)")?;
        }
        for frame in self.loop_frames() {
            let binding = frame.binding_span();
            write!(
                formatter,
                "; loop `{}` index = {} binding at {}:{}..{}",
                frame.logical_key().as_str(),
                frame.index(),
                frame.source(),
                binding.start,
                binding.end
            )?;
        }
        for frame in self.call_frames() {
            if let (Some(origin), Some(span)) = (frame.origin(), frame.authored_span()) {
                write!(
                    formatter,
                    "; called from `{}` at {}:{}..{}",
                    frame.function_name(),
                    origin.source_id(),
                    span.start,
                    span.end
                )?;
            } else {
                let span = frame.span();
                write!(
                    formatter,
                    "; called from `{}` at {}..{}",
                    frame.function_name(),
                    span.start,
                    span.end
                )?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for ExpressionError {}
