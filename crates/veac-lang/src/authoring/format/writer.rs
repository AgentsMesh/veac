#[derive(Default)]
pub(super) struct Writer {
    output: String,
    depth: usize,
}

impl Writer {
    pub(super) fn line(&mut self, value: impl AsRef<str>) {
        self.output.push_str(&"  ".repeat(self.depth));
        self.output.push_str(value.as_ref());
        self.output.push('\n');
    }

    pub(super) fn blank(&mut self) {
        if !self.output.ends_with("\n\n") {
            self.output.push('\n');
        }
    }

    pub(super) fn block(&mut self, header: impl AsRef<str>, body: impl FnOnce(&mut Self)) {
        self.line(format!("{} {{", header.as_ref()));
        self.depth += 1;
        body(self);
        self.depth -= 1;
        self.line("}");
    }

    pub(super) fn finish(mut self) -> String {
        while self.output.ends_with("\n\n") {
            self.output.pop();
        }
        self.output
    }
}
