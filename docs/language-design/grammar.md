# Executable Surface Grammar

The normative grammar is [VEAC executable programming syntax](programming-grammar.md). The lexer publishes
its exact spelling/position contract through `veac language-spec`; standard-library type and operation names
are resolved symbols, not an ever-growing keyword set.

## Compilation Units

```ebnf
entry = { import | declaration | temporal-declaration } ;
module = "module" , "{" , { [ "export" ] , declaration } , "}" ;
```

An entry must contain one root-local `fn main(context: Context) -> Project`. A module cannot declare
`main` for the entry and cannot own `animate`. Import paths are confined relative source IDs with cycle,
depth, byte and symlink-escape guards.

## Declarations

The executable declaration system contains typed constants, functions, structs, closed enums and immutable
methods. Reuse uses these ordinary language features. There is no parallel preset/component macro grammar,
generated-source interpolation or legacy `project { ... }` property syntax.

Expressions include exact literals/units, unary/binary operators, blocks, `let`, calls, field projection,
nominal construction, exhaustive `match`, `if`, closures, list/map/tuple values, finite ranges, `for`, and
bounded collection operations. Every expression lowers to typed HIR then numeric-ID Core.

## Temporal Declaration

```ebnf
temporal-declaration = "animate" , temporal-property , "on" , temporal-target ,
                       [ "using" , "resource" , resource-path ] , function-body ;
temporal-target = clip-target | text-target | clip-mask-target | clip-effect-target
                | apply-target | apply-mask-target | apply-effect-target ;
```

The property/target matrix additionally covers mask, text, closed `EffectParameter` values backed by existing
`Animatable<f64>` leaves, and apply leaves.
Item paths name project, sequence, layer and item keys; apply paths name project, sequence and apply keys.
Only Item targets may bind `source_time`; apply targets expose sequence time and frame only.

## Domain Construction

Project topology is expressed by statically resolved calls such as `project`, `sequence`, `visual_layer`,
`item` and owner methods such as `.with_item`. These are closed stdlib operations with Effect and Stage
contracts. Their argument lists are grammar-level function calls, not flattened fields.

The effect lattice is `Pure | LocalMutation | GraphEmit`; the stage lattice is
`Const < Build < Temporal`. Topology and owner selection must be at most Build. Only approved leaf values
may retain Temporal dependencies.

## Source Fidelity

Source spans and exact bytes remain outside Core for diagnostics, provenance and source editing. Core is the
only runtime input. Canonical JSON is the validated backend/interchange ABI. Neither Core nor JSON is rendered
back to `.veac`, and no execution path interprets Surface AST after verification.
