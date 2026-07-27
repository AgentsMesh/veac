use veac_ir::{Frame, Length, LengthUnit, Rect, VideoStreamInfo};

fn aspect_fill_crop(
    media_width: f64,
    media_height: f64,
    target_width: f64,
    target_height: f64,
) -> Rect {
    let media_aspect = media_width / media_height;
    let target_aspect = target_width / target_height;
    if media_aspect > target_aspect {
        let width = target_aspect / media_aspect;
        Rect {
            x: (1.0 - width) / 2.0,
            y: 0.0,
            width,
            height: 1.0,
        }
    } else {
        let height = media_aspect / target_aspect;
        Rect {
            x: 0.0,
            y: (1.0 - height) / 2.0,
            width: 1.0,
            height,
        }
    }
}

pub(crate) fn crop(info: VideoStreamInfo, frame: Option<Frame>, canvas: (u32, u32)) -> Rect {
    let mut width = f64::from(info.width) * info.sample_aspect_ratio.numerator as f64
        / f64::from(info.sample_aspect_ratio.denominator);
    let mut height = f64::from(info.height);
    match info.rotation_degrees.rem_euclid(360) {
        0 | 180 => {}
        90 | 270 => std::mem::swap(&mut width, &mut height),
        degrees => {
            let radians = f64::from(degrees).to_radians();
            let cosine = radians.cos().abs();
            let sine = radians.sin().abs();
            (width, height) = (
                width * cosine + height * sine,
                width * sine + height * cosine,
            );
        }
    }
    let (target_width, target_height) = target_size(frame, canvas);
    aspect_fill_crop(width, height, target_width, target_height)
}

fn target_size(frame: Option<Frame>, canvas: (u32, u32)) -> (f64, f64) {
    frame.map_or((f64::from(canvas.0), f64::from(canvas.1)), |frame| {
        (
            length(frame.width, f64::from(canvas.0)),
            length(frame.height, f64::from(canvas.1)),
        )
    })
}

fn length(value: Length, canvas: f64) -> f64 {
    match value.unit {
        LengthUnit::Pixels => value.value,
        LengthUnit::Normalized => value.value * canvas,
        LengthUnit::Percent => value.value * canvas / 100.0,
    }
}
