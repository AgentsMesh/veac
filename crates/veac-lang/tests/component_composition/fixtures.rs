pub const COMPOSED: &str = r#"component sequence leaf {
  param time duration;
  slot visual visual;
  body { layer visual @leaf-layer { item @leaf {
    source slot visual; record { at 0s; duration ${duration}; }
  } } }
}
component sequence shell {
  param time duration;
  slot visual backdrop;
  instance sequence @card from leaf {
    bind duration duration;
    fill visual { source slot backdrop; }
  }
  body { layer visual @shell-layer { item @nested {
    source sequence sequence @card; record { at 0s; duration ${duration}; }
  } } }
}
instance sequence root from shell {
  bind duration 2s;
  fill backdrop { source generated solid { color #176b87ff; } }
}
project nested {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main { layer visual content { item root-item {
    source sequence sequence root; record { at 0s; duration 2s; }
  } } }
}"#;

pub const MODULE: &str = r#"module {
  const time leaf_default = 1s;
  component sequence leaf {
    param time duration default leaf_default;
    body { layer visual @visual { item @content {
      source generated transparent; record { at 0s; duration ${duration}; }
    } } }
  }
  export component sequence shell {
    param time duration default 3s;
    instance sequence @default-leaf from leaf {}
    instance sequence @bound-leaf from leaf { bind duration duration; }
    body { layer visual @children {
      item @default { source sequence sequence @default-leaf;
        record { at 0s; duration 1s; } }
      item @bound { source sequence sequence @bound-leaf;
        record { at 1s; duration ${duration}; } }
    } }
  }
}"#;

pub const ENTRY: &str = r#"import "./components.veac" as components;
const time caller_duration = 5s;
instance sequence root from components.shell { bind duration caller_duration; }
project lexical {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main { layer visual content { item root-item {
    source sequence sequence root; record { at 0s; duration 5s; }
  } } }
}"#;
