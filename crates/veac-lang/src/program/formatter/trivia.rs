#[derive(Clone, Copy)]
pub(super) struct Comment<'a> {
    pub(super) leading: &'a str,
    pub(super) text: &'a str,
    pub(super) line: bool,
}

pub(super) struct Trivia<'a> {
    pub(super) comments: Vec<Comment<'a>>,
    pub(super) tail: &'a str,
}

pub(super) fn parse(source: &str) -> Trivia<'_> {
    let bytes = source.as_bytes();
    let mut comments = Vec::new();
    let mut cursor = 0;
    let mut leading = 0;
    while cursor + 1 < bytes.len() {
        let line = bytes[cursor] == b'/' && bytes[cursor + 1] == b'/';
        let block = bytes[cursor] == b'/' && bytes[cursor + 1] == b'*';
        if !line && !block {
            cursor += 1;
            continue;
        }
        let start = cursor;
        cursor += 2;
        if line {
            while cursor < bytes.len() && bytes[cursor] != b'\n' {
                cursor += 1;
            }
        } else {
            while cursor + 1 < bytes.len() && !(bytes[cursor] == b'*' && bytes[cursor + 1] == b'/')
            {
                cursor += 1;
            }
            cursor = (cursor + 2).min(bytes.len());
        }
        comments.push(Comment {
            leading: &source[leading..start],
            text: &source[start..cursor],
            line,
        });
        leading = cursor;
    }
    Trivia {
        comments,
        tail: &source[leading..],
    }
}

pub(super) fn has_newline(value: &str) -> bool {
    value.as_bytes().contains(&b'\n')
}

pub(super) fn has_blank_line(value: &str) -> bool {
    let mut newlines = 0;
    for byte in value.bytes() {
        if byte == b'\n' {
            newlines += 1;
            if newlines >= 2 {
                return true;
            }
        } else if !byte.is_ascii_whitespace() {
            newlines = 0;
        }
    }
    false
}
