# Compile-Time Programming

VEAC's programming layer is a static authoring system. It resolves a confined source graph,
evaluates pure expressions, and expands presets and components before the core authoring
`Document` is parsed. No module, expression, preset, component, or instance survives in canonical
JSON IR.

```text
entry .veac + imported modules
  -> resolve names and exports
  -> type-check and evaluate expressions
  -> expand typed presets and sequence components
  -> core authoring Document
  -> canonical JSON IR
```

The executable example links its [`entry`](../../examples/programming-language/main.veac), [`brand module`](../../examples/programming-language/brand.veac), and revision-bound [`source edit batch`](../../examples/programming-language/source-edit.json).
Its preview publishes the matching revision, semantic index, and dry-run outcome without rewriting either `.veac` module.

## Modules

An entry owns declarations plus exactly one `project`. An imported file owns one `module`:

```veac,fragment
import "./brand.veac" as brand;

module {
  export const length title_size = 64px;
}
```

`module` is a file-kind marker, not a second identity declaration. It is deliberately written as
`module {}`: the canonical module identity is the loader-provided root-relative source ID, while the
caller chooses its local namespace with `import ... as <alias>`. There is no self-name that can drift
away from either identity.

Only `export` declarations cross the module boundary. References are qualified through the import
alias, such as `brand.title_size` or `brand.title_card`. Imports must be relative and form an
acyclic graph. VEAC opens every path segment relative to an already-open source-root descriptor and
does not follow symlinks. Absolute paths, `..`, symlinks, root escape, duplicate aliases, project
files used as modules, and import cycles are errors.

Canonical source IDs are UTF-8, root-relative `/`-separated paths of at most 4,096 bytes. Empty,
`.` and `..` segments, `:`, `\`, control characters, and the reserved transaction path
`.veac-source.lock` are rejected at the compile boundary. Filesystem loading accepts only regular
files opened with nonblocking type inspection. Two lexical IDs may not resolve to the same physical
`(device, inode)` file; revisions and source edits therefore cannot diverge through case-folding,
Unicode-normalization, or hard-link aliases.

## Constants And Pure Expressions

```veac,fragment
const time duration = clamp(base + 500ms, 2s, 3s);
const time base = 2500ms;
const text title = "同一组件" + "，再次实例化";
```

Constants may refer forward to other constants. Resolution uses a dependency graph and rejects
cycles. Values have one compile-time type:

```text
scalar | time | length | percent | angle | text | color | bool | identifier
```

Every declaration name, expression symbol segment, import alias, and source-edit target name uses
one lexical contract: 1 to 128 ASCII bytes matching
`[A-Za-z_][A-Za-z0-9_]*(-[A-Za-z0-9_]+)*`. `true` and `false`
are reserved boolean literals, not legal names. A qualified reference joins canonical segments with
one dot, for example `brand.title-card`; a declaration name itself never contains a dot. Unicode is
fully supported inside quoted text, but not in semantic names. This makes every accepted declaration
directly referenceable by an expression and addressable by a source edit.

Bare identifiers resolve names from the lexical environment. Use `identifier("cover-art")` when an
expression must construct an identifier value, including a resource reference. It enforces the same
name contract, and its result is emitted as syntax rather than text. The nine type names above are
the only accepted spellings; aliases such as `number`, `string`, `boolean`, and `resource` are not
part of the language. `resource(...)` is likewise not a function alias.

Arithmetic is exact rational arithmetic. `+` and `-` require matching numeric dimensions; text
supports `text + text` concatenation. Every evaluated text value, whether it came from a literal,
an environment binding, or concatenation, is limited to 1 MiB and deterministically rejected with
`EXPRESSION_TEXT_LIMIT` above that bound. Multiplication and division require a scalar or percent
according to the dimensional rules, and dividing equal dimensions yields a scalar. Parentheses and
unary `+`/`-` are supported. `min(a, b)`, `max(a, b)`, and `clamp(value, min, max)` require one
numeric type. `${expression}` injects a type-safe source literal into a component or project body.
Line comments (`// ...`) and block comments (`/* ... */`) are accepted anywhere expression
whitespace is accepted. Because `-` is legal inside a name, subtraction after a symbol is written
with whitespace (`duration - offset`); maximal-munch `title-card` is one symbol.
Trailing or repeated dashes are rejected, so `foo--bar` remains the unambiguous expression
`foo - -bar`.

Numbers remain exact rationals throughout parsing, type checking, dependency resolution, and
evaluation. Injection into core authoring syntax is the explicit materialization boundary: scalar,
length, percent, and angle values become deterministic decimal literals because their core fields
use floating-point numbers. Time remains exact and must be representable as an integer `s`, `ms`,
or `us` literal. A value such as `1s / 3` is rejected with
`PROGRAM_EXPRESSION_MATERIALIZATION` instead of being silently rounded.

Expressions cannot read files, environment variables, clocks, probes, networks, or random state.
They cannot mutate state or invoke user code. Evaluation and expansion are bounded.

## Typed Presets

Presets reuse a coherent semantic block, not a flat property map:

```veac,fragment
preset text-style readable { fill #ffffffff; }
export preset text-style title {
  use text-style readable;
  size 64px;
}
```

The closed kinds are `text-style`, `text-layout`, `modifier-stack`, `effect-pipeline`,
`color-pipeline`, `audio-processors`, and `delivery-profile`. `use <kind> <name>;` is legal only at
a matching site. Presets may compose presets of the same declared kind. The enclosing declaration,
the `use` kind, and the referenced preset kind must all match; an empty preset body cannot bypass
this check. Kind mismatches, missing names, and cycles fail before core parsing.

## Sequence Components

The first component primitive produces a sequence:

```veac,fragment
export component sequence title_card {
  param text title;
  param time duration default 3s;
  param time accent_duration default duration / 3;
  slot visual backdrop;
  instance sequence @visuals from card_visuals {
    bind duration duration; bind accent_duration accent_duration; fill backdrop { source slot backdrop; }
  }
  body {
    layer visual @copy {
      item @title {
        source text { content ${title}; style { use text-style title; } }
        record { at 0s; duration ${duration}; }
      }
    }
  }
}

instance sequence opener from brand.title_card {
  bind title "可复用标题";
  fill backdrop { source generated solid { color #123047ff; } }
}
```

Parameters are typed. Defaults may refer to constants or other parameters and are dependency
resolved. Every declared slot must be filled once with a compatible source; unknown, missing, or
wrong-kind fills are errors. A fill contains exactly one complete source declaration. Additional
item members such as `record`, `state`, or `modifiers`, and a second source declaration, are rejected
before injection. Slot kinds are `video`, `audio`, `visual`, `text`, `caption`, and `sequence`.

Component bodies and parameter defaults use the component's definition scope. `bind` expressions
and `fill` bodies use the instance caller's scope. A fill may therefore use caller constants and
qualified imported presets, but it cannot see component parameters or private declarations from the
component's module. VEAC fully expands and type-checks each fill in the caller scope before injecting
the resulting source into the component body. Caller-local `@id` references are hygienized against
that caller instance before injection, so a nested child fill can refer to a sibling without being
captured by an identically named child local. Component definitions cannot capture project resources
or sequences by name; expose those dependencies as typed parameters or slots.
Definition checking runs against two identity-disjoint synthetic projects, so success cannot depend
on a fixture name that happens to match a caller-owned resource, sequence, or component instance.

A component may own `instance sequence @local from component { ... }` declarations before its
`body`. The `@` marks a compile-time local sequence, not a project-owned runtime construct. Its
component reference resolves in the parent's definition scope, so an exported component may compose
a private sibling component or a qualified import without exporting that implementation detail.
The nested `bind` and `fill` caller scope is the currently expanded parent: it contains the parent's
bound parameters plus declarations captured at the parent definition. The child's defaults and body
still use the child's own definition scope. This separation prevents the top-level caller from
accidentally leaking names through the parent while allowing explicit parameter forwarding.

`fill child_slot { source slot parent_slot; }` explicitly forwards a parent's already type-checked
slot fill to a child. After binding and filling, VEAC recursively emits each child as a top-level core
sequence and rewrites `source sequence sequence @local;` in the parent body to that exact sequence
ID. The expanded `Document` therefore contains only ordinary sequences and references: no component,
instance, closure, script, or deferred evaluation reaches canonical JSON IR.

The top-level instance ID becomes its generated sequence ID. Component-local paths expand to
`veac-h-<root-byte-count>-<root>-<local-byte-count>-<local>...`, adding one length-framed segment at
each composition or local-declaration level. This encoding is injective even when names contain
delimiters. `veac-h-` is reserved: an explicit source ID may not claim a generated name. Generated
IDs must fit the 128-byte identifier bound; VEAC never silently hashes an oversized ID. Agents should
edit `@name` and use provenance's `get_local(root, name)` or
`get_local_path(root, &[child, local])` lookup instead of synthesizing generated IDs.

## Resource Budgets

All public compilation paths enforce the deterministic limits in the focused
[programming resource budget](programming-limits.md).

## Static Boundary

Expansion must produce valid core authoring syntax. VEAC then parses and lowers that `Document`
through the same canonical validator used for a non-program source. Planner and backend layers only
consume the expanded canonical project. This keeps execution deterministic and prevents runtime
scripts, hidden I/O, or agent-specific behavior from entering the IR.
