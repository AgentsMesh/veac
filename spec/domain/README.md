# Domain Family Opspecs

Files in this directory are authored machine-readable inputs to
`scripts/stdlib_codegen.py`. Every family is loaded from its JSON opspec;
its executable-stdlib Markdown is a documentation projection and is checked for
signature and declared-type parity during generation.

## Format

The closed `veac.domain-family` format has these top-level fields:

- `format`: the literal `veac.domain-family`.
- `format_version`: the loader contract version, currently `3`.
- `domain_opset_version`: the numeric runtime opset targeted by the family.
- `family`: the stable kebab-case generator family name.
- `types`: the declared Domain type identities. Every entry explicitly owns its
  stable `name`, numeric `opcode`, and closed `classification`.
- `operation_shards`: an ordered, unique list of root-confined JSON shard names.

Each shard uses the closed `veac.domain-operations` format and contains a unique
`operations` array. Shards keep each family reviewable and below the repository
file-size limit without participating in opcode assignment.

Each operation explicitly owns its stable numeric `opcode`, callable surface,
operand contracts, result, and execution semantics. `receiver` is either `null`
for a free function or an object containing its Domain `type` and operand
`axis`. Each parameter contains `name`, `type`, and an explicit `topology` or
`leaf` axis; list types use `list<Type>`.

The closed `semantics` object records `instruction`, `runtime_action`, `effect`,
`max_stage`, and optional `temporal_lowering`. These values are projected into
the generated Rust contract without name-based inference.

Array and shard order are non-semantic. The loader orders types and operations
by their explicit opcodes, so reordering cannot change runtime identity or
generated output. Duplicate type or operation opcodes fail closed, as do
unknown fields, values outside the closed vocabularies, duplicate callables,
and duplicate parameter names.

## Authority

All eleven family directories are authoritative machine sources. The Markdown
files under `docs/language-design/executable-stdlib-v6` are checked signature
and declared-type projections. Canonical Rust tables are generated from these
JSON files; neither Markdown, generated Rust, array order, nor Python naming
policy is an identity or semantics input.
