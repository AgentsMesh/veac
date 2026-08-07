use super::super::trivia::{self, has_blank_line, has_newline};
use super::rules::Separator;

pub(super) struct Writer {
    output: String,
    indent: usize,
    continuation: usize,
    line_start: bool,
    braces: Vec<bool>,
}

impl Default for Writer {
    fn default() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            continuation: 0,
            line_start: true,
            braces: Vec::new(),
        }
    }
}

impl Writer {
    pub(super) fn trivia(
        &mut self,
        raw: &str,
        separator: Separator,
        has_previous: bool,
        has_current: bool,
    ) {
        let parsed = trivia::parse(raw);
        if parsed.comments.is_empty() {
            self.separate(separator);
            return;
        }
        let mut standalone = false;
        for comment in parsed.comments {
            standalone = has_newline(comment.leading) || self.line_start && has_previous;
            if standalone {
                self.newline(has_blank_line(comment.leading));
            } else {
                self.space();
            }
            self.raw(comment.text);
            if comment.line {
                self.comment_line_end();
                standalone = true;
            }
        }
        if has_newline(parsed.tail) || standalone && has_current {
            self.newline(has_blank_line(parsed.tail));
        } else if has_current && !has_previous && !parsed.tail.is_empty() {
            self.space();
        } else if has_current {
            self.separate(separator);
        }
    }

    pub(super) fn token(&mut self, value: &str) {
        if self.line_start && value.starts_with('.') {
            self.output.extend(std::iter::repeat_n(
                ' ',
                2 * (self.indent + self.continuation.min(1) + 1),
            ));
            self.line_start = false;
            self.output.push_str(value);
            return;
        }
        self.raw(value);
    }

    pub(super) fn closes_regular_block(&self) -> bool {
        self.braces.last().copied() == Some(true)
    }

    pub(super) fn close_regular_block(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    pub(super) fn open_regular_block(&mut self) {
        self.braces.push(true);
        self.indent += 1;
    }

    pub(super) fn open_interpolation(&mut self) {
        self.braces.push(false);
    }

    pub(super) fn close_brace(&mut self) {
        self.braces.pop();
    }

    pub(super) fn open_continuation(&mut self) {
        self.continuation += 1;
    }

    pub(super) fn close_continuation(&mut self) {
        self.continuation = self.continuation.saturating_sub(1);
    }

    pub(super) fn finish(mut self) -> String {
        while self.output.ends_with('\n') {
            self.output.pop();
        }
        self.output.push('\n');
        self.output
    }

    fn separate(&mut self, separator: Separator) {
        match separator {
            Separator::None => {}
            Separator::Space => self.space(),
            Separator::Line => self.newline(false),
            Separator::BlankLine => self.newline(true),
        }
    }

    fn raw(&mut self, value: &str) {
        if self.line_start {
            self.output.extend(std::iter::repeat_n(
                ' ',
                2 * (self.indent + self.continuation.min(1)),
            ));
            self.line_start = false;
        }
        self.output.push_str(value);
        self.line_start = value.ends_with('\n');
    }

    fn space(&mut self) {
        if !self.line_start && !self.output.ends_with(char::is_whitespace) {
            self.output.push(' ');
        }
    }

    fn newline(&mut self, blank: bool) {
        if self.output.is_empty() {
            self.line_start = true;
            return;
        }
        while self.output.ends_with([' ', '\t', '\r']) {
            self.output.pop();
        }
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        if blank && !self.output.ends_with("\n\n") {
            self.output.push('\n');
        }
        self.line_start = true;
    }

    fn comment_line_end(&mut self) {
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.line_start = true;
    }
}
