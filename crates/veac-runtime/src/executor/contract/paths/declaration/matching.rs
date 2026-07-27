use std::path::Path;

use veac_ir::ImageSequencePattern;

use super::Declaration;
use crate::executor::contract::pattern;

pub(in crate::executor::contract) fn overlaps(left: &Declaration, right: &Declaration) -> bool {
    match (left, right) {
        (Declaration::Static(left), Declaration::Static(right)) => left == right,
        (
            Declaration::Static(path),
            Declaration::Pattern {
                parent,
                prefix,
                suffix,
                minimum_width,
            },
        )
        | (
            Declaration::Pattern {
                parent,
                prefix,
                suffix,
                minimum_width,
            },
            Declaration::Static(path),
        ) => {
            same_parent(path, parent)
                && file_name(path).is_some_and(|value| {
                    image_pattern(prefix, suffix, *minimum_width).matches(value)
                })
        }
        (Declaration::Static(path), Declaration::Passlog { parent, prefix })
        | (Declaration::Passlog { parent, prefix }, Declaration::Static(path)) => {
            same_parent(path, parent)
                && pattern::passlog_accepts(prefix, path.file_name().unwrap().as_encoded_bytes())
        }
        (
            Declaration::Pattern {
                parent: left_parent,
                prefix: left_prefix,
                suffix: left_suffix,
                minimum_width: left_width,
            },
            Declaration::Pattern {
                parent: right_parent,
                prefix: right_prefix,
                suffix: right_suffix,
                minimum_width: right_width,
            },
        ) => {
            left_parent == right_parent
                && image_pattern(left_prefix, left_suffix, *left_width).overlaps(image_pattern(
                    right_prefix,
                    right_suffix,
                    *right_width,
                ))
        }
        (
            Declaration::Passlog { parent, prefix },
            Declaration::Pattern {
                parent: other,
                prefix: a,
                suffix: b,
                minimum_width,
            },
        )
        | (
            Declaration::Pattern {
                parent: other,
                prefix: a,
                suffix: b,
                minimum_width,
            },
            Declaration::Passlog { parent, prefix },
        ) => parent == other && pattern::passlog_overlap_pattern(prefix, a, b, *minimum_width),
        (
            Declaration::Passlog { parent, prefix },
            Declaration::Passlog {
                parent: other,
                prefix: other_prefix,
            },
        ) => parent == other && pattern::passlog_overlap(prefix, other_prefix),
    }
}

pub(in crate::executor::contract) fn consumes(declaration: &Declaration, path: &Path) -> bool {
    match declaration {
        Declaration::Static(value) => value == path,
        Declaration::Pattern {
            parent,
            prefix,
            suffix,
            minimum_width,
        } => {
            same_parent(path, parent)
                && file_name(path).is_some_and(|value| {
                    image_pattern(prefix, suffix, *minimum_width).matches(value)
                })
        }
        Declaration::Passlog { parent, prefix } => {
            same_parent(path, parent)
                && pattern::passlog_accepts(prefix, path.file_name().unwrap().as_encoded_bytes())
        }
    }
}

fn same_parent(path: &Path, parent: &Path) -> bool {
    path.parent() == Some(parent)
}

fn file_name(path: &Path) -> Option<&str> {
    path.file_name()?.to_str()
}

fn image_pattern<'a>(
    prefix: &'a [u8],
    suffix: &'a [u8],
    minimum_width: Option<usize>,
) -> ImageSequencePattern<'a> {
    ImageSequencePattern {
        prefix: std::str::from_utf8(prefix).expect("validated pattern prefix"),
        suffix: std::str::from_utf8(suffix).expect("validated pattern suffix"),
        minimum_width,
    }
}
