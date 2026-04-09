/// System font resolution: maps font family names to system font file paths.
///
/// Searches platform-specific font directories for matching .ttf/.otf files.
/// This is needed because FFmpeg's drawtext `fontfile=` parameter requires
/// a file path, not a font family name.
use std::path::{Path, PathBuf};

/// Resolve a font family name to its system font file path.
/// Returns `None` if the font cannot be found on this system.
pub fn resolve_font(family: &str) -> Option<PathBuf> {
    let family_lower = family.to_lowercase();

    for dir in font_dirs() {
        if !dir.exists() {
            continue;
        }
        if let Some(path) = search_dir(&dir, &family_lower) {
            return Some(path);
        }
    }
    None
}

/// Platform-specific font directories.
fn font_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    #[cfg(target_os = "macos")]
    {
        dirs.push(PathBuf::from("/System/Library/Fonts"));
        dirs.push(PathBuf::from("/System/Library/Fonts/Supplemental"));
        dirs.push(PathBuf::from("/Library/Fonts"));
        if let Some(home) = std::env::var_os("HOME") {
            dirs.push(PathBuf::from(home).join("Library/Fonts"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        dirs.push(PathBuf::from("/usr/share/fonts"));
        dirs.push(PathBuf::from("/usr/local/share/fonts"));
        if let Some(home) = std::env::var_os("HOME") {
            dirs.push(PathBuf::from(home).join(".local/share/fonts"));
            dirs.push(PathBuf::from(home).join(".fonts"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(windir) = std::env::var_os("WINDIR") {
            dirs.push(PathBuf::from(windir).join("Fonts"));
        }
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            dirs.push(PathBuf::from(local).join("Microsoft\\Windows\\Fonts"));
        }
    }

    dirs
}

/// Search a directory (non-recursively first, then subdirs) for a matching font.
fn search_dir(dir: &Path, family_lower: &str) -> Option<PathBuf> {
    // Priority: exact match "Arial.ttf" > "Arial Bold.ttf" > partial match
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return None,
    };

    let mut candidates: Vec<PathBuf> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            // Recurse into subdirectories
            if let Some(found) = search_dir(&path, family_lower) {
                candidates.push(found);
            }
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext != "ttf" && ext != "otf" {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Exact match: "arial" == "arial"
        if stem == family_lower {
            return Some(path);
        }
        // Regular variant: "arial regular"
        if stem == format!("{family_lower} regular") || stem == format!("{family_lower}-regular") {
            return Some(path);
        }
        // Prefix match: "arial bold", "arial narrow" etc. (lower priority)
        if stem.starts_with(family_lower) {
            candidates.push(path);
        }
    }

    // Return the shortest-named candidate (closest to the base font)
    candidates.sort_by_key(|p| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .len()
    });
    candidates.into_iter().next()
}
