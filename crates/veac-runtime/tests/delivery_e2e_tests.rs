//! Real canonical IR -> plan -> typed bundle -> checkpointed FFmpeg delivery tests.

#![allow(unused_imports)]

#[path = "delivery_e2e/audio_stems.rs"]
mod audio_stems;
#[path = "delivery_e2e/captions.rs"]
mod captions;
#[path = "delivery_e2e/checkpoint.rs"]
mod checkpoint;
#[path = "delivery_e2e/color_alpha.rs"]
mod color_alpha;
#[path = "delivery_e2e/color_alpha_source_over.rs"]
mod color_alpha_source_over;
#[path = "delivery_e2e/delivery_probe.rs"]
mod delivery_probe;
#[path = "delivery_e2e/frames_scopes.rs"]
mod frames_scopes;
#[path = "delivery_e2e/hls.rs"]
mod hls;
#[path = "delivery_e2e/hls_failure.rs"]
mod hls_failure;
#[path = "delivery_e2e/hls_probe.rs"]
mod hls_probe;
#[path = "delivery_e2e/input_scope.rs"]
mod input_scope;
#[path = "delivery_e2e/lossless_video.rs"]
mod lossless_video;
#[path = "delivery_e2e/matte_precision.rs"]
mod matte_precision;
#[path = "delivery_e2e/mp3_gif.rs"]
mod mp3_gif;
#[path = "delivery_e2e/output_executable.rs"]
mod output_executable;
#[path = "delivery_e2e/professional_aux.rs"]
mod professional_aux;
#[path = "delivery_e2e/professional_video.rs"]
mod professional_video;
#[path = "delivery_e2e/still_formats.rs"]
mod still_formats;
#[path = "render_e2e/support/mod.rs"]
mod support;
#[path = "delivery_e2e/video.rs"]
mod video;
