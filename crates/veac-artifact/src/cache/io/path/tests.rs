use super::*;
use crate::ArtifactErrorKind;

#[test]
fn relative_cache_paths_are_normalized_without_allowing_parents() {
    let (base, names, normalized) = path_parts(Path::new("cache/item")).unwrap();
    assert_eq!(base, Path::new("."));
    assert_eq!(names, [OsString::from("cache"), OsString::from("item")]);
    assert_eq!(normalized, std::env::current_dir().unwrap());
    assert_eq!(
        path_parts(Path::new("../escape")).unwrap_err().kind,
        ArtifactErrorKind::UnsafePath
    );
    assert_eq!(
        target_parts(Path::new("/cache"), Path::new("/cache"))
            .unwrap_err()
            .kind,
        ArtifactErrorKind::UnsafePath
    );
}
