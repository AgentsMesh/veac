# RFC: Executable VEAC Language

Status: Implemented; G0-G4 production executable path complete

## Decision
VEAC will be one executable, strongly typed video programming language, with one
surface language, `fn` system, type system, object/value model, and module system.
Stage-specific behavior is inferred; authors do not select separate sublanguages.

This supersedes the static programming-layer decision in
[Agent Authoring And Canonical IR](agent-authoring-and-canonical-ir.md); its later execution boundaries
remain.

Execution evaluates known inputs to build static project topology. Approved dynamic leaves are
residualized from verified Core into canonical temporal programs and typed bindings.

The source AST is never an execution ABI. A reference evaluator may execute typed Core IR;
verified bytecode is a later choice.

The normative G1 contracts are [Core Semantics](executable-core-semantics.md),
[Core IR](executable-core-ir.md), [Structural Values](executable-structural-values.md),
[Nominal Values](executable-nominal-values.md), and
[Domain Graph Values](executable-domain-graph-values.md); this document remains
the architecture charter. The accepted G2-G4 completion contract is
[Executable VEAC Completion](executable-language-completion.md).
## Goals
- Express reusable algorithms with functions, closures, methods, control flow, collections, modules, structs, and closed enums.
- Make time domains and media units native types rather than numeric conventions.
- Generate project topology by executing typed code, without text expansion or reparsing generated source.
- Preserve random-access temporal evaluation and multi-backend lowering.
- Keep `.veac` as authoring truth and canonical IR as the execution, interchange, validation, and caching contract.
- Produce definition, call-site, logical-key, and entity provenance precise enough for diagnostics and agent source edits.
- Guarantee bounded, deterministic builds for identical declared inputs.

## Surface Model
The following uses the implemented Surface grammar:
```veac
struct Brand { accent: color, title_size: length, font: Resource, }

impl Brand @presentation {
    fn card(self, key: identifier, title: text, at: time) -> Item {
        let style = text_style(text_metrics(font_stack(font_resource_ref(self.font), []), weight_bold(), font_style_normal(), self.title_size, 0px, 1.2, self.accent), text_layout(text_box_width(640px), text_wrap_word(), text_overflow_clip(), text_align_center(), text_align_middle(), writing_horizontal_tb(), orientation_mixed()), text_path_none(), text_decoration(text_background_none(), text_outline_none(), shadow_none()), [], text_animation_none());
        let visual = visual_style(visual_layout(placement_anchor(anchor_center(), vector(0.0, 0.0)), frame_none(), transform_2d(transform_motion(point_constant(point(0px, 0px)), vector_constant(vector(1.0, 1.0)), angle_constant(0deg)), transform_geometry(vector(0.0, 0.0), flip_none(), vector(0.5, 0.5), crop_none()))), visual_surface(percent_constant(100%), compositing(0, blend_normal()), card_none()), [], color_pipeline_none());
        item(key, item_enabled(), during(at, 2s), source_text(title, style), source_timing_native()).with_visual(visual)
    }
}

fn main(context: Context) -> Project {
    let font = font_resource(identifier("font"), resource_file("assets/font.ttf"), sha256("0000000000000000000000000000000000000000000000000000000000000000"));
    let brand = Brand { accent: #ffffffff, title_size: 64px, font: font, };
    let state = track_state(track_playback_enabled(), track_audio_audible(), track_isolation_normal(), track_editing_unlocked());
    let layer = visual_layer(identifier("titles"), 0, placement_free(), state, track_routing_default())
        .with_item(brand.card(identifier("title"), "可执行 VEAC", 0s));
    let timeline = sequence(identifier("main"), "主时间线", sequence_settings(canvas(1280px, 720px), frame_rate(30, 1), 48000)).with_layer(layer);
    project(identifier("demo"), project_settings(600))
        .with_resource(font)
        .with_sequence(timeline)
        .entry(timeline)
}
```

`brand.card(...)` executes while building. Pure calls may fold; authored `animate` declarations
residualize approved temporal leaves from the same statically resolved function system.

## Compiler Architecture
```text
.veac source graph
  -> surface AST
  -> name resolution and type/effect checking
  -> Typed HIR
  -> verified executable Core v10
  -> bounded evaluator + one graph transaction
  -> freeze -> direct canonical project envelope
  -> validation -> RenderPlan -> backend lowering -> render executor
```

Typed HIR retains spans, symbols, type substitutions, units, ownership, inferred
effects, and time dependencies. It favors diagnostics over serialization stability.

Core v10 is a small typed instruction set for values, calls, bindings, branches,
bounded iteration, constructors, and graph emission. It defines evaluation order
and errors, pins the numeric domain opset/digest, and is versioned separately from
surface grammar and canonical schema. Runtime only accepts verified Core.

## Partial Evaluation

The compiler classifies dependencies, not function declarations:

- Calls with fully known inputs execute or constant-fold.
- Calls constructing `Project`, `Sequence`, `Layer`, `Item`, or related values execute during the build.
- Pure approved leaf computation depending on temporal inputs is residualized.
- A temporal dependency cannot escape into topology, identity, collection size, resource selection, or module loading.

The invariant is **static topology, dynamic leaf values**. Time-dependent creation,
removal, reordering, or renaming of a timeline entity is a compile error.

Local mutation is build-only and cannot leak shared mutable identity. Closures may
capture immutable serializable values. Domain values freeze before lowering.

## Temporal Programs

The published temporal model is pure, deterministic, and random-access:

```text
value_at(context) -> typed value
```

Inputs include sequence/clip/source time, frame, progress, and typed parameters. Declared analyzer
results are Build-stage source inputs and are resolved before residualization; a time-varying analysis
signal is not part of the published Temporal contract.
Residual programs use canonical backend-neutral expression/curve IR. Backends compile them
or emit a structured capability error; FFmpeg defines no VEAC semantics.

State, events, integration, feedback, particles, and previous-frame dependency
require a later RFC defining seek, warm-up, checkpoint, and render-shard rules.

## Canonical Envelope

The target executable canonical envelope contains:

- the frozen, typed project graph;
- closed, versioned temporal program definitions and typed bindings;
- a language/opset manifest and content digests;
- declared build-input identities;
- provenance from each generated entity and program to definition spans,
  call-site spans, the VEAC call stack, and stable loop logical keys.

Canonical IR contains neither surface AST nor unchecked source strings. Derived
bytecode is only a cache. Editing changes `.veac` and rebuilds the envelope;
generated IR is not decompiled or independently mutated back into source.

## Determinism And Budgets

Build execution may observe only the source graph, explicit build parameters,
versioned standard library, declared asset-metadata and analysis snapshots, and an
explicitly declared random seed. It has no ambient filesystem, network, environment, system clock,
process, thread, dynamic import, reflection, or implicit randomness capability.

Iteration order is specified. Numeric operations, unit conversion, comparisons,
overflow, non-finite values, and error propagation have normative semantics.
Equivalent declared inputs must produce byte-identical canonical envelopes.

The verifier enforces configurable, deterministic limits for source/module size,
instructions, call depth, recursion, loop iterations, heap and collection size,
emitted entities, residual program nodes, diagnostics, and output bytes. Budget
failure is a source-located language error, never a process crash or partial IR.

## Implemented Boundary

Opset v7 is the closed executable registry for project topology, resources, visual/audio/caption items,
typed transforms, effects, transitions, multicam, annotations, templates and deliveries. Root-local
`main(Context) -> Project` is the only production frontend; an entry cannot mix executable declarations
with a legacy `project` block. Authored `animate` declarations cover the approved dynamic leaves and lower
through verified Core into canonical Temporal programs. New mechanisms require closed operation contracts,
direct lowering, source editing and backend evidence, never property bags or generated-source reparsing.

## Non-Goals

- Becoming a general-purpose operating-system or web application language.
- Preserving historical `.veac` source syntax or expansion behavior.
- Interpreting surface AST during preview or render.
- Arbitrary I/O, FFI, package install hooks, `eval`, threads, or an event loop.
- JavaScript-style prototypes, reflection, or inheritance as foundational reuse.
- Time-dependent project topology or implicit stateful frame execution.
- JIT, production bytecode VM, pixel kernels, or an initial package registry.

## Delivery Gates

### G0: Semantic Contract

The charter, evaluation order, types/units, effects, Core IR, canonical programs, determinism,
and diagnostics are accepted. At least 15 video programs and invalid counterexamples form an executable conformance corpus.

### G1: Executable Build Core (implemented)

Typed HIR, Core IR, and a bounded evaluator support modules, `fn`, closures,
control flow, iteration, collections, structs, enums, methods, and domain values.
Repeated root-local `main(Context) -> Project` builds match; new language code exceeds 95%
line coverage, with CLI build and diagnostic integration tests.

### G2: Domain Migration And Unified Provenance (implemented)

Editing domains and reusable components use values and calls. Production never expands or reparses text.
Stable keys and source indexes trace entities through calls and modules.

### G3: Temporal Residualization (implemented)

Time-dependent pure calls serialize as canonical programs; constants, curves, expressions,
parameters, and time conversions share one model with differential backend tests.

### G4: Production Execution (implemented)

Supported examples build, preview, render, and preserve provenance. Cache keys cover source, inputs,
language, opset, and runtime; new backends require semantic-equivalence and isolation evidence.

No gate may weaken canonical validation, the 200-line limit, or unit/E2E guards.
