use std::borrow::Cow;
use std::ops::Range;

use crate::program::diagnostic::Diagnostic;

use super::{checked_add, ensure, ensure_replacement_available, size_error, SOURCE_MESSAGE};

#[derive(Debug)]
pub(in crate::program::expand) struct Replacements<'path, 'source> {
    path: &'path str,
    source: Cow<'source, str>,
    spans: Vec<Range<usize>>,
    pub(in crate::program::expand) values: Vec<Cow<'source, str>>,
    output_len: usize,
    owned_value_bytes: usize,
    limit: usize,
}

impl<'path, 'source> Replacements<'path, 'source> {
    pub(in crate::program::expand) fn with_owned(
        path: &'path str,
        source: String,
        spans: impl IntoIterator<Item = Range<usize>>,
        limit: usize,
    ) -> Result<Self, Diagnostic> {
        Self::with_source(path, Cow::Owned(source), spans, limit)
    }

    pub(in crate::program::expand) fn with_source(
        path: &'path str,
        source: Cow<'source, str>,
        spans: impl IntoIterator<Item = Range<usize>>,
        limit: usize,
    ) -> Result<Self, Diagnostic> {
        let spans = collect_spans(path, &source, spans)?;
        let removed = spans.iter().try_fold(0usize, |total, span| {
            total.checked_add(span.end - span.start)
        });
        let output_len = removed
            .and_then(|removed| source.len().checked_sub(removed))
            .ok_or_else(|| size_error(path))?;
        ensure(path, output_len, limit, SOURCE_MESSAGE)?;
        Ok(Self {
            path,
            source,
            values: Vec::with_capacity(spans.len()),
            spans,
            output_len,
            owned_value_bytes: 0,
            limit,
        })
    }

    pub(in crate::program::expand) fn len(&self) -> usize {
        self.spans.len()
    }

    pub(in crate::program::expand) fn source(&self) -> &str {
        &self.source
    }

    pub(in crate::program::expand) fn span(&self, index: usize) -> Range<usize> {
        self.spans[index].clone()
    }

    pub(in crate::program::expand) fn push_owned(
        &mut self,
        value: String,
    ) -> Result<(), Diagnostic> {
        self.push(Cow::Owned(value))
    }

    pub(in crate::program::expand) fn push_borrowed(
        &mut self,
        value: &'source str,
    ) -> Result<(), Diagnostic> {
        self.push(Cow::Borrowed(value))
    }

    fn push(&mut self, value: Cow<'source, str>) -> Result<(), Diagnostic> {
        if self.values.len() >= self.spans.len() {
            return Err(invalid_range(self.path));
        }
        let output_len = checked_add(self.path, self.output_len, value.len(), SOURCE_MESSAGE)?;
        let owned_value_bytes = if matches!(value, Cow::Owned(_)) {
            checked_add(
                self.path,
                self.owned_value_bytes,
                value.len(),
                SOURCE_MESSAGE,
            )?
        } else {
            self.owned_value_bytes
        };
        self.ensure_working_set(output_len, owned_value_bytes)?;
        self.output_len = output_len;
        self.owned_value_bytes = owned_value_bytes;
        self.values.push(value);
        Ok(())
    }

    fn ensure_working_set(
        &self,
        output_len: usize,
        owned_value_bytes: usize,
    ) -> Result<(), Diagnostic> {
        let source_bytes = match self.source {
            Cow::Borrowed(_) => 0,
            Cow::Owned(_) => self.source.len(),
        };
        let working = checked_add(self.path, source_bytes, owned_value_bytes, SOURCE_MESSAGE)?;
        let working = checked_add(self.path, working, output_len, SOURCE_MESSAGE)?;
        ensure(self.path, working, self.limit, SOURCE_MESSAGE)
    }

    pub(in crate::program::expand) fn finish_cow(self) -> Result<Cow<'source, str>, Diagnostic> {
        if self.values.len() != self.spans.len() {
            return Err(invalid_range(self.path));
        }
        if self.spans.is_empty() {
            return Ok(self.source);
        }
        let mut output = String::with_capacity(self.output_len);
        let mut cursor = 0usize;
        for (span, value) in self.spans.into_iter().zip(self.values) {
            output.push_str(&self.source[cursor..span.start]);
            output.push_str(&value);
            cursor = span.end;
        }
        output.push_str(&self.source[cursor..]);
        debug_assert_eq!(output.len(), self.output_len);
        Ok(Cow::Owned(output))
    }

    pub(in crate::program::expand) fn finish(self) -> Result<String, Diagnostic> {
        self.finish_cow().map(Cow::into_owned)
    }
}

fn collect_spans(
    path: &str,
    source: &str,
    spans: impl IntoIterator<Item = Range<usize>>,
) -> Result<Vec<Range<usize>>, Diagnostic> {
    let mut collected = Vec::new();
    let mut previous_end = 0usize;
    for span in spans {
        ensure_replacement_available(path, collected.len())?;
        if span.start < previous_end
            || span.start > span.end
            || span.end > source.len()
            || !source.is_char_boundary(span.start)
            || !source.is_char_boundary(span.end)
        {
            return Err(invalid_range(path));
        }
        previous_end = span.end;
        collected.push(span);
    }
    Ok(collected)
}

fn invalid_range(path: &str) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_EXPANSION_BUDGET",
        path,
        "invalid expansion replacement range",
        crate::authoring::Span::default(),
    )
}
