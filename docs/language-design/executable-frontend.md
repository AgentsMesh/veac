# 唯一可执行 Frontend

VEAC CLI 只接受 executable source。入口必须提供：

```veac
fn main(context: Context) -> Project { ... }
```

CLI 不再暴露 `auto`、`authoring`、`--frontend` 或独立 `compile` 命令，也不会在 executable
失败后回退到另一套 parser/lowering pipeline。

## 生产调用图

```text
build
  -> program::build_path_with_root
  -> verified Core execution
  -> frozen domain graph
  -> canonical ProjectEnvelope validation

check
  -> 与 build 相同的完整执行和验证，但不写 IR

fmt
  -> executable parse/resolve/type preparation
  -> 仅重写 trivia 的 syntax-aware formatter
  -> token/comment 保真检查 + 再次 parse/resolve/type preparation

source-revision / source-index
  -> program::prepare_path
  -> executable source graph index

source-edit
  -> apply_executable_source_edit_path_with_root
  -> overlay rebuild + Core execution + canonical validation
  -> revision revalidation + atomic commit
```

### Lossless Syntax Contract

The executable parser creates one `SyntaxDocument` per source file. It owns the exact UTF-8 bytes,
non-trivia tokens, line/block comments, and whitespace gaps as one contiguous sequence of spans.
`SurfaceFile` declarations retain `SyntaxSlice` ranges into that document; a slice never owns a
second body or expression string. The document source is therefore byte-identical on roundtrip,
including comments, unusual spacing, and invalid-but-tokenizable trivia.

Function, method, constant, and `animate` production paths consume their absolute token slices.
Source indexing reuses the same outer token array instead of lexing the file again. Expression
parsing may adapt a declaration slice to expression tokens, but must preserve the slice's absolute
byte origin when reporting diagnostics. Standalone expression-string APIs remain available for
callers that do not have a `SyntaxDocument`.

`build` 与 `check` 执行 `main`；`fmt`、`source-revision` 和 `source-index` 只 prepare，因此可以检查、
索引并修复一个包含 runtime failure 的 source graph。`source-edit` 必须执行编辑后的 graph，只有完整
build 成功才允许发布。

`fmt` 对入口与独立模块使用同一套语法和语义前端。它只规范空白、缩进和换行，不改写 token spelling、
字符串、颜色、数字或注释内容；格式化前后都 fail closed，并保证确定性、幂等和唯一末尾换行。

## 已删除边界

以下能力不属于生产语言：

- CLI frontend 自动分类和显式 frontend selection。
- CLI authoring `compile` 命令。
- CLI check/fmt/source commands 的 authoring 分支。
- CLI source-edit preview 的 authoring/executable sum type。
- `SourceFrontend` / `classify_source_frontend` 和 fallback dispatch。
- public `CompiledProgram`、legacy `compile`/`expand` 与 generated-source reparsing。
- generic legacy source transaction 和 authoring `Document` preview。

共享的 source loader、typed parser model、source index selection/revision、vocabulary descriptors 与
canonical validators 仍是 executable compiler 的普通内部模块；共享实现不形成第二个 frontend。

## 不变量

- 一个 CLI source command 对应一个 frontend，错误不触发 fallback。
- executable source 是 source of truth；IR 不反编译回 source。
- source edit 在写入前重新执行并验证完整 edited graph。
- 本地素材 URI 始终以 source root 为基准。
- 所有拆分后的 Rust、测试和文档文件保持少于 200 行。
