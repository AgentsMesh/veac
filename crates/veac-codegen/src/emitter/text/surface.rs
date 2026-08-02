use veac_plan::canonical::TextOverflow;
use veac_plan::{EffectiveVisualProperties, ResolvedTextStyle};

use crate::emitter::canvas::Canvas;
use crate::emitter::{geometry, visual_frame};

#[derive(Clone, Copy)]
pub(super) struct Surface {
    pub width: u32,
    pub height: u32,
    pub origin: (f64, f64),
    pub source_geometry: visual_frame::SourceGeometry,
}

impl Surface {
    pub fn dimensions(self) -> (u32, u32) {
        (self.width, self.height)
    }
}

pub(super) fn resolve(
    canvas: Canvas,
    visual: &EffectiveVisualProperties,
    style: &ResolvedTextStyle,
) -> Surface {
    let anchor = visual.transform.anchor;
    let layout = style.layout;
    if let (Some(width), Some(height)) = (layout.box_width_pixels, layout.box_height_pixels) {
        if layout.overflow == TextOverflow::Visible {
            let dimensions = (canvas.width, canvas.height);
            let origin = (
                (f64::from(canvas.width) - width) / 2.0,
                (f64::from(canvas.height) - height) / 2.0,
            );
            return Surface {
                width: canvas.width,
                height: canvas.height,
                origin,
                source_geometry: visual_frame::SourceGeometry::visible_overflow(
                    dimensions,
                    origin,
                    (width, height),
                    anchor,
                ),
            };
        }
        return Surface {
            width: width.round().max(1.0) as u32,
            height: height.round().max(1.0) as u32,
            origin: (0.0, 0.0),
            source_geometry: visual_frame::SourceGeometry::framed(anchor),
        };
    }
    let (width, height) = match visual.frame {
        Some(frame) => (
            geometry::pixel_count(frame.width, canvas.width),
            geometry::pixel_count(frame.height, canvas.height),
        ),
        None => (canvas.width, canvas.height),
    };
    Surface {
        width,
        height,
        origin: (0.0, 0.0),
        source_geometry: visual_frame::SourceGeometry::already_sized(anchor),
    }
}
