use std::collections::BTreeMap;

pub(crate) fn vtt_ids(input: &str) -> Vec<Option<String>> {
    let normalized = input.replace("\r\n", "\n");
    normalized
        .split("\n\n")
        .filter_map(|block| {
            let mut lines = block.lines().map(str::trim).filter(|line| !line.is_empty());
            let first = lines.next()?;
            if first.starts_with("WEBVTT")
                || first.starts_with("NOTE")
                || first.starts_with("STYLE")
                || first.starts_with("REGION")
            {
                return None;
            }
            if first.contains("-->") {
                Some(None)
            } else {
                lines
                    .next()
                    .filter(|line| line.contains("-->"))
                    .map(|_| Some(first.to_owned()))
            }
        })
        .collect()
}

pub(crate) fn ass_extras(input: &str) -> Vec<BTreeMap<String, String>> {
    input
        .lines()
        .filter_map(|line| {
            let value = line
                .trim()
                .strip_prefix("Dialogue:")
                .or_else(|| line.trim().strip_prefix("Comment:"))?
                .trim();
            let fields: Vec<_> = value.splitn(10, ',').collect();
            if fields.len() != 10 {
                return Some(BTreeMap::new());
            }
            let mut settings = BTreeMap::new();
            insert_nondefault(&mut settings, "ass.layer", fields[0], "0");
            insert_nondefault(&mut settings, "ass.margin_left", fields[5], "0");
            insert_nondefault(&mut settings, "ass.margin_right", fields[6], "0");
            insert_nondefault(&mut settings, "ass.margin_vertical", fields[7], "0");
            insert_nondefault(&mut settings, "ass.effect", fields[8], "");
            Some(settings)
        })
        .collect()
}

fn insert_nondefault(
    settings: &mut BTreeMap<String, String>,
    key: &str,
    value: &str,
    default: &str,
) {
    let value = value.trim();
    if value != default {
        settings.insert(key.to_owned(), value.to_owned());
    }
}

pub(crate) fn has_unsupported_vtt_markup(input: &str) -> bool {
    ["<c.", "<ruby", "<rt", "<lang", "<00:"]
        .iter()
        .any(|tag| input.contains(tag))
}

pub(crate) fn has_unsupported_ass_override(text: &str) -> bool {
    text.split('{')
        .skip(1)
        .filter_map(|part| part.split_once('}').map(|value| value.0))
        .any(|block| {
            block.split('\\').filter(|tag| !tag.is_empty()).any(|tag| {
                !["b", "i", "u", "c", "r"]
                    .iter()
                    .any(|prefix| tag.starts_with(prefix))
            })
        })
}
