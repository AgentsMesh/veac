# Editing Capability Matrix And Roadmap

Snapshot: 2026-07-24

VEAC targets editing-mechanism parity with mainstream products such as CapCut/Jianying. Bundled
media, music, fonts, stickers, effect/template packages, catalogs, licensing, accounts, publishing,
and proprietary assets are out of scope. Typed template slots and deterministic filling are editing
mechanisms and remain in scope.

## Scope And Architecture

A mechanism is in scope when it changes how a project is represented, edited, analyzed, composed,
cached, or exported. Marketing quality claims are not acceptance criteria. Delivery requires a typed
model, validation, an authoring or edit surface, resolution, backend behavior where applicable,
invalid-case coverage, documentation, and the evidence linked below.

The comparison baseline uses public mechanism documentation: [Jianying](https://www.jianying.com/)
for multitrack, masks, keyframes, color, multicam, audio cleanup, retouch, cutout, and automation;
[CapCut Desktop](https://www.capcut.com/tools/desktop-video-editor) and its official
[product index](https://www.capcut.com/llms.txt) for captions, translation, speed, tracking, reframe,
keying, cleanup, and export; and Adobe's established [proxy](https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/ingest-and-proxy-workflow.html)
and [sequence preview](https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/render-a-section-of-a-sequence.html)
workflows. VEAC accepts mechanisms, not vendor quality or proprietary asset claims.

```text
typed authoring source or typed EditBatch
    -> Canonical JSON IR
    -> ResolvedRenderPlan
    -> sealed BackendBundle
    -> guarded BundleExecutor
    -> FFmpeg or artifact writes
```

Machine-local paths exist only in execution bindings. Providers produce content-addressed artifacts;
they do not run in plan resolution or code generation. The flat language, legacy frontend IR, old
render path, `build`, CSV `batch`, variables, and `include` are deleted without compatibility readers.

## Acceptance Slices

- **P1 executable**: a deterministic editing or delivery mechanism that runs locally through the
  canonical chain. Backend-visible behavior requires decoded pixel, audio, timing, or metadata E2E.
- **P2 provider application**: a versioned external-inference contract plus verified artifacts,
  evidence-bound deterministic mapping, and atomic canonical application. VEAC executes the local
  provider protocol and application; model engines, weights, and inference quality remain external.

The declared completion claim is precise: every P1 row below is locally executable, and every P2
row has its provider-application slice. It does not claim bundled models, remote acquisition, or an
explicit hardware upload/device implementation.

## Status Legend

- **Delivered**: complete on the public canonical path with behavior and integration evidence.
- **Delivered/Contract**: local mechanisms and canonical application are complete; inference is an
  external provider responsibility.
- **Partial**: useful executable support exists, but the declared baseline is broader.
- **Contract**: a deterministic typed boundary exists, but no executable implementation is claimed.
- **Missing**: no complete public mechanism exists.

## Capability Matrix

| ID | Slice | Capability | Status | Evidence |
|---|---|---|---|---|
| `P1-01` | P1 | Strict project schema, typed IDs, exact timebase, JCS/schema and semantic/snapshot hashes | Delivered | [P1-01](capabilities/p1-evidence-a.md#p1-01) |
| `P1-02` | P1 | Typed authoring source, spans, exact literals, closed primitives, and semantic formatting | Delivered | [P1-02](capabilities/p1-evidence-a.md#p1-02) |
| `P1-03` | P1 | Separate record/source time, magnetic/free placement, overlaps, gaps, freeze, reuse and ranges | Delivered | [P1-03](capabilities/p1-evidence-a.md#p1-03) |
| `P1-04` | P1 | Atomic revisioned edits, preconditions, idempotency, structural/timeline/property edits and snapping | Delivered | [P1-04](capabilities/p1-evidence-a.md#p1-04) |
| `P1-05` | P1 | Typed template slots, identity-bound probes, timing/fit modes and atomic complete fill proposals | Delivered | [P1-05](capabilities/p1-evidence-a.md#p1-05) |
| `P1-06` | P1 | Multiple video/audio/visual/caption tracks, state, order and audio buses | Delivered | [P1-06](capabilities/p1-evidence-a.md#p1-06) |
| `P1-07` | P1 | Multiple sequences, named render configs, nested precompositions and cycle defense | Delivered | [P1-07](capabilities/p1-evidence-a.md#p1-07) |
| `P1-08` | P1 | Sequence-owned AV/group membership, lock propagation and synchronized linked split | Delivered | [P1-08](capabilities/p1-evidence-a.md#p1-08) |
| `P1-09` | P1 | Shared frame/placement/transform/flip/opacity/composition/card/mask model for visual sources | Delivered | [P1-09](capabilities/p1-evidence-a.md#p1-09) |
| `P1-10` | P1 | Nine anchors, pixel/percent/normalized absolute coordinates and contain/cover/fill boxes | Delivered | [P1-10](capabilities/p1-evidence-a.md#p1-10) |
| `P1-11` | P1 | Constant/keyframed geometry, crop viewport, opacity, audio and effect values | Delivered | [P1-11](capabilities/p1-evidence-a.md#p1-11) |
| `P1-12` | P1 | Hold/linear/named/cubic easing with exact split/trim and FFmpeg evaluation | Delivered | [P1-12](capabilities/p1-evidence-a.md#p1-12) |
| `P1-13` | P1 | Explicit z-order and 12 straight-alpha-preserving blend modes | Delivered | [P1-13](capabilities/p1-evidence-a.md#p1-13) |
| `P1-14` | P1 | Seven mask primitives, closed paths, animated controls and alpha/luma track mattes | Delivered | [P1-14](capabilities/p1-evidence-a.md#p1-14) |
| `P1-15` | P1 | Rounded card clipping and independent card/text shadows | Delivered | [P1-15](capabilities/p1-evidence-a.md#p1-15) |
| `P1-16` | P1 | Font binding, size/color/background/outline/shadow and common text composition | Delivered | [P1-16](capabilities/p1-evidence-a.md#p1-16) |
| `P1-17` | P1 | Unicode/BiDi shaping, rich spans, boxes, fallback, vertical text and text-on-path | Delivered | [P1-17](capabilities/p1-evidence-a.md#p1-17) |
| `P1-18` | P1 | Bounded whole/line/word/grapheme reveal, opacity and transform text animation | Delivered | [P1-18](capabilities/p1-evidence-a.md#p1-18) |
| `P1-19` | P1 | Caption timelines plus loss-aware SRT/WebVTT/ASS interchange and resolved delivery | Delivered | [P1-19](capabilities/p1-evidence-a.md#p1-19) |
| `P1-20` | P1 | Arbitrary audio placement/trim/overlap, sample delay, mix, buses and mute/solo | Delivered | [P1-20](capabilities/p1-evidence-a.md#p1-20) |
| `P1-21` | P1 | Gain/pan curves, fades, pitch policy and partial-range loudness normalization | Delivered | [P1-21](capabilities/p1-evidence-b.md#p1-21) |
| `P1-22` | P1 | Ordered brightness, contrast and saturation with scalar/curve execution | Delivered | [P1-22](capabilities/p1-evidence-b.md#p1-22) |
| `P1-23` | P1 | Chroma/luma key, inversion, animated controls, softness and spill suppression | Delivered | [P1-23](capabilities/p1-evidence-b.md#p1-23) |
| `P1-24` | P1 | Exact speed, bounded reverse, containing-frame freeze, repeat, pitch and synthesis policy | Delivered | [P1-24](capabilities/p1-evidence-b.md#p1-24) |
| `P1-25` | P1 | Exact source-time curves, ramps, holds, audio mapping and malformed-plan defense | Delivered | [P1-25](capabilities/p1-evidence-b.md#p1-25) |
| `P1-26` | P1 | Seven typed transition families, alignment, handle checks and audio continuity fades | Delivered | [P1-26](capabilities/p1-evidence-b.md#p1-26) |
| `P1-27` | P1 | First-class CompositeBand, Layer and exact ItemSet Apply targets with ordered stages/mix | Delivered | [P1-27](capabilities/p1-evidence-b.md#p1-27) |
| `P1-28` | P1 | Ordered/bypass/ranged effects with registry-gated typed parameters and curves | Delivered | [P1-28](capabilities/p1-evidence-b.md#p1-28) |
| `P1-29` | P1 | Solid/transparent/silence, multi-stop gradients and filled/stroked vector shapes | Delivered | [P1-29](capabilities/p1-evidence-b.md#p1-29) |
| `P1-30` | P1 | Typed containers/codecs, canvas/rate control/GOP/profile/pixel/audio compatibility | Delivered | [P1-30](capabilities/p1-evidence-b.md#p1-30) |
| `P1-31` | P1 | Canonical and untrusted-plan duration/frame/pixel/audio/structure budgets | Delivered | [P1-31](capabilities/p1-evidence-b.md#p1-31) |
| `P1-32` | P1 | Portable software alpha/HDR/MXF, image/caption/audio/scopes, two-pass and resume delivery | Delivered | [P1-32](capabilities/p1-evidence-b.md#p1-32) |
| `P1-33` | P1 | Loss-aware OTIO projection, integrity extension, explicit bindings and atomic proposals | Delivered | [P1-33](capabilities/p1-evidence-b.md#p1-33) |
| `P1-34` | P1 | Identity-verified schema-v3 probe, safe input policy and exact stream selection | Delivered | [P1-34](capabilities/p1-evidence-b.md#p1-34) |
| `P1-35` | P1 | Tool/input manifests, SHA-256 package/restore and bounded identity relink | Delivered | [P1-35](capabilities/p1-evidence-b.md#p1-35) |
| `P1-36` | P1 | Content-addressed artifact cache with bounded no-follow verified storage | Delivered | [P1-36](capabilities/p1-evidence-b.md#p1-36) |
| `P1-37` | P1 | Proxy/analysis/optical-flow/source-segment derivation and exact render-segment reuse | Delivered | [P1-37](capabilities/p1-evidence-b.md#p1-37) |
| `P1-38` | P1 | Canonical source/edit/template/render/caption/OTIO/provider/artifact CLI workflows | Delivered | [P1-38](capabilities/p1-evidence-b.md#p1-38) |
| `P1-39` | P1 | Scoped snapshots, deadlines, atomic commit/recovery, locks and alias defenses | Delivered | [P1-39](capabilities/p1-evidence-b.md#p1-39) |
| `P2-01` | P2 | ASR/language/translation/TTS/dubbing contracts and evidence-bound project mappings | Delivered/Contract | [P2-01](capabilities/p2-evidence.md#p2-01) |
| `P2-02` | P2 | Local audio EQ/dynamics/ducking plus denoise/separation media application | Delivered/Contract | [P2-02](capabilities/p2-evidence.md#p2-02) |
| `P2-03` | P2 | Advanced color/LUT/scopes plus evidence-bound color-match application | Delivered/Contract | [P2-03](capabilities/p2-evidence.md#p2-03) |
| `P2-04` | P2 | Executable deshake plus tracking/stabilization sample-to-transform/crop mappings | Delivered/Contract | [P2-04](capabilities/p2-evidence.md#p2-04) |
| `P2-05` | P2 | Temporal segmentation/correction-matte contracts and evidence-bound matte application | Delivered/Contract | [P2-05](capabilities/p2-evidence.md#p2-05) |
| `P2-06` | P2 | Language/scene/beat/silence/filler/highlight/selection analysis annotations | Delivered/Contract | [P2-06](capabilities/p2-evidence.md#p2-06) |
| `P2-07` | P2 | Subject/crop paths mapped exactly to animated clip-local crop viewports | Delivered/Contract | [P2-07](capabilities/p2-evidence.md#p2-07) |
| `P2-08` | P2 | Retouch/removal results mapped to mattes, effect curves or replacement media | Delivered/Contract | [P2-08](capabilities/p2-evidence.md#p2-08) |
| `P2-09` | P2 | Manual multicam plus evidence-bound automatic sync and authored multiview monitoring | Delivered/Contract | [P2-09](capabilities/p2-evidence.md#p2-09) |
| `P2-10` | P2 | Deterministic bounded provider negotiation/execution/staging/artifact protocol | Delivered | [P2-10](capabilities/p2-evidence.md#p2-10) |

## Explicit Contract Boundaries

- **Remote material acquisition is Contract, not Delivered.** A remote locator may enter canonical
  IR, but plan hydration requires an externally supplied identity-verified local binding. VEAC does
  not download remote media. See [asset binding](language-reference/assets.md#media).
- **`hardware require` is Contract, not Delivered.** Portable `auto`/`software` execution is P1-32;
  an explicit hardware requirement fails closed until typed device and upload-path bindings exist.
  See [execution policy](language-reference/project.md#defaults-and-identity).

## Evidence And Quality Gates

The evidence index is repository-relative and statically checked by
`scripts/check-capability-evidence.sh`. [Execution notes](capabilities/execution-notes.md) explain the
security and render boundaries behind the matrix. New mechanisms must extend the same canonical
chain and add model/schema rejection, edit or parser behavior, planning/codegen preflight, and
observable FFmpeg/ffprobe integration evidence appropriate to their risk.

Public source files are registered by the [example conformance suite](../crates/veac-lang/tests/examples_authoring.rs).
Complete Markdown snippets use `veac,compile` and are parsed, validated, and checked for canonical
format by authoring parse/format idempotence tests; fragments stay unmarked.

Every Rust source/test file stays below 200 lines; `include!` is rejected. Production Lines and
Functions must each be at least 95.02 percent for every crate and the workspace. Coverage is a floor:
a backend-visible row cannot be Delivered without decoded pixel, audio, timing, or metadata evidence.
