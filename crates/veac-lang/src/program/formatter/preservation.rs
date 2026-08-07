use super::super::token::Token;
use super::trivia;

pub(super) fn same_comments(
    left_source: &str,
    left_tokens: &[Token],
    right_source: &str,
    right_tokens: &[Token],
) -> bool {
    comments_by_gap(left_source, left_tokens) == comments_by_gap(right_source, right_tokens)
}

fn comments_by_gap<'a>(source: &'a str, tokens: &[Token]) -> Vec<Vec<&'a str>> {
    let mut offset = 0;
    tokens
        .iter()
        .map(|token| {
            let comments = trivia::parse(&source[offset..token.span.start])
                .comments
                .into_iter()
                .map(|comment| comment.text)
                .collect();
            offset = token.span.end;
            comments
        })
        .collect()
}
