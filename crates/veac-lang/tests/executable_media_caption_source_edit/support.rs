use std::fs;
use std::path::{Path, PathBuf};

use tempfile::{tempdir, TempDir};
use veac_lang::program::{build_path, prepare_path};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

const ENTRY: &str = r#"import "./library.veac" as library;
fn main(context: Context) -> Project {
  let voice = library.voice();
  let primary = library.primary_font();
  let alternate = library.alternate_font();
  let caption = library.caption(primary, alternate, voice);
  let state = track_state(
    track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked()
  );
  let sound = item(identifier("voice"), item_enabled(), during(0s, 2s),
    source_media(voice), source_timing_native());
  let audio = audio_layer(identifier("audio"), 0, placement_free(), state,
    track_routing_default()).with_item(sound);
  let captions = caption_layer(identifier("captions"), 1, placement_free(), state,
    track_routing_default()).with_item(caption);
  let timeline = sequence(identifier("main"), "源码编辑",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
    .with_layer(audio).with_layer(captions);
  project(identifier("source-edit"), project_settings(600))
    .with_resource(voice).with_resource(primary).with_resource(alternate)
    .with_sequence(timeline).entry(timeline)
}
"#;

const MODULE: &str = r#"module {
  export fn voice() -> Resource {
    audio_resource(identifier("voice"), resource_file("assets/old.wav"),
      sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
      stream_auto())
  }
  export fn primary_font() -> Resource {
    font_resource(identifier("primary"), resource_file("assets/primary.ttf"),
      sha256("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"))
  }
  export fn alternate_font() -> Resource {
    font_resource(identifier("alternate"), resource_file("assets/alternate.ttf"),
      sha256("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"))
  }
  fn style(font: Resource) -> TextStyle {
    let fonts = font_stack(font_resource_ref(font), []);
    let metrics = text_metrics(
      fonts, weight_normal(), font_style_normal(), 32px, 0px, 1.0, #ffffffff
    );
    let layout = text_layout(
      text_box_auto(), text_wrap_none(), text_overflow_visible(),
      text_align_center(), text_align_middle(),
      writing_horizontal_tb(), orientation_mixed()
    );
    text_style(metrics, layout, text_path_none(),
      text_decoration(text_background_none(), text_outline_none(), shadow_none()),
      [], text_animation_none())
  }
  export fn caption(primary: Resource, alternate: Resource, wrong: Resource) -> Item {
    item(identifier("caption"), item_enabled(), during(0s, 2s),
      source_caption_speaker("旧字幕", "旧旁白", style(primary)),
      source_timing_native())
  }
}
"#;

pub(super) struct Fixture {
    _directory: TempDir,
    pub(super) entry: PathBuf,
    pub(super) module: PathBuf,
}

impl Fixture {
    pub(super) fn new() -> Self {
        let directory = tempdir().unwrap();
        let entry = directory.path().join("main.veac");
        let module = directory.path().join("library.veac");
        fs::write(&entry, ENTRY).unwrap();
        fs::write(&module, MODULE).unwrap();
        Self {
            _directory: directory,
            entry,
            module,
        }
    }

    pub(super) fn baseline(&self) -> Vec<u8> {
        veac_ir::canonical_json(build_path(&self.entry).unwrap().envelope())
            .unwrap()
            .into_bytes()
    }
}

pub(super) fn batch(entry: &Path, function: &str, body: &str) -> SourceEditBatch {
    let prepared = prepare_path(entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_media_caption_source_edit").unwrap(),
        prepared.source_revision().unwrap(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("library.veac", function),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: body.to_owned(),
        },
    });
    batch
}

pub(super) fn assert_unchanged(fixture: &Fixture, entry: &[u8], module: &[u8], ir: &[u8]) {
    assert_eq!(fs::read(&fixture.entry).unwrap(), entry);
    assert_eq!(fs::read(&fixture.module).unwrap(), module);
    assert_eq!(fixture.baseline(), ir);
}
