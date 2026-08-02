use super::budget::{self, MAX_EXPANDED_BYTES};
use crate::program::diagnostic::Diagnostic;

const PROJECT_MESSAGE: &str = "expanded project exceeds 32 MiB";

pub(super) struct Sequences<'source> {
    path: &'source str,
    project: String,
    close: usize,
    values: Vec<Sequence>,
    retained_len: usize,
    output_len: usize,
    limit: usize,
}

struct Sequence {
    id: String,
    body: String,
    content: std::ops::Range<usize>,
}

impl<'source> Sequences<'source> {
    pub(super) fn new(path: &'source str, project: String) -> Result<Self, Diagnostic> {
        Self::with_limit(path, project, MAX_EXPANDED_BYTES)
    }

    fn with_limit(path: &'source str, project: String, limit: usize) -> Result<Self, Diagnostic> {
        let close = project.rfind('}').ok_or_else(|| {
            Diagnostic::new(
                "PROGRAM_PROJECT_BLOCK",
                path,
                "project has no closing brace",
                crate::authoring::Span::default(),
            )
        })?;
        budget::ensure(path, project.len(), limit, PROJECT_MESSAGE)?;
        let output_len = project.len();
        Ok(Self {
            path,
            project,
            close,
            values: Vec::new(),
            retained_len: 0,
            output_len,
            limit,
        })
    }

    pub(super) fn project(&self) -> &str {
        &self.project
    }

    pub(super) fn remaining(&self, external: usize) -> Result<usize, Diagnostic> {
        let retained = budget::checked_add(
            self.path,
            self.project.len(),
            self.retained_len,
            PROJECT_MESSAGE,
        )?;
        let retained = budget::checked_add(self.path, retained, external, PROJECT_MESSAGE)?;
        self.limit
            .checked_sub(retained)
            .ok_or_else(|| project_error(self.path))
    }

    pub(super) fn push(
        &mut self,
        id: &str,
        body: String,
        external: usize,
    ) -> Result<(), Diagnostic> {
        let content = trimmed_range(&body);
        let added = rendered_len(self.path, id, &body[content.clone()])?;
        let next = budget::checked_add(self.path, self.output_len, added, PROJECT_MESSAGE)?;
        let body_and_id = budget::checked_add(self.path, body.len(), id.len(), PROJECT_MESSAGE)?;
        let retained =
            budget::checked_add(self.path, self.retained_len, body_and_id, PROJECT_MESSAGE)?;
        ensure_working_set(
            self.path,
            self.project.len(),
            retained,
            external,
            next,
            self.limit,
        )?;
        self.retained_len = retained;
        self.output_len = next;
        self.values.push(Sequence {
            id: id.to_owned(),
            body,
            content,
        });
        Ok(())
    }

    pub(super) fn finish(self) -> String {
        if self.values.is_empty() {
            return self.project;
        }
        let mut output = String::with_capacity(self.output_len);
        output.push_str(&self.project[..self.close]);
        for sequence in self.values {
            render(&mut output, &sequence);
        }
        output.push_str(&self.project[self.close..]);
        debug_assert_eq!(output.len(), self.output_len);
        output
    }
}

fn ensure_working_set(
    path: &str,
    project_len: usize,
    sequence_len: usize,
    external_len: usize,
    output_len: usize,
    limit: usize,
) -> Result<(), Diagnostic> {
    let inputs = budget::checked_add(path, project_len, sequence_len, PROJECT_MESSAGE)?;
    let inputs = budget::checked_add(path, inputs, external_len, PROJECT_MESSAGE)?;
    let total = budget::checked_add(path, inputs, output_len, PROJECT_MESSAGE)?;
    budget::ensure(path, total, limit, PROJECT_MESSAGE)
}

fn rendered_len(path: &str, id: &str, body: &str) -> Result<usize, Diagnostic> {
    let mut length = 1usize;
    let header = budget::checked_add(path, "sequence ".len(), id.len(), PROJECT_MESSAGE)?;
    let header = budget::checked_add(path, header, " {".len(), PROJECT_MESSAGE)?;
    length = line_len(path, length, header)?;
    if body.is_empty() {
        length = line_len(path, length, 0)?;
    } else {
        for line in body.lines() {
            length = line_len(path, length, line.len())?;
        }
    }
    line_len(path, length, 1)
}

fn line_len(path: &str, total: usize, content: usize) -> Result<usize, Diagnostic> {
    let total = budget::checked_add(path, total, 3, PROJECT_MESSAGE)?;
    budget::checked_add(path, total, content, PROJECT_MESSAGE)
}

fn render(output: &mut String, sequence: &Sequence) {
    output.push_str("\n  sequence ");
    output.push_str(&sequence.id);
    output.push_str(" {\n");
    let body = &sequence.body[sequence.content.clone()];
    if body.is_empty() {
        output.push_str("  \n");
    } else {
        for line in body.lines() {
            output.push_str("  ");
            output.push_str(line);
            output.push('\n');
        }
    }
    output.push_str("  }\n");
}

fn trimmed_range(value: &str) -> std::ops::Range<usize> {
    let start = value.len() - value.trim_start().len();
    let end = value.trim_end().len();
    if start <= end {
        start..end
    } else {
        0..0
    }
}

fn project_error(path: &str) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_EXPANSION_BUDGET",
        path,
        PROJECT_MESSAGE,
        crate::authoring::Span::default(),
    )
}

#[cfg(test)]
mod tests;
