/// Apply text overlays, image overlays, pip overlays, and subtitles onto a video stream.
use std::collections::HashMap;

use veac_lang::ir::FitMode;
use veac_lang::ir::IrImageOverlay;
use veac_lang::ir::IrPip;
use veac_lang::ir::IrSubtitle;
use veac_lang::ir::IrTextOverlay;
use veac_lang::ir::Position;

use crate::filter_graph::FilterGraph;

/// Format a numeric pixel coordinate for an overlay `x=`/`y=` value.
fn fnum(v: f64) -> String {
    format!("{v}")
}

/// Numeric resting position (top-left px) of a `cw`×`ch` box anchored at `pos`, inset by
/// (`mx`,`my`) from its edges. The numeric twin of [`Position::to_overlay_xy`] — required so a
/// drop shadow can be offset from an exact coordinate rather than an `overlay`-relative expr.
fn anchor_num(
    pos: Position,
    out_w: f64,
    out_h: f64,
    cw: f64,
    ch: f64,
    mx: f64,
    my: f64,
) -> (f64, f64) {
    let x_left = mx;
    let x_right = out_w - cw - mx;
    let x_center = (out_w - cw) / 2.0;
    let y_top = my;
    let y_bottom = out_h - ch - my;
    let y_center = (out_h - ch) / 2.0;
    match pos {
        Position::Center => (x_center, y_center),
        Position::TopLeft => (x_left, y_top),
        Position::TopRight => (x_right, y_top),
        Position::BottomLeft => (x_left, y_bottom),
        Position::BottomRight => (x_right, y_bottom),
        Position::Top => (x_center, y_top),
        Position::Bottom => (x_center, y_bottom),
        Position::Left => (x_left, y_center),
        Position::Right => (x_right, y_center),
    }
}

/// Per-frame overlay x/y expressions for an animated (zooming) pip that exactly fills the frame at
/// full size and comes to rest inset by (`mx`, `my`) px from its anchored edges at corner size
/// (`cw`×`ch`). `eval=frame` re-evaluates these as w/h shrink, so the inset grows in lockstep with
/// the zoom: 0 at full frame (no ghost against the base track), full margin at the corner. `mx=my=0`
/// reproduces the flush margin-less anchor. Center anchoring ignores margins.
fn animated_inset_xy(
    pos: Position,
    out_w: f64,
    out_h: f64,
    cw: f64,
    ch: f64,
    mx: f64,
    my: f64,
) -> (String, String) {
    // Constant denominators (how far the pip travels from fill to corner); guard against /0 for a
    // degenerate scale==1 zoom pip.
    let wmt = (out_w - cw).max(1.0);
    let hmt = (out_h - ch).max(1.0);
    let x_left = format!("{mx}*(W-w)/{wmt}");
    let x_right = format!("{}*(W-w)/{wmt}", out_w - cw - mx);
    let x_center = "(W-w)/2".to_string();
    let y_top = format!("{my}*(H-h)/{hmt}");
    let y_bottom = format!("{}*(H-h)/{hmt}", out_h - ch - my);
    let y_center = "(H-h)/2".to_string();
    match pos {
        Position::Center => (x_center, y_center),
        Position::TopLeft => (x_left, y_top),
        Position::TopRight => (x_right, y_top),
        Position::BottomLeft => (x_left, y_bottom),
        Position::BottomRight => (x_right, y_bottom),
        Position::Top => (x_center, y_top),
        Position::Bottom => (x_center, y_bottom),
        Position::Left => (x_left, y_center),
        Position::Right => (x_right, y_center),
    }
}

/// Chain drawtext filters onto a video stream.
/// Supports optional fade_in/fade_out alpha animation.
pub fn apply_text_overlays(
    overlays: &[&IrTextOverlay],
    video_label: &str,
    graph: &mut FilterGraph,
) -> String {
    use veac_lang::ir::Position;
    let mut current = video_label.to_string();
    for ov in overlays {
        let (x, y_anchor) = ov.position.to_ffmpeg_xy();
        // Custom margin overrides the fixed 10px vertical offset for top/bottom positions
        // (e.g. lower-third subtitles clearing a vertical player's bottom UI).
        let y_custom = ov.margin.map(|m| match ov.position {
            Position::Bottom | Position::BottomLeft | Position::BottomRight => {
                format!("h-text_h-{m}")
            }
            Position::Top | Position::TopLeft | Position::TopRight => format!("{m}"),
            _ => y_anchor.to_string(),
        });
        let y_anchored = y_custom.as_deref().unwrap_or(y_anchor);
        // Explicit x/y (px) override the anchor per axis.
        let x_final = ov.x.map(fnum).unwrap_or_else(|| x.to_string());
        let y_final = ov.y.map(fnum).unwrap_or_else(|| y_anchored.to_string());
        let end_sec = ov.at_sec + ov.duration_sec;

        // Build alpha expression for fade in/out
        let alpha_expr = build_text_alpha_expr(ov.at_sec, end_sec, ov.fade_in_sec, ov.fade_out_sec);

        // Use resolved font path if available, otherwise fall back to font name
        let font_ref = ov.resolved_font_path.as_deref().unwrap_or(&ov.font);

        // Fold background box + drop shadow + outline into the drawtext option string.
        let mut style_opts: Vec<String> = Vec::new();
        if let Some(bg) = &ov.background {
            let padding = ov.background_padding.unwrap_or(12);
            style_opts.push(format!("box=1:boxcolor={bg}:boxborderw={padding}"));
        }
        if let Some(sh) = &ov.shadow {
            style_opts.push(format!(
                "shadowx={}:shadowy={}:shadowcolor={}",
                sh.dx, sh.dy, sh.color
            ));
        }
        if let Some(ol) = &ov.outline {
            style_opts.push(format!("borderw={}:bordercolor={}", ol.width, ol.color));
        }
        let box_opts = (!style_opts.is_empty()).then(|| style_opts.join(":"));

        current = graph.add_drawtext_with_alpha(
            &current,
            &ov.content,
            font_ref,
            ov.size,
            &ov.color,
            &x_final,
            &y_final,
            ov.at_sec,
            end_sec,
            alpha_expr.as_deref(),
            box_opts.as_deref(),
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

/// Apply image overlays onto a video stream. Beyond the legacy scale+opacity+anchor path, an
/// image can be sized to an explicit box (`width`/`height`) with a `fit`, given rounded corners
/// (`radius`), faded in/out, and placed at exact `x`/`y` pixels — everything a caption card or a
/// logo needs. (Drop shadows are a pip feature; wrap a shadowed card in a `pip`.)
pub fn apply_image_overlays(
    overlays: &[&IrImageOverlay],
    video_label: &str,
    input_map: &HashMap<String, usize>,
    graph: &mut FilterGraph,
    fps: u32,
) -> String {
    let mut current = video_label.to_string();

    for ov in overlays {
        let idx = input_map[&ov.asset_name];
        let img_in = format!("{idx}:v");
        let end_sec = ov.at_sec + ov.duration_sec;
        let fade_in = ov.fade_in_sec.unwrap_or(0.0);
        let fade_out = ov.fade_out_sec.unwrap_or(0.0);
        let has_fade = fade_in > 0.0 || fade_out > 0.0;

        // Size: an explicit width/height box (with `fit`) overrides the scale multiplier.
        let sized = match (ov.width, ov.height) {
            (Some(w), Some(h)) => {
                let fit = ov.style.fit.unwrap_or(FitMode::Fill);
                graph.add_fit_scale(&img_in, w as u32, h as u32, fit)
            }
            _ => match ov.scale {
                Some(scale) => {
                    graph.add_scale(&img_in, &format!("iw*{scale}"), &format!("ih*{scale}"))
                }
                None => img_in,
            },
        };

        // Rounded corners.
        let carded = match ov.style.radius {
            Some(r) => graph.add_round_corners(&sized, r),
            None => sized,
        };

        // Fade needs real frames: loop the still across its window, ramp alpha, then shift PTS
        // into [at, end]. Opacity applies only on the non-fading path.
        let composited = if has_fade {
            let looped = graph.add_image_loop(&carded, ov.duration_sec, fps);
            let faded = graph.add_alpha_fade(&looped, fade_in, fade_out, ov.duration_sec);
            graph.add_pts_offset(&faded, ov.at_sec)
        } else if let Some(opacity) = ov.opacity.filter(|o| *o < 1.0) {
            let out = graph.next_label("op");
            graph.add(
                vec![carded],
                &format!("format=rgba,colorchannelmixer=aa={opacity}"),
                vec![out.clone()],
            );
            out
        } else {
            carded
        };

        // Position. A scroll animates it linearly across the window (per-frame eval); otherwise
        // explicit x/y (numeric) overrides the anchor per axis.
        if ov.scroll_x != 0.0 || ov.scroll_y != 0.0 {
            let x0 = ov.x.unwrap_or(0.0);
            let y0 = ov.y.unwrap_or(0.0);
            let dur = ov.duration_sec.max(0.0001);
            let xe = format!("{x0}+({})*(t-{})/{dur}", ov.scroll_x, ov.at_sec);
            let ye = format!("{y0}+({})*(t-{})/{dur}", ov.scroll_y, ov.at_sec);
            current = graph.add_overlay_anim(&current, &composited, &xe, &ye, ov.at_sec, end_sec);
        } else {
            let (ax, ay) = ov.position.to_overlay_xy();
            let x = ov.x.map(fnum).unwrap_or_else(|| ax.to_string());
            let y = ov.y.map(fnum).unwrap_or_else(|| ay.to_string());
            current = graph.add_overlay(&current, &composited, &x, &y, ov.at_sec, end_sec);
        }
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

        // Scale pip to the desired size. Explicit width/height (px) override scale — needed for a
        // square overlay on a portrait canvas (scale alone would give a rectangle).
        let pip_w = if pip.width > 0.0 {
            pip.width as u32
        } else {
            ((target_w as f64) * pip.scale) as u32
        };
        let pip_h = if pip.height > 0.0 {
            pip.height as u32
        } else {
            ((target_h as f64) * pip.scale) as u32
        };
        let end_sec = pip.at_sec + pip.duration_sec;
        let has_fade = pip.fade_in_sec > 0.0 || pip.fade_out_sec > 0.0;

        if pip.zoom_in_sec > 0.0 || pip.zoom_out_sec > 0.0 {
            // Animated pip: size interpolates full-frame → corner (`zoom_in`) and back (`zoom_out`),
            // giving a shrink-to-corner / grow-to-full transition. Anchor is margin-less so a
            // full-frame pip fills exactly. PTS shifted to `at` (see add_pts_offset) then overlaid
            // with per-frame position eval.
            let scaled = graph.add_scale_anim(
                &trimmed,
                target_w as f64,
                target_h as f64,
                pip_w as f64,
                pip_h as f64,
                pip.zoom_in_sec,
                pip.zoom_out_sec,
                pip.duration_sec,
            );
            // Fade runs on 0-based local PTS, before the offset shifts it into the timeline window.
            let faded = if has_fade {
                graph.add_alpha_fade(&scaled, pip.fade_in_sec, pip.fade_out_sec, pip.duration_sec)
            } else {
                scaled
            };
            let shifted = graph.add_pts_offset(&faded, pip.at_sec);
            let (x, y) = animated_inset_xy(
                pip.position,
                target_w as f64,
                target_h as f64,
                pip_w as f64,
                pip_h as f64,
                pip.margin_x,
                pip.margin_y,
            );
            current = graph.add_overlay_anim(&current, &shifted, &x, &y, pip.at_sec, end_sec);
        } else {
            // Fit the source into the pip box (aspect-preserving when requested), then optionally
            // round its corners — the two moves that turn a raw widget clip into a card.
            let fit = pip.style.fit.unwrap_or(FitMode::Fill);
            let scaled = graph.add_fit_scale(&trimmed, pip_w, pip_h, fit);
            let carded = match pip.style.radius {
                Some(r) => graph.add_round_corners(&scaled, r),
                None => scaled,
            };
            let faded = if has_fade {
                graph.add_alpha_fade(&carded, pip.fade_in_sec, pip.fade_out_sec, pip.duration_sec)
            } else {
                carded
            };
            // Shift the (trim-reset) PTS to `at` so the overlay plays through its enable window
            // instead of freezing on the source's last frame.
            let shifted = graph.add_pts_offset(&faded, pip.at_sec);
            let anchor = || {
                anchor_num(
                    pip.position,
                    target_w as f64,
                    target_h as f64,
                    pip_w as f64,
                    pip_h as f64,
                    pip.margin_x,
                    pip.margin_y,
                )
            };
            if let Some(sh) = &pip.style.shadow {
                // A shadow needs an exact coordinate to offset from, so resolve position numerically.
                let (ax, ay) = anchor();
                let (cx, cy) = (pip.x.unwrap_or(ax), pip.y.unwrap_or(ay));
                // Split the card: one branch becomes the blurred silhouette laid under the other.
                let (card_a, card_b) = graph.add_split(&shifted);
                let (shadow, pad) = graph.add_card_shadow(&card_b, sh);
                current = graph.add_overlay(
                    &current,
                    &shadow,
                    &fnum(cx - pad + sh.dx),
                    &fnum(cy - pad + sh.dy),
                    pip.at_sec,
                    end_sec,
                );
                current =
                    graph.add_overlay(&current, &card_a, &fnum(cx), &fnum(cy), pip.at_sec, end_sec);
            } else {
                // Position: explicit x/y > margin inset > legacy 10px anchor expression.
                let (x, y) = if pip.x.is_some() || pip.y.is_some() {
                    let (ax, ay) = anchor();
                    (fnum(pip.x.unwrap_or(ax)), fnum(pip.y.unwrap_or(ay)))
                } else if pip.margin_x > 0.0 || pip.margin_y > 0.0 {
                    let (ax, ay) = anchor();
                    (fnum(ax), fnum(ay))
                } else {
                    let (ax, ay) = pip.position.to_overlay_xy();
                    (ax.to_string(), ay.to_string())
                };
                current = graph.add_overlay(&current, &shifted, &x, &y, pip.at_sec, end_sec);
            }
        }
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
