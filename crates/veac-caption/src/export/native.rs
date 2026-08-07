pub(super) mod ass;
pub(super) mod srt;
pub(super) mod webvtt;

pub(super) fn millis_timestamp(value: u64, separator: char) -> String {
    let hours = value / 3_600_000;
    let minutes = value / 60_000 % 60;
    let seconds = value / 1_000 % 60;
    let millis = value % 1_000;
    format!("{hours:02}:{minutes:02}:{seconds:02}{separator}{millis:03}")
}

pub(super) fn ass_timestamp(value: u64) -> String {
    let hours = value / 3_600_000;
    let minutes = value / 60_000 % 60;
    let seconds = value / 1_000 % 60;
    let centis = value % 1_000 / 10;
    format!("{hours}:{minutes:02}:{seconds:02}.{centis:02}")
}
