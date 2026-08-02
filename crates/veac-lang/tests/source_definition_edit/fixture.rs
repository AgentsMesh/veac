use veac_lang::program::{
    compile_source, SourceIndex, SourceIndexExpression, SourceIndexInventory,
};
use veac_lang::source_edit::{ExpressionSite, SourceNodeRef};

pub const SOURCE: &str = r#"preset text-style shared {
  font family "Arial"; size 24px; weight bold; font-style italic;
  fill #ffffffff; tracking 1px; line-height 1.2;
  background { color #00000099; padding 8px; }
  outline { color #000000ff; width 1px; }
  shadow {
    color #000000ff; opacity 50%; blur 4px;
    offset { x 1px; y 2px; }
  }
}
preset text-layout shared {
  box-width 320px; box-height 80px; wrap none; overflow clip;
  horizontal-align center; vertical-align middle;
  writing-mode horizontal-tb; orientation upright;
  path {
    point { x 0%; y 50%; } point { x 50%; y 25%; }
    point { x 100%; y 50%; }
    start-offset 0px; reverse false; align start;
  }
}
preset modifier-stack shared-effects {
  effect stacked-sharpen { type video.sharpen; parameter amount 1.25; }
}
preset effect-pipeline shared-effects {
  stage effect pipeline-blur { type video.blur; parameter radius 2; }
}
preset color-pipeline finishing-color {
  input-space { primaries bt709; transfer bt709; matrix bt709; range limited; }
  working-space { primaries bt709; transfer linear; matrix rgb; range full; }
  output-space { primaries bt709; transfer bt709; matrix bt709; range limited; }
  basic {
    exposure 0stops; temperature 6500k; tint 0;
    highlights 0; shadows 0; fade 0%;
  }
}
preset audio-processors voice-chain {
  processor eq tone-shaper {
    band presence { frequency 1000hz; gain 1db; q 1; }
  }
  processor limiter final-limiter { ceiling -1db; attack 1ms; release 50ms; }
}
preset delivery-profile wave-delivery {
  artifact audio-stem voice-stem {
    target file "before.wav"; source master;
    encode wav {
      sample-format pcm-s24le; sample-rate 48khz; channel-layout stereo;
    }
  }
}
component sequence child {
  param time duration;
  body {}
}
component sequence card {
  param text title;
  param time duration default 1s;
  instance sequence @child from child { bind duration 500ms; }
  body {
    layer visual @content {
      item @title {
        source text {
          content "before";
          style { use text-style shared; }
          layout { use text-layout shared; }
        }
        record { at 0s; duration 1s; }
        state { playback enabled; }
        modifiers {
          use modifier-stack shared-effects;
          effect @sharpen { type video.sharpen; parameter amount 1; }
        }
      }
    }
    layer audio @voice {
      item @narration {
        source generated silence; record { at 0s; duration 1s; }
        modifiers { audio @voice-chain { use audio-processors voice-chain; } }
      }
    }
    apply @finish {
      scope layer @content; record { at 0s; duration 1s; }
      pipeline {
        use effect-pipeline shared-effects;
        stage effect @polish { type video.sharpen; parameter amount 1.1; }
        stage color @grade { use color-pipeline finishing-color; }
      }
      mix {}
    }
  }
}
instance sequence sample from card { bind title "sample"; bind duration 1s; }
project definition-edit {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {}
  delivery package {
    sequence main;
    use delivery-profile wave-delivery;
  }
}"#;

pub fn source_index() -> SourceIndex {
    compile_source(SOURCE).unwrap().source_index().unwrap()
}

pub fn expression<'a>(
    inventory: &'a SourceIndexInventory,
    target: &SourceNodeRef,
    site: &ExpressionSite,
) -> &'a SourceIndexExpression {
    inventory
        .nodes
        .iter()
        .find(|value| &value.target == target)
        .unwrap()
        .expressions
        .iter()
        .find(|value| &value.site == site)
        .unwrap()
}
