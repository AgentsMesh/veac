pub(super) fn ass_name(value: &str) -> String {
    value.replace(['\\', '{', '}', ','], "")
}

pub(super) fn filter_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace(':', "\\:")
        .replace(',', "\\,")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace(';', "\\;")
}
