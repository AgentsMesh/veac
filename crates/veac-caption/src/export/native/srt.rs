use std::fmt::Write;

use subtitler::model::Subtitle;
use veac_ir::CaptionNativeCue;

use crate::CaptionCue;

use super::millis_timestamp;

pub(crate) fn render(subtitles: &[Subtitle], cues: &[CaptionCue]) -> String {
    let mut output = String::new();
    for (position, (subtitle, cue)) in subtitles.iter().zip(cues).enumerate() {
        let index = match &cue.native {
            Some(CaptionNativeCue::Srt { index }) => *index,
            _ => position as u64 + 1,
        };
        let _ = writeln!(output, "{index}");
        let _ = writeln!(
            output,
            "{} --> {}",
            millis_timestamp(subtitle.start, ','),
            millis_timestamp(subtitle.end, ',')
        );
        let _ = writeln!(output, "{}", subtitle.text);
        if position + 1 < subtitles.len() {
            output.push('\n');
        }
    }
    output
}
