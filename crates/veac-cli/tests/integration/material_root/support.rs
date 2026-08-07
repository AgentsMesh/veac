use std::path::{Path, PathBuf};

use super::super::support::{make_video, tempdir, veac, Command, TempDir, MEDIA_SOURCE};

pub(super) struct DetachedFixture {
    temp: TempDir,
    pub(super) source_root: PathBuf,
    pub(super) build_root: PathBuf,
    pub(super) source: PathBuf,
    pub(super) media: PathBuf,
    pub(super) project: PathBuf,
}

impl DetachedFixture {
    pub(super) fn new() -> Self {
        let temp = tempdir().unwrap();
        let source_root = temp.path().join("source");
        let build_root = temp.path().join("build");
        std::fs::create_dir(&source_root).unwrap();
        std::fs::create_dir(&build_root).unwrap();
        let media = source_root.join("clip.mp4");
        make_video(&media);
        let identity = veac_runtime::asset::sha256_identity(&media).unwrap();
        let source_text = MEDIA_SOURCE.replace(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            &identity.digest,
        );
        let source = source_root.join("main.veac");
        std::fs::write(&source, source_text).unwrap();
        let project = build_root.join("project.veac.json");
        Self {
            temp,
            source_root,
            build_root,
            source,
            media,
            project,
        }
    }

    pub(super) fn build(&self) -> std::process::Output {
        veac()
            .arg("build")
            .arg(&self.source)
            .arg("--emit-ir")
            .arg(&self.project)
            .arg("--material-root")
            .arg(&self.source_root)
            .output()
            .unwrap()
    }

    pub(super) fn build_success(&self) {
        let output = self.build();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    pub(super) fn command(&self, name: &str) -> Command {
        self.command_at_root(name, &self.source_root)
    }

    pub(super) fn command_at_root(&self, name: &str, root: &Path) -> Command {
        let mut command = veac();
        command
            .arg(name)
            .arg(&self.project)
            .arg("--material-root")
            .arg(root);
        command
    }

    pub(super) fn material_id(&self) -> String {
        let json = std::fs::read_to_string(&self.project).unwrap();
        veac_ir::decode_canonical_json(&json)
            .unwrap()
            .project
            .materials[0]
            .id
            .to_string()
    }

    pub(super) fn package_root(&self) -> PathBuf {
        self.temp.path().join("package")
    }

    pub(super) fn workspace(&self) -> &Path {
        self.temp.path()
    }
}
