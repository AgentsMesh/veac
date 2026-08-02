# veac-caption

`veac-caption` owns VEAC's versioned, backend-neutral caption interchange model and its
SRT, WebVTT, ASS, and canonical-IR adapters. It does not own a second render timeline.

The format parser is `subtitler` 2.6.1 with default features disabled. Its published
`ass` feature currently references the `Ssa` enum tag without guarding that reference,
so Cargo must also enable the zero-dependency `ssa` feature for the dependency to
compile. VEAC intentionally exposes no SSA format or API.
