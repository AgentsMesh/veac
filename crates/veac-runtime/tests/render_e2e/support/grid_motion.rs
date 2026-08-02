use std::collections::BTreeSet;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GridPhase {
    pub x: usize,
    pub y: usize,
}

pub(crate) fn grid_phase(frame: &[u8], width: usize, height: usize, period: usize) -> GridPhase {
    assert_eq!(frame.len(), width * height * 3);
    let mut horizontal = vec![0_usize; period];
    let mut vertical = vec![0_usize; period];
    for (pixel, rgb) in frame.chunks_exact(3).enumerate() {
        if rgb.iter().all(|channel| *channel > 205) {
            horizontal[(pixel % width) % period] += 1;
            vertical[(pixel / width) % period] += 1;
        }
    }
    GridPhase {
        x: landmark(&horizontal),
        y: landmark(&vertical),
    }
}

pub(crate) fn grid_motion(phases: &[GridPhase], period: usize) -> f64 {
    let total: usize = phases
        .windows(2)
        .map(|pair| {
            wrapped_delta(pair[1].x, pair[0].x, period).unsigned_abs() as usize
                + wrapped_delta(pair[1].y, pair[0].y, period).unsigned_abs() as usize
        })
        .sum();
    total as f64 / (phases.len() - 1) as f64
}

pub(crate) fn aligned_content_count(
    frames: &[Vec<u8>],
    phases: &[GridPhase],
    width: usize,
    height: usize,
    period: usize,
) -> usize {
    let reference = phases[0];
    frames
        .iter()
        .zip(phases)
        .map(|(frame, phase)| aligned_signature(frame, *phase, reference, width, height, period))
        .collect::<BTreeSet<_>>()
        .len()
}

fn landmark(bins: &[usize]) -> usize {
    let total: usize = bins.iter().sum();
    assert!(total > 0, "grid landmark is missing");
    let (phase, peak) = bins
        .iter()
        .enumerate()
        .max_by_key(|(_, value)| *value)
        .unwrap();
    let average = total as f64 / bins.len() as f64;
    assert!(*peak as f64 >= average * 2.2, "grid peak {peak}/{average}");
    phase
}

fn wrapped_delta(value: usize, reference: usize, period: usize) -> isize {
    let mut delta = value as isize - reference as isize;
    let half = period as isize / 2;
    if delta > half {
        delta -= period as isize;
    } else if delta < -half {
        delta += period as isize;
    }
    delta
}

fn aligned_signature(
    frame: &[u8],
    phase: GridPhase,
    reference: GridPhase,
    width: usize,
    height: usize,
    period: usize,
) -> u64 {
    let dx = wrapped_delta(phase.x, reference.x, period);
    let dy = wrapped_delta(phase.y, reference.y, period);
    let mut hash = DefaultHasher::new();
    for y in (period..height - period).step_by(2) {
        for x in (period..width - period).step_by(2) {
            let source_x = (x as isize + dx) as usize;
            let source_y = (y as isize + dy) as usize;
            let offset = (source_y * width + source_x) * 3;
            [
                frame[offset] / 32,
                frame[offset + 1] / 32,
                frame[offset + 2] / 32,
            ]
            .hash(&mut hash);
        }
    }
    hash.finish()
}
