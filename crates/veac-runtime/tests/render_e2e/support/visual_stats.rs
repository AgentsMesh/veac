use super::*;

#[derive(Debug)]
pub(crate) struct FrameStats {
    pub ratio: f64,
    pub centroid_x: f64,
    pub centroid_y: f64,
    pub energy: f64,
    pub left_strip_ratio: f64,
    pub right_strip_ratio: f64,
}

#[derive(Debug)]
pub(crate) struct LitExtents {
    pub left: usize,
    pub right: usize,
    pub top: usize,
    pub bottom: usize,
}

pub(crate) fn frame_stats(frame: &[u8]) -> FrameStats {
    frame_stats_with_threshold(frame, 60)
}

pub(crate) fn frame_stats_with_threshold(frame: &[u8], threshold: usize) -> FrameStats {
    let background = [frame[0], frame[1], frame[2]];
    let mut count = 0_usize;
    let mut sum_x = 0_usize;
    let mut sum_y = 0_usize;
    let mut energy = 0_usize;
    let mut left = 0_usize;
    let mut right = 0_usize;
    for (index, pixel) in frame.chunks_exact(3).enumerate() {
        let difference: usize = pixel
            .iter()
            .zip(background)
            .map(|(&value, base)| value.abs_diff(base) as usize)
            .sum();
        if difference > threshold {
            let x = index % WIDTH as usize;
            count += 1;
            sum_x += x;
            sum_y += index / WIDTH as usize;
            energy += difference;
            left += usize::from(x < 16);
            right += usize::from(x >= 80);
        }
    }
    FrameStats {
        ratio: count as f64 / f64::from(WIDTH * HEIGHT),
        centroid_x: sum_x as f64 / count as f64,
        centroid_y: sum_y as f64 / count as f64,
        energy: energy as f64,
        left_strip_ratio: left as f64 / f64::from(16 * HEIGHT),
        right_strip_ratio: right as f64 / f64::from(16 * HEIGHT),
    }
}

pub(crate) fn changed_channels(left: &[u8], right: &[u8]) -> usize {
    left.iter()
        .zip(right)
        .filter(|(a, b)| u8::abs_diff(**a, **b) > 24)
        .count()
}

pub(crate) fn lit_bounds(frame: &[u8]) -> (usize, usize) {
    let extents = lit_extents(frame);
    (
        extents.right - extents.left + 1,
        extents.bottom - extents.top + 1,
    )
}

pub(crate) fn lit_extents(frame: &[u8]) -> LitExtents {
    let background = [frame[0], frame[1], frame[2]];
    let points: Vec<_> = frame
        .chunks_exact(3)
        .enumerate()
        .filter(|(_, pixel)| {
            pixel
                .iter()
                .zip(background)
                .map(|(&value, base)| value.abs_diff(base) as usize)
                .sum::<usize>()
                > 60
        })
        .map(|(index, _)| (index % WIDTH as usize, index / WIDTH as usize))
        .collect();
    LitExtents {
        left: points.iter().map(|point| point.0).min().unwrap(),
        right: points.iter().map(|point| point.0).max().unwrap(),
        top: points.iter().map(|point| point.1).min().unwrap(),
        bottom: points.iter().map(|point| point.1).max().unwrap(),
    }
}
