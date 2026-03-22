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

/// Categorized track items extracted from the timeline.
struct TrackCategories<'a> {
    video_items: Vec<&'a IrTrackItem>,
    audio_clips: Vec<&'a IrClip>,
    text_overlays: Vec<&'a IrTextOverlay>,
    image_overlays: Vec<&'a IrImageOverlay>,
    pip_items: Vec<&'a IrPip>,
    subtitle_items: Vec<&'a IrSubtitle>,
}

fn categorize_tracks(ir: &IrProgram) -> TrackCategories<'_> {
    let mut cats = TrackCategories {
        video_items: Vec::new(),
        audio_clips: Vec::new(),
        text_overlays: Vec::new(),
        image_overlays: Vec::new(),
        pip_items: Vec::new(),
        subtitle_items: Vec::new(),
    };
    for track in &ir.timeline.tracks {
        for item in &track.items {
            match track.kind {
                IrTrackKind::Video => cats.video_items.push(item),
                IrTrackKind::Audio => {
                    if let IrTrackItem::Clip(c) = item {
                        cats.audio_clips.push(c);
                    }
                }
                IrTrackKind::Text => {
                    if let IrTrackItem::TextOverlay(t) = item {
                        cats.text_overlays.push(t);
                    }
                }
                IrTrackKind::Overlay => match item {
                    IrTrackItem::ImageOverlay(o) => cats.image_overlays.push(o),
                    IrTrackItem::Pip(p) => cats.pip_items.push(p),
                    IrTrackItem::Subtitle(s) => cats.subtitle_items.push(s),
                    _ => {}
                },
            }
        }
    }
    cats
}

fn register_inputs(cats: &TrackCategories) -> (Vec<InputSpec>, HashMap<String, usize>) {
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

    for item in &cats.video_items {
        match item {
            IrTrackItem::Clip(c) => reg(&c.asset_name, &c.asset_path),
            IrTrackItem::Freeze(f) => reg(&f.asset_name, &f.asset_path),
            _ => {}
        }
    }
    for c in &cats.audio_clips {
        reg(&c.asset_name, &c.asset_path);
    }
    for o in &cats.image_overlays {
        reg(&o.asset_name, &o.asset_path);
    }
    for p in &cats.pip_items {
        reg(&p.asset_name, &p.asset_path);
    }

    (inputs, map)
}

/// Generate an `FfmpegCommand` from validated IR and a desired output path.
pub fn generate(ir: &IrProgram, output_path: &Path) -> FfmpegCommand {
    let mut graph = FilterGraph::new();
    let cats = categorize_tracks(ir);
    let (inputs, input_map) = register_inputs(&cats);
    let mut map_args = Vec::new();

    if !cats.video_items.is_empty() {
        let (vout, aout) = clips::build_clip_filters(
            &cats.video_items,
            &input_map,
            &mut graph,
            ir.project.width,
            ir.project.height,
            ir.project.fps,
        );

        // Apply letterbox padding if fit mode is Letterbox
        let vout = if ir.project.fit == FitMode::Letterbox {
            graph.add_pad(&vout, ir.project.width, ir.project.height, "black")
        } else {
            vout
        };

        let v = overlays::apply_pip_overlays(
            &cats.pip_items,
            &vout,
            &input_map,
            &mut graph,
            ir.project.width,
            ir.project.height,
        );
        let v = overlays::apply_image_overlays(&cats.image_overlays, &v, &input_map, &mut graph);
        let v = overlays::apply_subtitles(&cats.subtitle_items, &v, &mut graph);
        let v = overlays::apply_text_overlays(&cats.text_overlays, &v, &mut graph);
        map_args.push(format!("[{v}]"));

        // Audio mixing
        if !cats.audio_clips.is_empty() {
            let mut audio_labels = vec![aout];
            for clip in &cats.audio_clips {
                let idx = input_map[&clip.asset_name];
                let mut a = graph.add_atrim(&format!("{idx}:a"), clip.from_sec, clip.to_sec);
                if let Some(vol) = clip.volume {
                    a = graph.add_volume(&a, vol);
                }
                if let Some(fi) = clip.fade_in_sec {
                    a = graph.add_afade(&a, "in", 0.0, fi);
                }
                if let Some(fo) = clip.fade_out_sec {
                    a = graph.add_afade(&a, "out", 0.0, fo);
                }
                audio_labels.push(a);
            }
            if audio_labels.len() == 1 {
                map_args.push(format!("[{}]", audio_labels[0]));
            } else {
                let n = audio_labels.len();
                map_args.push(format!("[{}]", graph.add_amix(&audio_labels, n)));
            }
        } else {
            map_args.push(format!("[{aout}]"));
        }
    }

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
