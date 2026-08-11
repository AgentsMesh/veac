# RFC: Executable Domain And Graph Values

Status: Implemented in opset v7

## Scope

This document fixes the executable model for video-domain values and graph assembly.
It follows nominal values and precedes temporal residualization. Historical project,
component, preset, and property-block syntax is not a compatibility constraint.

The domain API is not a second object-literal schema. Project entities are opaque,
typed values composed through a small standard library. Authors cannot construct or
project their storage fields, and adding a canonical JSON field does not add surface
syntax. This keeps language primitives stable while the canonical IR evolves.

## Domain Model

The implemented closed registry currently has 15 types:

```text
Context
Project, Sequence, Layer, Item, Relation
Source, Resource
Canvas, FrameRate, TimeRange, TextStyle, Transform, Transition, ContentIdentity
```

The current v8 family includes `Modifier`, `Delivery`, and `AudioMix` together with their closed
operations, verifier rules, lowering, and tests. A name is not an implemented type and cannot be
used in source unless it is present in the versioned registry.

`Project`, `Sequence`, `Layer`, and `Item` are graph containers. `Resource` is Project-owned and
`Relation` is Sequence-owned; neither is a container. `Source` may hold a non-owning Resource
reference, `TextStyle` holds a non-owning Font Resource reference, and a Relation holds two
non-owning Item references. `ContentIdentity` separates verified SHA-256 syntax from text. Other values are
immutable descriptions attached to containers. Closed variants
represent semantic choices such as media/text/generated sources and video/audio/
visual/caption layers. Raw maps and JSON strings are never accepted as domain values.

Domain type and callable names belong to the versioned standard library, not the
lexer keyword set. User declarations cannot replace standard symbols in the root
namespace; a lexical local may still shadow a callable at an expression site.

## Composition Surface

Construction uses typed functions and immutable methods rather than broad property
blocks. The exact catalog grows by domain capability, while its composition shape is
fixed:

```veac
fn title_card(key: identifier, title: text, duration: time, font: Resource) -> Sequence {
    let style = text_style(text_metrics(font_stack(font_resource_ref(font), []), weight_bold(), font_style_normal(), 64px, 0px, 1.2, #ffffffff), text_layout(text_box_width(900px), text_wrap_word(), text_overflow_clip(), text_align_center(), text_align_middle(), writing_horizontal_tb(), orientation_mixed()), text_path_none(), text_decoration(text_background_none(), text_outline_none(), shadow_none()), [], text_animation_none());
    let visual = visual_style(visual_layout(placement_anchor(anchor_center(), vector(0.0, 0.0)), frame_none(), transform_2d(transform_motion(point_constant(point(0px, 0px)), vector_constant(vector(1.0, 1.0)), angle_constant(0deg)), transform_geometry(vector(0.0, 0.0), flip_none(), vector(0.5, 0.5), crop_none()))), visual_surface(percent_constant(100%), compositing(0, blend_normal()), card_none()), [], color_pipeline_none());
    let item = item(identifier("title"), item_enabled(), during(0s, duration), source_text(title, style), source_timing_native()).with_visual(visual);
    let state = track_state(track_playback_enabled(), track_audio_audible(), track_isolation_normal(), track_editing_unlocked());
    sequence(key, "章节", sequence_settings(canvas(1080px, 1920px), frame_rate(30, 1), 48000))
        .with_layer(visual_layer(identifier("titles"), 0, placement_free(), state, track_routing_default()).with_item(item))
}

fn main(context: Context) -> Project {
    let font = font_resource(identifier("font"), resource_file("assets/font.ttf"),
        sha256("0000000000000000000000000000000000000000000000000000000000000000"));
    let timeline = title_card(identifier("chapter-1"), "第一章", 3s, font);
    project(identifier("launch"), project_settings(600))
        .with_resource(font)
        .with_sequence(timeline)
        .entry(timeline)
}
```

Constructors require semantic arguments; they do not mirror every canonical field.
Small value types such as `TimeRange` and `TextStyle` may be nominal values when
their whole layout is useful to algorithms. Graph container storage remains opaque.
Defaults are versioned standard-library behavior and are materialized before the
canonical envelope is published.

Every method returns a new immutable handle. Source evaluation remains expression
oriented; there are no hidden statement blocks, ambient current-track variables, or
order-dependent property assignment. Receiver then argument evaluation is left to
right under the normal method contract.

## Identity And Ownership

Every graph container has an explicit `identifier` key. A child key is unique within
its parent and combines with the parent path to form a stable logical entity key.
List ordinal is provenance only and never identity. Reordering a generated list does
not rename surviving entities.

An entity has exactly one owning parent. Attaching the same entity handle twice,
using a duplicate child key, creating an ownership cycle, or attaching the wrong
domain kind is an authored build error. Cross-entity references use typed stable
keys, never pointers, list indices, or source spelling.

Domain handles are immutable and source-graph local. They cannot enter declared
inputs, cross compilation boundaries, be used as map keys, or be serialized apart
from their verified graph transaction. Lists and nominal values may contain handles
only while executing Build-stage code.

## Core Contract

Core v10 在每个 program 和 closure body 中固定 domain opset version 与标准 registry digest。
verifier、直接调用边界和 runtime 必须匹配同一 identity；未知或漂移的 ABI 在执行前失败。

Core adds closed `DomainConstruct` and `GraphEmit` instructions. `DomainConstruct`
creates a typed immutable description or unattached container. `GraphEmit` attaches
one or more children to a parent and returns the updated parent handle. Its opcode
contains a stable domain operation ID; it never names a Rust type or source method.

Each operation contract fixes operand types, evaluation order, output type, graph
kind, topology operands, leaf operands, retained bounds, and canonical lowering.
The standard-library surface call is resolved to that contract in Typed HIR. Runtime
does not dispatch by source string and cannot execute a Surface AST callback.

The verifier rejects unknown operations, wrong operands, forged handles, ownership
violations, missing keys, invalid topology stages, understated resource bounds, and
non-`GraphEmit` metadata. It verifies all callable bodies before any build executes.

The evaluator owns one transactional graph arena. All emissions reserve entity and
byte budgets atomically before mutation. Any later failure discards the arena, so a
failed program cannot publish a partial project, source index, or cache artifact.

## Effects And Stages

Description construction is `Pure`. Container creation and attachment are
`GraphEmit`; immutable return values do not weaken that effect. Graph values are
Build-stage shapes. A function's effect and stage remain inferred from the resolved
operations actually called.

Entity keys, kinds, parent choice, order, collection size, resource/source selection,
relations, and delivery topology must be at most Build stage. Approved visual,
audio, text, timing, and effect parameters are leaf operands and may later carry a
Temporal residual program. A Temporal value cannot choose whether an entity exists.

### Accepted Amendment: Transactional Topology Generation

Bounded `map` callbacks, including callbacks synthesized by Surface `for`, may have
inferred effect `GraphEmit`. This is transactional topology generation, not `Pure`
iteration. The callback must be verified Core and runs in the evaluator's single
graph transaction. `map` reserves its finite iteration and output-collection budget
before the first callback; every domain operation still atomically reserves its own
entity and emitted-byte delta before mutating the arena.

`filter` and `fold` callbacks remain `Pure`. They cannot conditionally retain or
reduce emitted entities, and `LocalMutation` is not admitted for any collection
callback. Any callback failure, later instruction failure, ownership error, or
budget error taints the transaction. The arena is then discarded and cannot publish
a partial project, source index, or cache artifact.

Effectful iteration does not synthesize identity. Every emitted entity key remains
an explicit constructor operand. Duplicate keys, repeated ownership, stale handles,
and cycles still fail during attachment or freeze. Iterable cardinality, entity
existence, keys, parent selection, ordering, and every downstream topology sink must
remain at most Build stage; Temporal data may only flow to approved leaf operands.

## Root And Canonical Boundary

An executable entry exports exactly one `fn main(context: Context) -> Project`.
The host creates an empty protected `Context` in the same graph arena. Declared parameters, asset
metadata, and Build-stage analysis values enter separate stable typed Core input slots through the
versioned manifest; they are not fields of `Context` and cannot be ambient inputs. No media catalog,
filesystem, clock, network, environment, or random source is visible.

The returned project must be closed, connected, and have one valid entry sequence.
Freezing the graph materializes defaults, resolves typed references, validates the
canonical model, and writes `ProjectEnvelope` directly. Production execution must
not render a domain value to `.veac`, reparse authoring text, or round-trip through
an intermediate Surface document.

Canonical output includes stable entity keys and provenance for constructor span,
method call span, definition, VEAC call stack, and enclosing loop logical keys.
Source edits target those originating `.veac` sites and rebuild from source truth.

## Completed Migration And Conformance

Migration proceeded container-first: root project and sequence, layer and item,
sources and resources, modifiers and relations, then delivery. Every published capability now has one
executable path. Generated-source text expansion and mixed ownership are forbidden.

Current opset v8 has 214 Domain types and 582 numeric operations. It covers generated and media sources,
identity-pinned resources, text/caption, audio processing, transforms, effects, relations, multicam,
templates, annotations and delivery. Approved temporal leaves are authored in `.veac` and residualize from
verified Core without changing topology.

Completion requires unit tests for every operation contract, raw-Core corruption,
exact and one-short budgets, deterministic keys, rollback, effect/stage sinks, and
ownership. Integration, CLI, source-edit, and canonical round-trip tests must cover
modules, functions, closures, collections, methods, and invalid counterexamples.
Chinese examples must demonstrate visible editing outcomes. New language code must
exceed 95% line coverage and every controlled file must remain below 200 lines.
