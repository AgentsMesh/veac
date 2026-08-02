# Source-Of-Truth Editing

When `.veac` remains the source of truth, edit that source graph rather than mutating canonical IR and
attempting to decompile it. VEAC exposes a separate, closed `SourceEditBatch` contract for this workflow.

## Revision

```bash
veac source-revision main.veac
```

The revision is SHA-256 over the exact bytes and identities of the entry and every imported module. Comments
and whitespace therefore participate in concurrency control even when they do not change render semantics.

## Inventory

```bash
veac source-index main.veac
```

`source-index` is the agent discovery boundary for source editing. It emits the versioned
`https://veac.dev/schemas/source-index` contract with the graph revision and a deterministic `nodes` array.
Nodes are ordered by module and typed hierarchical path; expression entries use the closed `ExpressionSite`
order. Project, sequence, layer, apply, component, and other declared nodes remain discoverable even when
their `expressions` array is empty.

Each node contains its module-qualified `target` and authored byte `range`. Each expression contains its
typed `site`, exact current `source`, and zero-based UTF-8 byte `range` in that target's module. The ranges
let tools display or verify source, but they are observations rather than identities. Agents should copy
`target`, `site`, and the inventory `revision` into a `SourceEditBatch`, and may use current `source` in an
`expression_equals` precondition. They never need private parser ASTs or generated hygienic IDs. Generate
the report schema with:

```bash
veac schema --contract source-index
```

## Batch

```json,source-edit-batch
{
  "schema": "https://veac.dev/schemas/source-edit",
  "schema_version": 1,
  "operation_id": "op_change_section_duration",
  "base_revision": {
    "source_graph_sha256": "0000000000000000000000000000000000000000000000000000000000000000"
  },
  "atomic": true,
  "preconditions": [
    {
      "type": "expression_equals",
      "target": {
        "module": "main.veac",
        "path": {
          "kind": "constant",
          "constant": "section_duration"
        }
      },
      "site": { "type": "constant_value" },
      "expression": { "source": "brand.card_duration" }
    }
  ],
  "operations": [
    {
      "type": "set_expression",
      "target": {
        "module": "main.veac",
        "path": {
          "kind": "constant",
          "constant": "section_duration"
        }
      },
      "site": { "type": "constant_value" },
      "expression": { "source": "6s" }
    }
  ]
}
```

Generate the machine contract with `veac schema --contract source-edit-batch`. A target combines a
module-qualified typed node identity with a closed expression site; byte offsets are not a public identity.
Supported sites include constant values, component defaults and arguments, definition and timeline item
timing/state/text, resource locators, modifier parameters, and typed preset fields.

The complete name grammar, owner hierarchy, component-local paths, and explicit audio processor/EQ band IDs
are defined in [source addressing](source-addressing.md). Agents use those semantic identities rather than
occurrence indexes, byte offsets, processor kinds, or generated hygienic IDs.

A definition edit recompiles the complete graph, so every component instance and preset use site
observes the change or the entire transaction is rejected.

## Transaction

```bash
veac source-edit main.veac source-edit.json --dry-run
veac source-edit main.veac source-edit.json
veac source-edit main.veac source-edit.json --output revised.veac
```

A batch must match the exact source-graph revision and satisfy all preconditions. The current
transaction changes one module per batch and preserves every untouched byte range. Before commit,
VEAC resolves imports, evaluates expressions, expands presets/components, parses the resulting
`Document`, lowers it, and runs canonical validation. A stale revision, ambiguous target, invalid
replacement, or failed downstream invariant rejects the whole batch. In-place writes use verified
staging and atomic replacement.

For an in-place transaction, VEAC first computes a validated preview, then acquires an exclusive
`.veac-source.lock` in the entry source root. Under that lock it revalidates the root and lock-file
device/inode identities, rereads the complete graph from descriptor-relative non-symlink paths, and
requires every byte to match the preview. Target traversal, old-byte comparison, private staging,
and `renameat` publication all remain bound to opened root and parent directory descriptors. The
target and every path segment must still be a regular, non-symlink source at commit time.

The lock name is not a legal module ID, is a protected output, and is ignored by Git. `--dry-run`
and writes to an independent `--output` do not create the lock file; an explicit output that aliases
the edited module is treated as an in-place transaction. Concurrent cooperating VEAC transactions
fail with `SOURCE_LOCKED`, while detected graph, root, parent, target, or lock-path changes fail with
`SOURCE_CHANGED`. The lock is POSIX advisory: an external writer that ignores it cannot be made into
a portable filesystem CAS. Descriptor binding and repeated identity/content checks narrow that
race and fail fast when observable, but integrations that modify `.veac` in place must honor the
same lock protocol.

## Two Edit Boundaries

`SourceEditBatch` changes authoring expressions and returns a new source-graph revision. Canonical
`EditBatch` changes a canonical JSON project revision. They are intentionally different schemas:

```text
SourceEditBatch -> .veac source graph -> recompile -> canonical JSON IR
EditBatch       -> canonical JSON IR  -> new canonical JSON IR revision
```

There is no IR-to-`.veac` decompiler and no rule that writes generated hygienic IDs back into a
component definition. Choose the boundary whose artifact is authoritative and keep subsequent
operations on that boundary.
