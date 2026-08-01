# Text, Captions, and Audio

Text owns content, style, layout, and optional unit animation:

```veac
source text {
  content "Typed editing";
  style {
    font family "Inter";
    size 64px;
    weight bold;
    fill #ffffffff;
    background { color #000000aa; padding 12px; }
  }
  layout {
    box-width 1000px;
    box-height 240px;
    wrap word;
    overflow ellipsis;
    horizontal-align center;
    vertical-align middle;
  }
  animation {
    unit word;
    stagger 100ms;
    reveal 100%;
    opacity 100%;
  }
}
```

Style also supports fallback fonts, outline, shadow, tracking, line height, and non-overlapping scalar-index spans. Layout supports writing mode, glyph orientation, and typed paths. Animation units are whole, line, word, or grapheme. On a line containing whitespace, `word` treats each non-whitespace token as an author-defined unit; without whitespace it follows Unicode word boundaries. Whitespace and rich style spans never create extra animation units, and numbering continues across lines in logical reading order.

Caption adds an optional speaker while reusing the text value objects:

```veac
source caption {
  content "Welcome";
  speaker "Narrator";
  style { font family "Inter"; size 48px; fill #ffffffff; }
}
```

Caption sources are valid only on caption layers. Sidecars select typed caption layers; cue timing comes from item record spans.

Audio is a closed pipeline:

```veac
audio voice {
  gain -2db;
  pan 0;
  muted false;
  normalize false;
  pitch preserve;
  crossfade { fade-in 80ms; fade-out 120ms; curve equal-power; }
  processor high-pass { frequency 80hz; q 0.707; poles 2; }
  processor compressor {
    threshold -18db; ratio 3; attack 10ms; release 120ms;
    knee 6db; makeup-gain 2db; mix 100%;
  }
}
```

Layers may route to typed buses; sidechain relations connect an item or bus key to a target item.
