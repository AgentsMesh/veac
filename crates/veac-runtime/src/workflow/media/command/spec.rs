use std::ffi::OsString;

use veac_artifact::{MediaArtifactSpec, OpticalFlowMethod, OpticalFlowSpec, SourceClockSpec};

use super::{push, ratio, scale, seconds, source_clock, strings, video_encoding};
use crate::workflow::WorkflowResult;

pub(super) fn append(args: &mut Vec<OsString>, spec: &MediaArtifactSpec) -> WorkflowResult<()> {
    match spec {
        MediaArtifactSpec::ProxyVideo(value) => {
            source_clock(args, value.source_clock)?;
            map(args, value.source_stream.global_index);
            args.extend(strings(&["-an", "-vf"]));
            push(args, scale(value.width, value.height));
            args.extend(strings(&["-r"]));
            push(args, ratio(value.frame_rate));
            args.extend(video_encoding(value.crf));
        }
        MediaArtifactSpec::ProxyAudio(value) => {
            source_clock(args, value.source_clock)?;
            map(args, value.source_stream.global_index);
            args.extend(strings(&["-vn", "-c:a", "pcm_s16le", "-ar"]));
            push(args, value.sample_rate.to_string());
            args.extend(strings(&["-ac"]));
            push(args, value.channels.to_string());
        }
        MediaArtifactSpec::Waveform(value) => {
            source_clock(args, value.source_clock)?;
            args.extend(strings(&["-filter_complex"]));
            push(
                args,
                format!(
                    "[0:{}]aresample={},aformat=channel_layouts=mono,showwavespic=s={}x{}:colors={}[wave]",
                    value.source_stream.global_index,
                    value.sample_rate,
                    value.width,
                    value.height,
                    value.color
                ),
            );
            args.extend(strings(&[
                "-map",
                "[wave]",
                "-frames:v",
                "1",
                "-update",
                "1",
            ]));
        }
        MediaArtifactSpec::Thumbnail(value) => {
            args.extend(strings(&["-ss"]));
            push(args, seconds(value.at)?);
            map(args, value.source_stream.global_index);
            args.extend(strings(&["-frames:v", "1", "-vf"]));
            push(args, scale(value.width, value.height));
            args.extend(strings(&["-update", "1"]));
        }
        MediaArtifactSpec::OpticalFlow(value) => {
            let mode = match value.method {
                OpticalFlowMethod::BlockMatching => "obmc",
                OpticalFlowMethod::MotionCompensated => "aobmc",
            };
            map(args, value.source_stream.global_index);
            args.extend(strings(&["-an", "-vf"]));
            push(args, optical_filter(value, mode)?);
            args.extend(video_encoding(18));
        }
        MediaArtifactSpec::SourceSegment(value) => {
            args.extend(strings(&["-ss"]));
            push(args, seconds(value.start)?);
            args.extend(strings(&["-t"]));
            push(args, seconds(value.duration)?);
            map(args, value.video_stream.global_index);
            if let Some(audio) = value.audio {
                map(args, audio.source_stream.global_index);
                args.extend(strings(&["-ar"]));
                push(args, audio.sample_rate.to_string());
                args.extend(strings(&["-ac"]));
                push(args, audio.channels.to_string());
            } else {
                args.extend(strings(&["-an"]));
            }
            args.extend(strings(&["-vf"]));
            push(args, scale(value.width, value.height));
            args.extend(strings(&["-r"]));
            push(args, ratio(value.frame_rate));
            args.extend(video_encoding(value.crf));
            if value.audio.is_some() {
                args.extend(strings(&["-c:a", "aac", "-b:a", "192k"]));
            }
        }
    }
    Ok(())
}

fn optical_filter(value: &OpticalFlowSpec, mode: &str) -> WorkflowResult<String> {
    let (start, duration) = optical_clock(value.source_clock)?;
    // minterpolate buffers its final input frames. Clone two at EOF, then trim away that padding.
    Ok(format!(
        "trim=start={start}:duration={duration},setpts=PTS-STARTPTS,{},tpad=stop_mode=clone:stop=2,minterpolate=fps={}:mi_mode=mci:mc_mode={mode},trim=duration={duration},setpts=PTS-STARTPTS",
        scale(value.width, value.height),
        ratio(value.frame_rate)
    ))
}

fn optical_clock(value: SourceClockSpec) -> WorkflowResult<(String, String)> {
    match value {
        SourceClockSpec::Identity { duration } => Ok(("0".to_owned(), seconds(duration)?)),
        SourceClockSpec::Bounded { logical_range } => Ok((
            seconds(logical_range.start)?,
            seconds(logical_range.duration)?,
        )),
    }
}

fn map(args: &mut Vec<OsString>, index: u32) {
    args.extend(strings(&["-map"]));
    push(args, format!("0:{index}"));
}
