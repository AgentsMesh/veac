# Source-Of-Truth 编辑

`.veac` 是源码真相时，Agent 必须编辑 source graph，再重新执行并生成 canonical JSON IR；不能修改
IR 后猜测如何反编译。VEAC 为此提供闭合、版本化的 `SourceEditBatch`，不公开 raw byte-range patch。

## Revision 与 Inventory

```bash
veac source-revision main.veac
veac source-index main.veac
veac schema --contract source-index
```

revision 是所有已加载 module 的 canonical source ID 与精确 UTF-8 字节的 SHA-256。空白和注释也参与
并发控制；inode、mode 与 parent identity 不进入可移植 revision，而由提交期 filesystem 护栏验证。

`source-index` v8 是无需 Build input 值的静态 Agent discovery boundary。输出固定包含 graph
`revision`、按名字排序的 `build_inputs`、按 module 排序的 `modules` 和按 typed path 排序的 `nodes`：

- Build input 发布 name、role 和闭合 value type；payloadless enum 还发布 nominal TypeId、definition
  digest 与有序 variants，使 Agent 能先发现 `locale` 的合法选择，再调用 build/check/source-edit；

- module 发布 body insertion `range`、typed `imports` 和可寻址顶层 `declarations`；
- import 包含 `{ module, alias }` target、解析前 path、精确 source 与观察到的 byte range；
- 顶层声明包含 input、const、fn、impl、struct、enum、animate 的 target、source 与 range；
- node 发布 closed `ExpressionSite`、`StatementSite`、`BodySite`、`DeclarationSite` 及对应精确
  fragment；function 与 method 的 expression/statement inventory 使用有界语义路径定位 local value、
  call argument、callback、control branch、match arm、nominal field 与 collection entry；statement
  fragment 从 `let`、`var` 或 `set` 开始，并保留结尾分号；
- range 只用于展示和实现 edit，公共身份始终是 module-qualified semantic target。

函数、method 和 animate body 都包含完整 `{ ... }`。exported 顶层声明的 range 包含 `export`；nominal
member 的 range 不包含分隔逗号。Agent 应把 inventory 的 revision、target 和当前 source 放入 batch
precondition，不应缓存 byte offset 或执行后生成的 canonical entity ID。

## Source-Edit v6

```json,source-edit-batch
{
  "schema": "https://veac.dev/schemas/source-edit",
  "schema_version": 6,
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
        "path": { "kind": "constant", "constant": "section_duration" }
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
        "path": { "kind": "constant", "constant": "section_duration" }
      },
      "site": { "type": "constant_value" },
      "expression": { "source": "6s" }
    }
  ]
}
```

机器 schema 由 `veac schema --contract source-edit-batch` 生成。v6 的操作分成三组：

- `expression_equals`/`set_expression` 编辑 const，或 function/method body 中由
  `body_expression` + `SourceExpressionPath` 寻址的 pure expression；
- `statement_equals`/`set_statement` 编辑 function/method body 中完整的 `let`、`var` 或 `set`
  statement；`body_statement` 使用以 `local_value` 结尾的 `SourceExpressionPath`，replacement 必须包含
  operation、binding、右值和结尾分号；
- `body_equals`/`set_body` 编辑 function、method 或 temporal body 的 typed block；
- `declaration_equals`/`set_declaration` 编辑 input、nominal member 或 animate declaration site。

source-of-truth 结构编辑使用完整顶层原语：

- `top_level_declaration_equals`、`set_top_level_declaration`、`remove_declaration`；
- `insert_declaration` 配合 `module_start`、`module_end`、`before_declaration`、
  `after_declaration`、`before_import` 或 `after_import` typed anchor；
- `import_exists`、`import_absent`、`import_equals`、`insert_import`、`remove_import`。

anchor 自身带 target module，必须与 operation module 完全相同。两个 insert 不能占用同一零宽 anchor；
找不到 target、target 不唯一或 text edit 真正重叠都会拒绝整个 batch。`ImportSource` 只暴露 path 与 alias，
由 formatter 生成 import syntax；调用方不能注入任意 import statement。

每个 replacement fragment 先由 production lexer/parser 独立验证。顶层 fragment 必须恰好包含一个 input、
const、fn、impl、struct、enum 或 animate，不能包含 import、module wrapper 或第二个 declaration。随后候选
source graph 会完整 parse、resolve、type/effect-check、verify、执行、freeze、lower 并做 canonical validation。
因此单独合法但 owner 错误的声明，例如 imported module 中的 root-only input，也会 fail closed。
每个 changed module 还必须在候选 graph 中保持 reachable，不能通过同批移除 import 绕过其完整验证。

每个 `impl` 必须声明源码可见的 `@identity`，完整 target 使用 module、receiver 与 implementation
identity。method target 刻意保持 module、receiver 与 method，因此移动 method 不会改变其 Agent 地址。

嵌套 expression path 不是 byte offset。每一步都是 closed typed step，例如带 binding、operation 与
statement ordinal 的 `local_value`，带 ordinal 的 `call_argument`，或带 closed pattern 的
`match_arm`。路径最大 128 步且不能为空；replacement 先独立 parse，再在完整 owner scope 中重新
type/effect-check。这样 Agent 可以只替换 `map` callback 中的一个表达式，同时 revision precondition
仍保护整个 source graph，不必提交整个 function body，也不会按字符串搜索字段。

statement fragment 直接交给 expression parser 的 standalone statement entry 解析，不会拼接临时
function 或 synthetic `.veac` program。这个局部检查只确认恰好存在一个完整 statement；owner 中的符号、
类型、effect 与 mutable binding 规则仍由候选 source graph 的完整重建统一验证。

## 多模块事务

```bash
veac source-edit main.veac source-edit.json --dry-run
veac source-edit main.veac source-edit.json
veac source-edit main.veac source-edit.json --inputs build-inputs.json
veac source-edit main.veac source-edit.json --inputs build-inputs.json --input locale=zh-Hans
veac source-edit main.veac source-edit.json --output revised.veac
```

一个 batch 可同时修改多个 module，例如在被导入模块 rename API，同时替换 entry 的 import alias 与调用
body。所有 operation 使用同一 base revision；VEAC 按 module 生成 deterministic change set，再只构建、
执行和验证一次候选 graph。任何 contract、parse、resolve、type、runtime、lower 或 IR validation 错误都不
写源码。Build input 项目必须通过 `--inputs`、可重复的 `--input NAME=VALUE` 或两者组合，提供与候选
执行相同的完整 typed bindings，不能回退为空值或 ambient state；inline 同名值显式覆盖 manifest base。

新插入的 import 可以加载 source root 中原 graph 未触达的既存 module。候选 build 的全部未编辑依赖也
会成为 commit guard；preview 后这些依赖发生 byte、path type、parent 或可观察 identity 变化时，提交拒绝。
移除 import 的旧 graph 仍由 previous revision 保守复核。

build、check、format、source-index、source-revision 和 source-edit preview 在加载整个 module graph 时，
会在 `.veac-source.lock` 持有 shared snapshot lock；in-place commit 在同一文件持有 exclusive lock。
writer 会在锁内验证旧 graph，descriptor-relative 打开所有 changed module 与 dependency guard，预先创建
全部 replacement 和 rollback stage，再做一次全量 identity、mode、parent 与 byte 检查后依次
`renameat` 发布。中途失败会逆序回滚已发布 module；回滚也失败时返回 `WRITE_COMMIT_UNCERTAIN`。
POSIX 没有多文件单 syscall CAS，因此所有 reader 和外部 writer 都必须遵守同一 advisory lock 协议。

`--dry-run` 只返回 preview。独立 `--output` 只允许一个 changed module；多模块 batch 必须原地提交，
否则返回 `SOURCE_EDIT_OUTPUT_REQUIRES_SINGLE_MODULE`。report 使用 `modules` 与 `destinations` 明确实际
change set。lock 文件不是合法 module ID，且不会被输出覆盖；read-only 操作创建或复用这个持久
coordination 文件不属于 source mutation。

## 两个编辑边界

```text
SourceEditBatch -> .veac source graph -> execute/lower -> canonical JSON IR
EditBatch       -> canonical JSON IR  -> new canonical JSON IR revision
```

两个 schema 互不替代。前者保留可编程源码为 truth，后者只编辑已有 canonical artifact；VEAC 不提供
IR-to-`.veac` decompiler。完整 owner hierarchy 见[稳定 source addressing](source-addressing.md)。
