/// Probe cache: avoids re-probing media files on subsequent builds.
///
/// Cache key: (file_path, mtime, file_size) → MediaInfo
/// Stored as `.veac-cache/probe.json` alongside the .veac file.
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use veac_lang::ir::MediaInfo;
use veac_lang::resolve::{ProbeBackend, ProbeError};

/// Cache entry with metadata for staleness detection.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    mtime_secs: u64,
    file_size: u64,
    info: CachedMediaInfo,
}

/// Serializable version of MediaInfo.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CachedMediaInfo {
    duration_secs: f64,
    has_video: bool,
    has_audio: bool,
    width: Option<u32>,
    height: Option<u32>,
    sample_rate: Option<u32>,
}

impl From<&MediaInfo> for CachedMediaInfo {
    fn from(info: &MediaInfo) -> Self {
        Self {
            duration_secs: info.duration_secs,
            has_video: info.has_video,
            has_audio: info.has_audio,
            width: info.width,
            height: info.height,
            sample_rate: info.sample_rate,
        }
    }
}

impl From<CachedMediaInfo> for MediaInfo {
    fn from(c: CachedMediaInfo) -> Self {
        Self {
            duration_secs: c.duration_secs,
            has_video: c.has_video,
            has_audio: c.has_audio,
            width: c.width,
            height: c.height,
            sample_rate: c.sample_rate,
        }
    }
}

type CacheMap = HashMap<String, CacheEntry>;

/// A caching wrapper around any `ProbeBackend`.
/// On cache hit (same mtime + size), returns cached MediaInfo without probing.
/// On cache miss, delegates to the inner backend and writes to cache.
pub struct CachedProbeBackend<P: ProbeBackend> {
    inner: P,
    cache: RefCell<CacheMap>,
    cache_path: PathBuf,
    dirty: RefCell<bool>,
}

impl<P: ProbeBackend> CachedProbeBackend<P> {
    /// Create a cached backend. `cache_dir` is typically `.veac-cache/` next to the .veac file.
    pub fn new(inner: P, cache_dir: &Path) -> Self {
        let cache_path = cache_dir.join("probe.json");
        let cache = load_cache(&cache_path);
        Self {
            inner,
            cache: RefCell::new(cache),
            cache_path,
            dirty: RefCell::new(false),
        }
    }

    /// Flush dirty cache to disk.
    pub fn flush(&self) {
        if *self.dirty.borrow() {
            if let Some(parent) = self.cache_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(&*self.cache.borrow()) {
                let _ = fs::write(&self.cache_path, json);
            }
        }
    }

    /// Remove the cache file.
    pub fn clean(cache_dir: &Path) {
        let cache_path = cache_dir.join("probe.json");
        let _ = fs::remove_file(cache_path);
    }
}

impl<P: ProbeBackend> ProbeBackend for CachedProbeBackend<P> {
    fn probe(&self, path: &Path) -> Result<MediaInfo, ProbeError> {
        let key = path.display().to_string();

        // Check cache hit
        if let Some(entry) = self.cache.borrow().get(&key).cloned() {
            if let Ok((mtime, size)) = file_metadata(path) {
                if entry.mtime_secs == mtime && entry.file_size == size {
                    return Ok(entry.info.into());
                }
            }
        }

        // Cache miss: probe
        let info = self.inner.probe(path)?;

        // Write to cache
        if let Ok((mtime, size)) = file_metadata(path) {
            self.cache.borrow_mut().insert(
                key,
                CacheEntry {
                    mtime_secs: mtime,
                    file_size: size,
                    info: CachedMediaInfo::from(&info),
                },
            );
            *self.dirty.borrow_mut() = true;
        }

        Ok(info)
    }
}

impl<P: ProbeBackend> Drop for CachedProbeBackend<P> {
    fn drop(&mut self) {
        self.flush();
    }
}

fn file_metadata(path: &Path) -> Result<(u64, u64), std::io::Error> {
    let meta = fs::metadata(path)?;
    let mtime = meta
        .modified()?
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let size = meta.len();
    Ok((mtime, size))
}

fn load_cache(path: &Path) -> CacheMap {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
