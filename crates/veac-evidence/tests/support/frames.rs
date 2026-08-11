use veac_evidence::{FrameObservation, PixelFormat, RationalTime};

pub fn rgba(pixels: &[[u8; 4]]) -> FrameObservation {
    FrameObservation {
        width: 4,
        height: 4,
        format: PixelFormat::Rgba8,
        actual_pts: RationalTime {
            value: 0,
            timescale: 30,
        },
        data: pixels.iter().flatten().copied().collect(),
    }
}

pub fn rgb(pixels: &[[u8; 3]]) -> FrameObservation {
    FrameObservation {
        width: 4,
        height: 4,
        format: PixelFormat::Rgb8,
        actual_pts: RationalTime {
            value: 0,
            timescale: 30,
        },
        data: pixels.iter().flatten().copied().collect(),
    }
}

pub fn solid(color: [u8; 4]) -> FrameObservation {
    rgba(&[color; 16])
}

pub fn underlay() -> FrameObservation {
    solid([10, 10, 10, 255])
}

pub fn overlay() -> FrameObservation {
    let mut pixels = [[0, 0, 0, 0]; 16];
    for index in [5, 6, 9, 10] {
        pixels[index] = [250, 20, 20, 255];
    }
    rgba(&pixels)
}

pub fn actual() -> FrameObservation {
    let base = underlay();
    let top = overlay();
    let pixels = (0..16)
        .map(|index| veac_evidence::source_over(pixel(&base, index), pixel(&top, index)))
        .collect::<Vec<_>>();
    rgba(&pixels)
}

pub fn reveal(prefix: usize) -> FrameObservation {
    let mut pixels = [[10, 10, 10, 255]; 16];
    for (index, pixel) in pixels.iter_mut().enumerate() {
        if index % 4 < prefix * 2 {
            *pixel = [240, 240, 240, 255];
        }
    }
    rgba(&pixels)
}

pub fn motion(changed: usize) -> FrameObservation {
    let mut pixels = [[0, 0, 0, 255]; 16];
    for pixel in pixels.iter_mut().take(changed) {
        *pixel = [255, 255, 255, 255];
    }
    rgba(&pixels)
}

fn pixel(frame: &FrameObservation, index: usize) -> [u8; 4] {
    let start = index * 4;
    frame.data[start..start + 4].try_into().unwrap()
}
