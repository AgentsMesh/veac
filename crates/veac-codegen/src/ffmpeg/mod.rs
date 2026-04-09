/// FFmpeg command generation — core struct and entry point.
mod clips;
mod output;
mod overlays;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use veac_lang::ir::{
    FitMode, IrClip, IrImageOverlay, IrPip, IrProgram, IrSubtitle, IrTextOverlay, IrTrackItem,
    IrTrackKind,
};

use crate::filter_graph::FilterGraph;

/// A complete FFmpeg invocation ready to be serialized to CLI arguments.
#[derive(Debug, Clone)]
pub struct FfmpegCommand {
    pub inputs: Vec<InputSpec>,
    pub filter_graph: Option<String>,
    pub map_args: Vec<String>,
    pub output_args: Vec<String>,
    pub output_path: PathBuf,
}

/// A single `-i` input.
#[derive(Debug, Clone)]
pub struct InputSpec {
    pub path: PathBuf,
}

// ---------------------------------------------------------------------------
// Track Plan: early categorization + audio routing decision
// ---------------------------------------------------------------------------

/// How audio from video clips should be handled.
#[derive(Debug, Clone, Copy, PartialEq)]
enum AudioRouting {
    /// No separate audio track — keep video's extracted audio.
    Keep,
    /// Separate audio track exists — discard video's audio.
    Discard,
}

/// Pre-categorized track items with routing decisions.
struct TrackPlan<'a> {
    video_items: Vec<&'a IrTrackItem>,
    audio_clips: Vec<&'a IrClip>,
    text_overlays: Vec<&'a IrTextOverlay>,
    image_overlays: Vec<&'a IrImageOverlay>,
    pip_items: Vec<&'a IrPip>,
    subtitle_items: Vec<&'a IrSubtitle>,
    audio_routing: AudioRouting,
}

impl<'a> TrackPlan<'a> {
    fn from_ir(ir: &'a IrProgram) -> Self {
        let mut plan = Self {
            video_items: Vec::new(),
            audio_clips: Vec::new(),
            text_overlays: Vec::new(),
            image_overlays: Vec::new(),
            pip_items: Vec::new(),
            subtitle_items: Vec::new(),
            audio_routing: AudioRouting::Keep,
        };
        for track in &ir.timeline.tracks {
            for item in &track.items {
                match track.kind {
                    IrTrackKind::Video => plan.video_items.push(item),
                    IrTrackKind::Audio => {
                        if let IrTrackItem::Clip(c) = item {
                            plan.audio_clips.push(c);
                        }
                    }
                    IrTrackKind::Text => {
                        if let IrTrackItem::TextOverlay(t) = item {
                            plan.text_overlays.push(t);
                        }
                    }
                    IrTrackKind::Overlay => match item {
                        IrTrackItem::ImageOverlay(o) => plan.image_overlays.push(o),
                        IrTrackItem::Pip(p) => plan.pip_items.push(p),
                        IrTrackItem::Subtitle(s) => plan.subtitle_items.push(s),
                        _ => {}
                    },
                }
            }
        }
        // Routing decision: if user has a separate audio track, discard video audio.
        if !plan.audio_clips.is_empty() {
            plan.audio_routing = AudioRouting::Discard;
        }
        plan
    }
}

// ---------------------------------------------------------------------------
// Input registration
// ---------------------------------------------------------------------------

fn register_inputs(plan: &TrackPlan) -> (Vec<InputSpec>, HashMap<String, usize>) {
    let mut inputs = Vec::new();
    let mut map: HashMap<String, usize> = HashMap::new();

    let mut reg = |name: &str, path: &Path| {
        if !map.contains_key(name) {
            map.insert(name.to_string(), inputs.len());
            inputs.push(InputSpec {
                path: path.to_path_buf(),
            });
        }
    };

    for item in &plan.video_items {
        match item {
            IrTrackItem::Clip(c) => reg(&c.asset_name, &c.asset_path),
            IrTrackItem::Freeze(f) => reg(&f.asset_name, &f.asset_path),
            _ => {}
        }
    }
    for c in &plan.audio_clips {
        reg(&c.asset_name, &c.asset_path);
    }
    for o in &plan.image_overlays {
        reg(&o.asset_name, &o.asset_path);
    }
    for p in &plan.pip_items {
        reg(&p.asset_name, &p.asset_path);
    }

    (inputs, map)
}

// ---------------------------------------------------------------------------
// Unified audio effect pipeline
// ---------------------------------------------------------------------------

/// Apply audio effects to a single audio clip label.
/// Used by both video-track audio and audio-track clips — single code path.
fn apply_audio_effects(
    mut a: String,
    clip: &IrClip,
    graph: &mut FilterGraph,
) -> String {
    let clip_dur = clip
        .resolved_duration
        .unwrap_or_else(|| clips::estimate_audio_clip_duration(clip));

    if let Some(vol) = clip.volume {
        a = graph.add_volume(&a, vol);
    }
    if let Some(spd) = clip.speed {
        a = graph.add_atempo(&a, spd);
    }
    if let Some(fi) = clip.fade_in_sec {
        a = graph.add_afade(&a, "in", 0.0, fi);
    }
    if let Some(fo) = clip.fade_out_sec {
        let start = (clip_dur - fo).max(0.0);
        a = graph.add_afade(&a, "out", start, fo);
    }
    if clip.reverse == Some(true) {
        a = graph.add_areverse(&a);
    }
    if clip.normalize == Some(true) {
        a = graph.add_loudnorm(&a);
    }
    a
}

// ---------------------------------------------------------------------------
// Audio track pipeline: build audio from separate audio track clips
// ---------------------------------------------------------------------------

fn build_audio_track(
    audio_clips: &[&IrClip],
    input_map: &HashMap<String, usize>,
    graph: &mut FilterGraph,
) -> String {
    let mut audio_labels = Vec::new();
    for clip in audio_clips {
        let idx = input_map[&clip.asset_name];
        let a = graph.add_atrim(&format!("{idx}:a"), clip.from_sec, clip.to_sec);
        let a = apply_audio_effects(a, clip, graph);
        audio_labels.push(a);
    }
    if audio_labels.len() == 1 {
        audio_labels.remove(0)
    } else {
        let n = audio_labels.len();
        graph.add_amix(&audio_labels, n)
    }
}

// ---------------------------------------------------------------------------
// Main entry point: pipeline-style generate()
// ---------------------------------------------------------------------------

/// Generate an `FfmpegCommand` from validated IR and a desired output path.
pub fn generate(ir: &IrProgram, output_path: &Path) -> FfmpegCommand {
    let mut graph = FilterGraph::new();
    let mut map_args = Vec::new();

    // Stage 1: Plan — categorize tracks and decide audio routing
    let plan = TrackPlan::from_ir(ir);

    // Stage 2: Register inputs
    let (inputs, input_map) = register_inputs(&plan);

    // Stage 3: Build video pipeline
    if !plan.video_items.is_empty() {
        // Build video + video-audio clip filters
        let skip_audio = plan.audio_routing == AudioRouting::Discard;
        let (vout, aout) = clips::build_clip_filters(
            &plan.video_items,
            &input_map,
            &mut graph,
            ir.project.width,
            ir.project.height,
            ir.project.fps,
            skip_audio,
        );

        // Letterbox padding
        let vout = if ir.project.fit == FitMode::Letterbox {
            graph.add_pad(&vout, ir.project.width, ir.project.height, "black")
        } else {
            vout
        };

        // Apply overlays
        let v = overlays::apply_pip_overlays(
            &plan.pip_items,
            &vout,
            &input_map,
            &mut graph,
            ir.project.width,
            ir.project.height,
        );
        let v = overlays::apply_image_overlays(&plan.image_overlays, &v, &input_map, &mut graph);
        let v = overlays::apply_subtitles(&plan.subtitle_items, &v, &mut graph);
        let v = overlays::apply_text_overlays(&plan.text_overlays, &v, &mut graph);
        map_args.push(format!("[{v}]"));

        // Stage 4: Audio routing
        match plan.audio_routing {
            AudioRouting::Discard => {
                // aout is silence (skip_audio=true), sink it
                graph.add_anullsink(&aout);
                // Build audio from separate audio track
                let audio_out = build_audio_track(&plan.audio_clips, &input_map, &mut graph);
                map_args.push(format!("[{audio_out}]"));
            }
            AudioRouting::Keep => {
                // Use video's extracted audio
                map_args.push(format!("[{aout}]"));
            }
        }
    }

    // Stage 5: Build output args
    let output_args = output::build_output_args(&ir.project);
    let filter_str = if graph.is_empty() {
        None
    } else {
        Some(graph.render())
    };

    FfmpegCommand {
        inputs,
        filter_graph: filter_str,
        map_args,
        output_args,
        output_path: output_path.to_path_buf(),
    }
}

/// Generate multiple `FfmpegCommand`s for multi-output support.
pub fn generate_all(ir: &IrProgram, default_output: &Path) -> Vec<FfmpegCommand> {
    if ir.outputs.is_empty() {
        return vec![generate(ir, default_output)];
    }
    ir.outputs
        .iter()
        .map(|cfg| {
            let mut m = ir.clone();
            if let Some(w) = cfg.width {
                m.project.width = w;
            }
            if let Some(h) = cfg.height {
                m.project.height = h;
            }
            if let Some(f) = cfg.format {
                m.project.format = f;
            }
            if let Some(c) = cfg.codec {
                m.project.codec = c;
            }
            if let Some(q) = cfg.quality {
                m.project.quality = q;
            }
            generate(&m, &cfg.path)
        })
        .collect()
}
