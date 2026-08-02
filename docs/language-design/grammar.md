# VEAC Authoring Grammar

This document describes the implemented `.veac` surface. The frontend first
resolves the static programming layer, then parses the expanded core grammar.
JSON source and canonical edit contracts are separate from this grammar.

## Lexical Rules

```ebnf
identifier = ( ASCII-letter | "_" ) , { ASCII-letter | digit | "_" }
           , { "-" , ( ASCII-letter | digit | "_" )
             , { ASCII-letter | digit | "_" } } ;
qualified-identifier = identifier , { "." , identifier } ;
string     = '"' , { character | escape } , '"' ;
number     = [ "-" ] , digit , { digit } , [ "." , digit , { digit } ] ;
time       = number , ( "us" | "ms" | "s" ) ;
ratio      = integer , "/" , positive-integer ;
color      = "#" , 6-or-8-hex-digits ;
comment    = "//" , { character } | "/*" , { character } , "*/" ;
```

Identifiers contain 1 to 128 ASCII bytes. Dashes separate non-empty name segments, so leading,
trailing, and repeated dashes are invalid; `true` and `false` are reserved literals. Declarations
contain one identifier segment, while imported references join canonical segments with `.`. The
same contract is enforced for expression symbols and source-edit target names. Identifiers are
semantic IDs and must be unique in their owning collection.
Strings are text or paths, numbers with units are typed values, and colors are
not arbitrary strings.

## Compile-Time Program

The module, expression, preset, component, and instance productions live in the focused
[compile-time grammar](programming-grammar.md). Their semantics and resource limits are specified by
the complete [compile-time reference](../language-reference/programming.md).

## Ownership

After static expansion, exactly one `project` is the core document root:

```ebnf
document = "project" , identifier , "{" , { project-member } , "}" ;

project-member = settings | resource | entry | multicam | sequence
               | delivery | annotation ;
sequence-member = layer | transition | apply | relation ;
layer-member = item ;
```

`multicam`, `sequence`, every delivery, and annotations are project-owned. A
delivery owns typed artifacts. A sequence owns timeline structure; a layer owns
items; one item owns one source. Moving declarations across owners is invalid.

The project entry is selected with `entry sequence <id>;`. Canonical settings
use typed fields such as `timebase 1/1000;`, `canvas 1080px by 1920px;`,
`frame-rate 30fps;`, and `sample-rate 48000hz;`.

## Resources And Streams

Media resources are closed by kind and location. The stream choices are only
`auto` and `disabled`; they lower to canonical
stream intent. Probe normalization records exact selections for planning.
Fonts and LUTs use the same project resource primitive, for example
`resource lut-3d show-look { locator local { path "color/show.cube"; } }`.

## Closed Item Sources

There are exactly six source variants:

```ebnf
source = media-source | text-source | caption-source | generated-source
       | sequence-source | multicam-source ;
```

Canonical examples are:

```veac
source media resource camera;
source text { content "Chapter one"; style { size 64px; fill #ffffffff; } }
source caption { text "Hello"; language "en"; speaker "host"; }
source generated solid { color #101820FF; }
source sequence sequence intro;
source multicam multicam interview { switch angle wide { at 0s; duration 4s; } }
```

Generated kinds are `transparent`, `silence`, `solid`, `gradient`, and `shape`.
A solid color is generated media, not a seventh `source color` variant. Caption
text is a source variant; a caption sidecar is a delivery artifact.

## Record And Source Time

Every item has one record range and may have one mapping:

```veac
record { at 2s; duration 4s; }
mapping linear { from 10s; to 14s; outside strict; }
```

The mapping variants are `linear`, `curve`, and `freeze`. Only media and
sequence sources accept mappings. Linear and curve accept
`strict`, `hold-first`, `hold-last`, or `hold-both`; freeze has no outside field.
Curve keys are ordered by record time. Source-time interpolation is only
`linear` or `hold`.

## Typed Item Modifiers

Item modifiers form a closed set rather than an open property bag:

```ebnf
modifier = layout | transform | composite | surface | mask | audio | color
         | effect ;
surface = "surface" , identifier , "{" , corner-radius , [ shadow ] , "}" ;
shadow = "shadow" , "{" , color , opacity , blur , offset , "}" ;
```

```veac
surface raised {
    corner-radius 36px;
    shadow {
        color #000000ff; opacity 38%; blur 28px;
        offset { x 0px; y 16px; }
    }
}
```

Surface lowers directly to the canonical card style. Generated shape geometry
continues to describe source pixels; it does not simulate item-level surface
semantics.

## Multicam

A project-level multicam group owns angles and sync; item sources own switches:

```veac
multicam interview {
    sync timecode { reference angle wide; }
    angle wide { source resource camera; source-offset 0s; }
    angle lav { source resource voice; source-offset 0s; }
}
```

An item references the group with `source multicam`; it does not declare a new
group. Its `switch angle <id> { at ...; duration ...; }` statements select
known angles within the item's record interval.

## Processing And Audio Routing

Editing operations are closed statements, not property bags. A `pipeline`
contains ordered color/effect stages; `scope composite-band`, `scope layer`, and
`scope items` create first-class Apply records.

LUT use is an ordered pipeline stage such as
`lut primary { resource show-look; interpolation tetrahedral; }`.

Audio layers declare processors and route to a named bus with `route bus mix;`.
Bus identity is created by referenced route IDs; there is no standalone `bus`
declaration. Each audio stem selects `master`, one `track`, or one `bus`.

## Deliveries And Artifacts

A project delivery selects one sequence, optional raster settings, and one or
more typed artifacts:

```ebnf
delivery = "delivery" , identifier , "{" , sequence-ref , [ raster ]
         , artifact , { artifact } , "}" ;
artifact = "artifact" , artifact-kind , identifier , "{"
         , target , artifact-recipe , "}" ;
artifact-kind = video | image-sequence | caption-sidecar | audio-stem | scope
              | audio-file | animated-image | still-image | adaptive-package ;
```

Targets are `file`, `pattern`, or `package`. Recipes use closed primitives such
as `mux mp4`, `encode png`, `encode mp3`, `frame containing`, and `package hls`.
Source selection, frame selection, raster, package, and codec settings stay
under their semantic owner; there is no anonymous recipe property block.

## Canonicalization And Validation

`veac fmt` is idempotent and preserves declaration order where order is
semantic. `veac check-ir` rejects unknown variants, unknown fields, duplicate
IDs, invalid ownership, unresolved references, illegal time mappings, and
unsupported artifact combinations. Successful compilation emits a fully
expanded typed `Document` and then canonical JSON IR. The IR contains no
imports, expressions, presets, components, instances, or scripts.
