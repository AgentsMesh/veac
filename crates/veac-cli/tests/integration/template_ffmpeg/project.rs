use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::super::support::TempDir;
use super::binding::replacement;
use super::fixture::{probe, FixtureMedia};

pub(super) const SOURCE: &str = r#"
project template-render-e2e {
  settings {
    timebase 1/600; canvas 64px by 36px;
    frame-rate 10fps; sample-rate 48000hz;
  }
  entry sequence main;
  resource video fit {
    locator local { path "placeholder-fit.mp4"; }
    streams { video auto; audio disabled; }
  }
  resource video head {
    locator local { path "placeholder-head.mp4"; }
    streams { video auto; audio disabled; }
  }
  resource video center {
    locator local { path "placeholder-center.mp4"; }
    streams { video auto; audio disabled; }
  }
  resource video crop {
    locator local { path "placeholder-crop.mp4"; }
    streams { video auto; audio disabled; }
  }
  resource image still { locator local { path "placeholder-still.png"; } }
  sequence main {
    layer video picture {
      item fit {
        source media resource fit;
        record { at 0s; duration 1s; }
        mapping linear { from 0s; to 1s; }
        modifiers { layout full { placement anchor { at center; inset { x 0px; y 0px; } } frame { width 64px; height 36px; fit fill; } } }
        template-slot media {
          accepts video;
          fill fit-duration;
          label "Fit complete source";
        }
      }
      item head {
        source media resource head;
        record { at 1s; duration 1s; }
        mapping linear { from 0s; to 1s; }
        modifiers { layout full { placement anchor { at center; inset { x 0px; y 0px; } } frame { width 64px; height 36px; fit fill; } } }
        template-slot media {
          accepts video;
          fill take-head;
          label "Take source head";
        }
      }
      item center {
        source media resource center;
        record { at 2s; duration 1s; }
        mapping linear { from 0s; to 1s; }
        modifiers { layout full { placement anchor { at center; inset { x 0px; y 0px; } } frame { width 64px; height 36px; fit fill; } } }
        template-slot media {
          accepts video;
          fill take-center;
          label "Take source center";
        }
      }
      item crop {
        source media resource crop;
        record { at 3s; duration 1s; }
        mapping linear { from 0s; to 1s; }
        modifiers { layout full { placement anchor { at center; inset { x 0px; y 0px; } } frame { width 64px; height 36px; fit fill; } } }
        template-slot media {
          accepts video;
          fill take-head;
          label "Portrait aspect fill";
        }
      }
      item still {
        source media resource still;
        record { at 4s; duration 1s; }
        modifiers { layout full { placement anchor { at center; inset { x 0px; y 0px; } } frame { width 64px; height 36px; fit fill; } } }
        template-slot media {
          accepts image;
          fill fit-duration;
          label "Static image";
        }
      }
    }
  }
  delivery main {
    sequence main;
    raster { canvas 64px by 36px; frame-rate 10fps; captions discard; }
    artifact video main {
      target file "template-render.mp4";
      mux mp4 { layout fast-start; video h264 { pixel-format yuv420p; alpha opaque; color-space source; rate-control crf { value 23; } gop automatic; b-frames automatic; profile automatic; level automatic; } audio none; passes single; accelerator auto; }
    }
  }
}
"#;

pub(super) fn write_request(temp: &TempDir, project: &Path, media: &FixtureMedia) -> PathBuf {
    let envelope = super::fixture::read_project(project);
    let temporal_probe = probe(&media.temporal);
    let portrait_probe = probe(&media.portrait);
    let still_probe = probe(&media.still);
    let bindings = [
        (
            "fit",
            &media.temporal,
            temporal_probe.clone(),
            veac_ir::MaterialKind::Video,
        ),
        (
            "head",
            &media.temporal,
            temporal_probe.clone(),
            veac_ir::MaterialKind::Video,
        ),
        (
            "center",
            &media.temporal,
            temporal_probe,
            veac_ir::MaterialKind::Video,
        ),
        (
            "crop",
            &media.portrait,
            portrait_probe,
            veac_ir::MaterialKind::Video,
        ),
        (
            "still",
            &media.still,
            still_probe,
            veac_ir::MaterialKind::Image,
        ),
    ]
    .into_iter()
    .map(|(name, path, probe, kind)| veac_template::MediaBinding {
        clip_id: veac_ir::ItemId::new(format!("itm_{name}")).unwrap(),
        material: replacement(name, path, probe, kind),
    })
    .collect();
    let request = veac_template::TemplateFillRequest {
        schema: veac_template::TEMPLATE_FILL_SCHEMA_ID.to_owned(),
        schema_version: veac_template::TEMPLATE_FILL_SCHEMA_VERSION,
        operation_id: veac_ir::OperationId::new("op_template_render_e2e").unwrap(),
        base_revision: envelope.project.revision,
        media_bindings: bindings,
        text_bindings: vec![],
    };
    let output = temp.path().join("template-request.json");
    std::fs::write(&output, serde_json::to_vec_pretty(&request).unwrap()).unwrap();
    output
}

pub(super) fn assert_template_state_cleared(path: &Path) {
    let envelope = super::fixture::read_project(path);
    let sources: BTreeMap<_, _> = envelope
        .project
        .materials
        .iter()
        .map(|material| {
            let veac_ir::MaterialSource::File { uri } = &material.source else {
                panic!("template replacement must remain a local material");
            };
            (material.id.as_str(), uri.as_str())
        })
        .collect();
    for (id, uri) in [
        ("med_fit", "temporal.mp4"),
        ("med_head", "temporal.mp4"),
        ("med_center", "temporal.mp4"),
        ("med_crop", "portrait.mp4"),
        ("med_still", "still.png"),
    ] {
        assert_eq!(sources[id], uri);
    }
    let clips = envelope
        .project
        .sequences
        .iter()
        .flat_map(|sequence| sequence.tracks.iter().flat_map(|track| track.clips.iter()));
    for clip in clips {
        assert!(clip.replaceable.is_none());
        assert!(!clip.template_editable_text);
    }
    assert_eq!(
        envelope
            .project
            .materials
            .iter()
            .find(|material| material.id.as_str() == "med_still")
            .unwrap()
            .kind,
        veac_ir::MaterialKind::Image
    );
}
