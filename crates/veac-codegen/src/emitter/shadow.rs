use veac_plan::canonical::{shadow_padding, Shadow};

use super::{time, EmitContext};

pub(super) struct Streams {
    pub foreground: String,
    pub shadow: String,
    pub x_delta: f64,
    pub y_delta: f64,
}

pub(super) fn render(
    context: &mut EmitContext<'_>,
    source: &str,
    shadow: &Shadow,
    pivot_x: f64,
    pivot_y: f64,
) -> Streams {
    let (foreground, silhouette) = context.graph.split(source, "shadowsplit");

    let padding = shadow_padding(shadow.blur_pixels).unwrap_or_default();
    let opacity = effective_opacity(shadow);
    let filter = format!(
        "format=gbrap16le,pad=iw+{0}:ih+{0}:{1}:{1}:color=black@0,\
         geq=r={2}:g={3}:b={4}:a='alpha(X,Y)*{5}',\
         gblur=sigma={6}:steps=2:planes=8",
        padding * 2,
        padding,
        channel(shadow.color.red),
        channel(shadow.color.green),
        channel(shadow.color.blue),
        time::number(opacity),
        time::number(shadow.blur_pixels),
    );
    let rendered = context.graph.filter(&[&silhouette], filter, "shadowv");
    Streams {
        foreground,
        shadow: rendered,
        x_delta: placement_delta(shadow.offset.x, padding, pivot_x),
        y_delta: placement_delta(shadow.offset.y, padding, pivot_y),
    }
}

fn channel(value: u8) -> u16 {
    u16::from(value) * 257
}

fn effective_opacity(shadow: &Shadow) -> f64 {
    shadow.opacity * f64::from(shadow.color.alpha) / 255.0
}

fn placement_delta(offset: f64, padding: u64, pivot: f64) -> f64 {
    offset - padding as f64 + 2.0 * padding as f64 * pivot
}

#[cfg(test)]
#[path = "shadow/tests.rs"]
mod tests;
