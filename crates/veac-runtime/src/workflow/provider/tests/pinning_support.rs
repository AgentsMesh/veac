use std::path::{Path, PathBuf};

use super::support::Fixture;

pub(super) fn valid_provider(path: &Path, fixture: &Fixture, variant: &str) -> PathBuf {
    let execution = format!(
        "printf '%s' speech > \"$VEAC_PROVIDER_STAGING\"/{}\nprintf '%s' {}",
        shell(&fixture.payload_name),
        shell(
            &String::from_utf8(
                veac_provider::canonical_response_bytes(&fixture.response).unwrap(),
            )
            .unwrap(),
        )
    );
    write(path, &source(fixture, variant, &execution))
}

pub(super) fn rejecting_provider(path: &Path, fixture: &Fixture) -> PathBuf {
    write(path, &source(fixture, "rejecting", "exit 73"))
}

fn source(fixture: &Fixture, variant: &str, execution: &str) -> String {
    let manifest = String::from_utf8(
        veac_provider::canonical_provider_manifest_bytes(&fixture.manifest).unwrap(),
    )
    .unwrap();
    let request =
        String::from_utf8(veac_provider::canonical_request_bytes(&fixture.request).unwrap())
            .unwrap();
    format!(
        r#"#!/bin/sh
# executable variant: {variant}
if [ "$VEAC_PROVIDER_MODE" = "manifest" ]; then
  printf '%s' {manifest}
  case "$1" in
    replace)
      cp "$3" "$2" || exit 81
      ;;
    retarget)
      rm -f "$2" || exit 82
      ln -s "$3" "$2" || exit 83
      ;;
    self-delete)
      rm -f "$0" || exit 84
      ;;
  esac
  exit 0
fi
request=$(cat)
[ "$request" = {request} ] || exit 85
{execution}
"#,
        manifest = shell(&manifest),
        request = shell(&request),
    )
}

fn write(path: &Path, source: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, source).unwrap();
    let mut permissions = std::fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(path, permissions).unwrap();
    path.to_owned()
}

fn shell(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
