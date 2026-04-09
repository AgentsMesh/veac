/// ProbeBackend trait and MediaResolver.
///
/// The `ProbeBackend` trait abstracts media file probing so that:
/// - `veac-runtime` provides the real `FfprobeBackend` implementation
/// - Tests use `MockProbe` with known durations
/// - `veac-lang` stays free of ffprobe dependencies
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use crate::ir::{IrProgram, IrTrackItem, MediaInfo};
use crate::resolve::font::resolve_font;

/// Error type for probe failures.
#[derive(Debug)]
pub struct ProbeError {
    pub message: String,
}

impl ProbeError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl fmt::Display for ProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ProbeError {}

/// Trait for probing media files. Implemented by `veac-runtime` (ffprobe)
/// and by test doubles.
pub trait ProbeBackend {
    fn probe(&self, path: &Path) -> Result<MediaInfo, ProbeError>;
}

/// Warning emitted when media resolution encounters a non-fatal issue.
#[derive(Debug, Clone)]
pub struct ResolveWarning {
    pub message: String,
}

/// Resolves media metadata on IR assets and clips.
///
/// Walks every `IrAsset`, probes it via `ProbeBackend`, and populates:
/// - `IrAsset.media_info`
/// - `IrClip.has_audio` (from probe result)
/// - `IrClip.resolved_duration` (with priority: duration > to-from > probe - from > probe > warn)
pub struct MediaResolver<P: ProbeBackend> {
    backend: P,
}

impl<P: ProbeBackend> MediaResolver<P> {
    pub fn new(backend: P) -> Self {
        Self { backend }
    }

    /// Resolve all media metadata in the IR. Mutates the IR in place.
    /// Returns warnings for assets that could not be probed.
    pub fn resolve(&self, ir: &mut IrProgram) -> Vec<ResolveWarning> {
        let mut warnings = Vec::new();

        // Phase 1: Probe each asset and populate media_info.
        let mut info_map: HashMap<String, MediaInfo> = HashMap::new();
        for asset in &mut ir.assets {
            let path = &asset.path;
            match self.backend.probe(path) {
                Ok(info) => {
                    asset.media_info = Some(info.clone());
                    info_map.insert(asset.name.clone(), info);
                }
                Err(e) => {
                    warnings.push(ResolveWarning {
                        message: format!(
                            "could not probe asset `{}` ({}): {}. Duration will be estimated.",
                            asset.name,
                            path.display(),
                            e
                        ),
                    });
                }
            }
        }

        // Phase 2: Walk every clip and resolve has_audio + resolved_duration.
        for track in &mut ir.timeline.tracks {
            for item in &mut track.items {
                if let IrTrackItem::Clip(clip) = item {
                    let probe_info = info_map.get(&clip.asset_name);

                    // Resolve has_audio from probe.
                    if let Some(info) = probe_info {
                        clip.has_audio = info.has_audio;
                    }
                    // else: keep default (true) — conservative

                    // Resolve duration with priority chain:
                    // 1. Explicit `duration` in DSL
                    // 2. `to - from` if both set
                    // 3. `probe_duration - from` if only `from` set
                    // 4. `probe_duration` if nothing set
                    // 5. Warn and leave None (codegen will estimate)
                    let base_duration = if let Some(d) = clip.duration_sec {
                        Some(d)
                    } else if let (Some(from), Some(to)) = (clip.from_sec, clip.to_sec) {
                        Some(to - from)
                    } else if let Some(info) = probe_info {
                        if let Some(from) = clip.from_sec {
                            Some(info.duration_secs - from)
                        } else {
                            Some(info.duration_secs)
                        }
                    } else {
                        warnings.push(ResolveWarning {
                            message: format!(
                                "clip `{}` has no duration, from/to, or probe data. \
                                 Codegen will estimate duration (may be inaccurate).",
                                clip.asset_name
                            ),
                        });
                        None
                    };

                    // Apply speed factor.
                    clip.resolved_duration = base_duration.map(|d| {
                        if let Some(spd) = clip.speed {
                            d / spd
                        } else {
                            d
                        }
                    });
                }
            }
        }

        // Phase 3: Resolve font paths for text overlays.
        let mut font_cache: HashMap<String, Option<String>> = HashMap::new();
        for track in &mut ir.timeline.tracks {
            for item in &mut track.items {
                if let IrTrackItem::TextOverlay(text) = item {
                    let resolved = font_cache
                        .entry(text.font.clone())
                        .or_insert_with(|| {
                            resolve_font(&text.font).map(|p| p.display().to_string())
                        })
                        .clone();

                    if resolved.is_none() {
                        warnings.push(ResolveWarning {
                            message: format!(
                                "font `{}` not found on this system. \
                                 Text overlay may not render. Install the font or use a different one.",
                                text.font
                            ),
                        });
                    }
                    text.resolved_font_path = resolved;
                }
            }
        }

        warnings
    }
}
