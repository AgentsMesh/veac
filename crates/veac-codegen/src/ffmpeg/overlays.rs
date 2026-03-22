/// Apply text overlays, image overlays, pip overlays, and subtitles onto a video stream.
use std::collections::HashMap;

use veac_lang::ir::IrImageOverlay;
use veac_lang::ir::IrPip;
use veac_lang::ir::IrSubtitle;
use veac_lang::ir::IrTextOverlay;

use crate::filter_graph::FilterGraph;

/// Chain drawtext filters onto a video stream.
/// Supports optional fade_in/fade_out alpha animation.
pub fn apply_text_overlays(
    overlays: &[&IrTextOverlay],
    video_label: &str,
    graph: &mut FilterGraph,
) -> String {
    let mut current = video_label.to_string();
    for ov in overlays {
        let (x, y) = ov.position.to_ffmpeg_xy();
        let end_sec = ov.at_sec + ov.duration_sec;

        // Build alpha expression for fade in/out
        let alpha_expr = build_text_alpha_expr(ov.at_sec, end_sec, ov.fade_in_sec, ov.fade_out_sec);

        current = graph.add_drawtext_with_alpha(
            &current,
            &ov.content,
            &ov.font,
            ov.size,
            &ov.color,
            x,
            y,
            ov.at_sec,
            end_sec,
            alpha_expr.as_deref(),
        );
    }
    current
}

/// Build FFmpeg alpha expression for text fade in/out.
fn build_text_alpha_expr(
    start: f64,
    end: f64,
    fade_in: Option<f64>,
    fade_out: Option<f64>,
) -> Option<String> {
    match (fade_in, fade_out) {
        (None, None) => None,
        (Some(fi), None) => {
            let fi_end = start + fi;
            Some(format!("if(lt(t\\,{fi_end})\\,(t-{start})/{fi}\\,1)"))
        }
        (None, Some(fo)) => {
            let fo_start = end - fo;
            Some(format!("if(gt(t\\,{fo_start})\\,({end}-t)/{fo}\\,1)"))
        }
        (Some(fi), Some(fo)) => {
            let fi_end = start + fi;
            let fo_start = end - fo;
            Some(format!(
                "if(lt(t\\,{fi_end})\\,(t-{start})/{fi}\\,if(gt(t\\,{fo_start})\\,({end}-t)/{fo}\\,1))"
            ))
        }
    }
}

/// Apply image overlays onto a video stream.
pub fn apply_image_overlays(
    overlays: &[&IrImageOverlay],
    video_label: &str,
    input_map: &HashMap<String, usize>,
    graph: &mut FilterGraph,
) -> String {
    let mut current = video_label.to_string();

    for ov in overlays {
        let idx = input_map[&ov.asset_name];
        let img_in = format!("{idx}:v");

        // Scale the image if scale is specified.
        let scaled = if let Some(scale) = ov.scale {
            let w = format!("iw*{scale}");
            let h = format!("ih*{scale}");
            graph.add_scale(&img_in, &w, &h)
        } else {
            img_in
        };

        // Apply opacity if specified via colorchannelmixer.
        let with_opacity = if let Some(opacity) = ov.opacity {
            if opacity < 1.0 {
                let out = graph.next_label("op");
                let expr = format!("format=rgba,colorchannelmixer=aa={opacity}");
                graph.add(vec![scaled], &expr, vec![out.clone()]);
                out
            } else {
                scaled
            }
        } else {
            scaled
        };

        // Overlay onto video with position and time enable.
        let (x, y) = ov.position.to_overlay_xy();
        let end_sec = ov.at_sec + ov.duration_sec;
        current = graph.add_overlay(&current, &with_opacity, x, y, ov.at_sec, end_sec);
    }

    current
}

/// Apply pip (picture-in-picture) overlays onto a video stream.
pub fn apply_pip_overlays(
    pips: &[&IrPip],
    video_label: &str,
    input_map: &HashMap<String, usize>,
    graph: &mut FilterGraph,
    target_w: u32,
    target_h: u32,
) -> String {
    let mut current = video_label.to_string();

    for pip in pips {
        let idx = input_map[&pip.asset_name];
        let v_in = format!("{idx}:v");

        // Trim the pip source
        let trimmed = graph.add_trim(&v_in, pip.from_sec, pip.to_sec);

        // Scale pip to the desired size
        let pip_w = ((target_w as f64) * pip.scale) as u32;
        let pip_h = ((target_h as f64) * pip.scale) as u32;
        let scaled = graph.add_scale(&trimmed, &pip_w.to_string(), &pip_h.to_string());

        // Overlay pip onto main video with position and time enable
        let (x, y) = pip.position.to_overlay_xy();
        let end_sec = pip.at_sec + pip.duration_sec;
        current = graph.add_overlay(&current, &scaled, x, y, pip.at_sec, end_sec);
    }

    current
}

/// Apply subtitle files onto a video stream.
pub fn apply_subtitles(subs: &[&IrSubtitle], video_label: &str, graph: &mut FilterGraph) -> String {
    let mut current = video_label.to_string();

    for sub in subs {
        let out = graph.next_label("sub");
        let path_str = sub
            .path
            .to_string_lossy()
            .replace('\\', "/")
            .replace('\'', "'\\''");
        let expr = format!("subtitles=filename='{path_str}'");
        graph.add(vec![current], &expr, vec![out.clone()]);
        current = out;
    }

    current
}
