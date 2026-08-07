# 可执行语言资源预算

所有 `SourceLoader` 返回的源码都会由 compiler 重新计量；自定义 loader 不能绕过限制。预算值是
deterministic logical cost，不依赖 allocator capacity 或平台对象大小。

## 解析与解析图

| 资源 | 上限 |
| --- | ---: |
| 单个 source module | 16 MiB |
| 完整 source graph | 1,024 modules / 64 MiB |
| 单个 lexer input | 1,000,000 tokens |
| import/callable dependency active depth | 64 |
| 单个 dependency graph | 16,384 resolved nodes |
| 单个 expression | 64 层 / 1,024 nodes |
| function parameters / call arguments | 各 64 |
| closure captures | 64 |
| function call depth | 64 |
| 单个 text value | 1 MiB |
| diagnostics | 256 |

dependency cycle 先于 depth/node limit 报错。所有 size arithmetic 使用 checked operation；溢出和越界
产生稳定 diagnostic，不分配候选大对象。

## 类型与标准库

| 资源 | 上限 |
| --- | ---: |
| 单 module nominal declarations | 256 |
| 单 struct/enum members | 64 |
| type registry | 1,024 definitions / 8 MiB |
| value type nesting / arity | 32 / 64 |
| 单 type methods | 64 |
| method registry | 1,024 definitions / 8 MiB |
| Domain registry | 1,024 operations / 1 MiB |
| 单 Domain operation operands | 16 |
| source-graph retained symbols | 64 MiB |

function、method、nominal type 和 imported alias 在完整 source graph 中只计量一次 payload。alias 计入新
key 的 logical bytes，但共享 immutable compiled definition。resolver 先预检完整 batch，再原子提交，
不会留下半个 scope。

## Build Execution Ledger

一次 `main(Context) -> Project` build 的所有 module、function、method、closure、collection callback 和
Temporal residualization 共享同一个 ledger：

| 资源 | 上限 |
| --- | ---: |
| verified Core fuel | 1,048,576 instructions |
| evaluated value payload | 64 MiB |
| aggregate bounded iterations | 1,048,576 |
| aggregate collection elements | 1,048,576 |
| logical collection storage | 64 MiB |
| emitted Domain entities | 65,536 |
| emitted Domain payload | 64 MiB |
| aggregate residual nodes | 262,144 |
| aggregate residual storage | 64 MiB |
| evaluator slots | 64 MiB，按 slot 计 64 logical bytes |

一次 reservation 会先检查全部 dimension，再一起 commit。进入 import、函数、closure 或下一次
iteration 不会重置计数；失败会回滚完整 graph transaction。list/map/tuple、GraphEmit owner attachment
和 residual node 都在分配前收费。所有 authored Temporal leaf 复用同一个 typed residual ledger view；
每个 canonical program 可以拥有独立 builder，但 builder 不能拥有或重置 source-graph counter。每个
residual node 按 64 logical bytes 加其 typed value payload 计入 aggregate residual storage。

单个 Temporal program 还受 canonical IR 的 65,536 nodes、256 inputs 和 value-size guards；library
最多 4,096 programs、65,536 bindings、262,144 aggregate nodes。compiler 只发布 verified Core
residualization 结果，runtime 不执行 Surface AST。

## Source Edit

| 资源 | 上限 |
| --- | ---: |
| preconditions / operations | 各 4,096 |
| 单 expression/body/declaration fragment | 64 KiB |
| batch fragment payload | 16 MiB |
| edited module output | 16 MiB |
| string rewrite working set | 48 MiB |
| SourceEdit JSON input | 64 MiB |

range、UTF-8 boundary、overlap、typed target 和 fragment syntax 在 rewrite 前验证。48 MiB working set
只计原 module、replacement payload 与 prospective output；成功 preview 保留重新 build 的 executable
source graph 与 revision。dry-run 和失败 transaction 都不会写源文件。
