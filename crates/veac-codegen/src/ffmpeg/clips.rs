/// Build filter chains for clips with transitions, speed, and effects.
use std::collections::HashMap;

use veac_lang::ir::{IrClip, IrTrackItem, IrTransition};

use crate::filter_graph::FilterGraph;

/// Build trim + transition + effects filters for video track items.
/// Returns `(video_out_label, audio_out_label)`.
///
/// When `skip_audio` is true, all audio chains use silence instead of
/// extracting audio from video sources. This is used when the user provides
/// a separate `track audio` — the original video audio should be discarded.
pub fn build_clip_filters(
    items: &[&IrTrackItem],
    input_map: &HashMap<String, usize>,
    graph: &mut FilterGraph,
    width: u32,
    height: u32,
    fps: u32,
    skip_audio: bool,
) -> (String, String) {
    let mut segments: Vec<(String, String)> = Vec::new();
    let mut pending_transition: Option<&IrTransition> = None;
    let mut cumulative_duration: f64 = 0.0;

    for item in items {
        match item {
            IrTrackItem::Clip(clip) => {
                let (v_label, a_label) =
                    process_single_clip(clip, input_map, graph, width, height, fps, skip_audio);

                // Handle loop: duplicate segment N times
                let (v_label, a_label) = if let Some(count) = clip.loop_count {
                    if count > 1 {
                        let mut v_labels = vec![v_label.clone()];
                        let mut a_labels = vec![a_label.clone()];
                        for _ in 1..count {
                            let (vl, al) =
                                process_single_clip(clip, input_map, graph, width, height, fps, skip_audio);
                            v_labels.push(vl);
                            a_labels.push(al);
                        }
                        let n = v_labels.len();
                        graph.add_concat(&v_labels, &a_labels, n)
                    } else {
                        (v_label, a_label)
                    }
                } else {
                    (v_label, a_label)
                };

                if let Some(trans) = pending_transition.take() {
                    if let Some(prev) = segments.last_mut() {
                        let offset = (cumulative_duration - trans.duration_sec).max(0.0);
                        prev.0 = graph.add_xfade(
                            &prev.0,
                            &v_label,
                            trans.kind.to_ffmpeg(),
                            trans.duration_sec,
                            offset,
                        );
                        prev.1 = graph.add_acrossfade(&prev.1, &a_label, trans.duration_sec);
                        cumulative_duration += estimate_clip_duration(clip) - trans.duration_sec;
                    }
                } else {
                    cumulative_duration += estimate_clip_duration(clip);
                    segments.push((v_label, a_label));
                }
            }
            IrTrackItem::Transition(t) => {
                pending_transition = Some(t);
            }
            IrTrackItem::Gap(gap) => {
                let v_label = graph.add_color_source(gap.duration_sec, width, height, fps, "black");
                let a_label = graph.add_silence(gap.duration_sec);
                cumulative_duration += gap.duration_sec;
                segments.push((v_label, a_label));
            }
            IrTrackItem::Freeze(freeze) => {
                let idx = input_map[&freeze.asset_name];
                let v_in = format!("{idx}:v");
                let trimmed =
                    graph.add_trim(&v_in, Some(freeze.at_sec), Some(freeze.at_sec + 0.04));
                let scaled = graph.add_scale(&trimmed, &width.to_string(), &height.to_string());
                let v_label = graph.add_tpad(&scaled, freeze.duration_sec);
                let a_label = graph.add_silence(freeze.duration_sec);
                cumulative_duration += freeze.duration_sec;
                segments.push((v_label, a_label));
            }
            _ => {}
        }
    }

    if segments.len() == 1 {
        let seg = segments.remove(0);
        (seg.0, seg.1)
    } else if segments.is_empty() {
        ("0:v".to_string(), "0:a".to_string())
    } else {
        let v_labels: Vec<String> = segments.iter().map(|s| s.0.clone()).collect();
        let a_labels: Vec<String> = segments.iter().map(|s| s.1.clone()).collect();
        graph.add_concat(&v_labels, &a_labels, segments.len())
    }
}

/// Process a single clip: trim, apply video effects, and build audio chain.
fn process_single_clip(
    clip: &IrClip,
    input_map: &HashMap<String, usize>,
    graph: &mut FilterGraph,
    width: u32,
    height: u32,
    fps: u32,
    skip_audio: bool,
) -> (String, String) {
    let idx = input_map[&clip.asset_name];
    let v_in = format!("{idx}:v");
    let v_label = graph.add_trim(&v_in, clip.from_sec, clip.to_sec);
    let v_label = apply_video_effects(clip, v_label, graph, width, height, fps);

    let a_label = if skip_audio {
        // When separate audio track exists, generate silence for video clips
        graph.add_silence(estimate_clip_duration(clip))
    } else {
        // Extract audio from video and apply effects via unified pipeline
        let a = if clip.has_audio {
            graph.add_atrim(&format!("{idx}:a"), clip.from_sec, clip.to_sec)
        } else {
            graph.add_silence(estimate_clip_duration(clip))
        };
        super::apply_audio_effects(a, clip, graph)
    };

    (v_label, a_label)
}

/// Apply the full video effect chain in the correct order.
fn apply_video_effects(
    clip: &IrClip,
    mut v: String,
    graph: &mut FilterGraph,
    width: u32,
    height: u32,
    fps: u32,
) -> String {
    // Order: crop → rotate → flip → scale → zoompan → speed → eq
    //        → blur → sharpen → vignette → grain → opacity
    //        → reverse → chromakey → stabilize
    if let Some(ref spec) = clip.crop {
        v = graph.add_crop(&v, spec);
    }
    if let Some(deg) = clip.rotate {
        v = graph.add_rotate(&v, deg);
    }
    if let Some(ref mode) = clip.flip {
        v = graph.add_flip(&v, mode);
    }

    v = graph.add_scale(&v, &width.to_string(), &height.to_string());

    if let Some(peak) = clip.zoom {
        let dur = estimate_clip_duration_raw(clip);
        v = graph.add_zoompan(&v, peak, dur, width, height, fps, clip.pan_x, clip.pan_y);
    }
    if let Some(spd) = clip.speed {
        v = graph.add_speed(&v, spd);
    }
    if clip.brightness.is_some() || clip.contrast.is_some() || clip.saturation.is_some() {
        v = graph.add_eq(&v, clip.brightness, clip.contrast, clip.saturation);
    }
    if let Some(b) = clip.blur {
        v = graph.add_blur(&v, b);
    }
    if let Some(s) = clip.sharpen {
        v = graph.add_sharpen(&v, s);
    }
    if let Some(val) = clip.vignette {
        v = graph.add_vignette(&v, val);
    }
    if let Some(g) = clip.grain {
        v = graph.add_grain(&v, g);
    }
    if let Some(o) = clip.opacity {
        if o < 1.0 {
            v = graph.add_opacity(&v, o);
        }
    }
    if clip.reverse == Some(true) {
        v = graph.add_reverse(&v);
    }
    if let Some(ref color) = clip.chromakey {
        v = graph.add_chromakey(&v, color);
    }
    if clip.stabilize == Some(true) {
        v = graph.add_deshake(&v);
    }
    v
}

/// Estimate clip duration from IR fields. Used as fallback when
/// `resolved_duration` is not available (probing was skipped).
pub fn estimate_audio_clip_duration(clip: &IrClip) -> f64 {
    estimate_clip_duration(clip)
}

fn estimate_clip_duration(clip: &IrClip) -> f64 {
    // Prefer resolved_duration from MediaResolver probe
    if let Some(d) = clip.resolved_duration {
        return d;
    }
    let base = estimate_clip_duration_raw(clip);
    if let Some(spd) = clip.speed {
        base / spd
    } else {
        base
    }
}

fn estimate_clip_duration_raw(clip: &IrClip) -> f64 {
    if let Some(d) = clip.duration_sec {
        return d;
    }
    if let (Some(from), Some(to)) = (clip.from_sec, clip.to_sec) {
        return to - from;
    }
    10.0
}
