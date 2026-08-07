#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use tempfile::{tempdir, TempDir};
use veac_lang::program::{prepare_path, BuiltProgram};
use veac_lang::source_edit::{ImportSource, SourceEditBatch, TopLevelDeclarationSource};

pub struct Fixture {
    _temp: TempDir,
    pub entry: PathBuf,
    expected: BTreeMap<PathBuf, String>,
}

impl Fixture {
    pub fn new(entry: &str, modules: &[(&str, &str)]) -> Self {
        let temp = tempdir().unwrap();
        let entry_path = temp.path().join("main.veac");
        let mut expected = BTreeMap::new();
        write(&mut expected, &entry_path, entry);
        for (path, source) in modules {
            write(&mut expected, &temp.path().join(path), source);
        }
        Self {
            _temp: temp,
            entry: entry_path,
            expected,
        }
    }

    pub fn batch(&self, id: &str) -> SourceEditBatch {
        let index = prepare_path(&self.entry).unwrap().source_index().unwrap();
        SourceEditBatch::new(
            veac_ir::OperationId::new(id).unwrap(),
            index.revision().clone(),
        )
    }

    pub fn assert_unchanged(&self) {
        for (path, source) in &self.expected {
            assert_eq!(
                &fs::read_to_string(path).unwrap(),
                source,
                "{}",
                path.display()
            );
        }
    }
}

pub fn project_with(declarations: &str, duration: &str) -> String {
    let mut item = String::new();
    write!(
        item,
        ".with_item(item(identifier(\"result\"), item_enabled(), during(0s, {duration}), \
         source_generated(generator_transparent()), source_timing_native()))"
    )
    .unwrap();
    format!(
        r#"{declarations}
fn main(context: Context) -> Project {{
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let layer = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()){item};
  let timeline = sequence(identifier("main"), "结构化源码编辑",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
    .with_layer(layer);
  project(identifier("structural-edit"), project_settings(1000))
    .with_sequence(timeline).entry(timeline)
}}"#
    )
}

pub fn declaration(source: &str) -> TopLevelDeclarationSource {
    TopLevelDeclarationSource {
        source: source.into(),
    }
}

pub fn import(path: &str, alias: &str) -> ImportSource {
    ImportSource {
        path: path.into(),
        alias: alias.into(),
    }
}

pub fn duration(program: &BuiltProgram) -> i64 {
    program.envelope().project.sequences[0].tracks[0].clips[0]
        .record_range
        .duration
        .value
}

fn write(expected: &mut BTreeMap<PathBuf, String>, path: &Path, source: &str) {
    fs::write(path, source).unwrap();
    expected.insert(path.to_path_buf(), source.to_owned());
}
