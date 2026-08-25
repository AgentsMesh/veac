# 编译器物理边界

当前 `veac-lang` 已把 execution-free 类型模型迁入 `veac-lang-model`，把闭合 Domain contract 迁入
`veac-domain-spec`。其余 frontend、source graph、Core、evaluator、domain graph、Temporal、lowering 与
source edit 仍由同一个 facade crate 拥有。该状态是可运行迁移点，不代表最终物理边界。

## Syntax 与 Source

每个 source file 只有一个 `SyntaxDocument`，它保存原始 UTF-8 bytes、token、comment、whitespace 和绝对
slice。declaration、body、expression 与 Temporal parser 都消费这个共享文档，因此 source edit 不从 AST
或 canonical IR 重建源码。

当前 declaration parser 与 expression parser 仍是两个消费同一 token/slice 合同的 parser subsystem，
尚未物理统一为单一 typed CST。统一时必须保留 production diagnostics、standalone expression API、
lossless roundtrip 与 source-edit typed target；不能以重新 lex body string 的方式伪造统一。

## Query 与 Execution

`CompilerDatabase` 缓存 syntax、module interface、typed HIR 和 verified Core。interface key 绑定完整
source graph 的 source bytes、source ID、`(importer, requested_path, resolved_source_id)` 路由边、compiler ABI
与 transitive dependency revision；变化会失效 importer 的语义查询。当前 interface 不是独立 module query，
HIR/Core 也按一次完整 function batch/context 缓存，因此“未变化模块继续命中”不能理解为细粒度增量编译。
failed query 与 graph-affine `BuiltProgram` 不缓存，
容量与 retained bytes 都有上界。依赖图也按完整双向边、路由字符串、pending key 和容器开销计费；
超预算时原子清空图和语义缓存并保守重算，不能保留部分 reverse projection 或继续命中 stale value。
查询数据库以 lifecycle write barrier 线性化显式 clear 与依赖 reset，并用不可回绕 epoch 拒绝旧 query 的
late insert；计费溢出永远 bypass，clear 同时释放 FIFO backing allocation。

execution、graph transaction、freeze、Temporal residualization、canonical lowering 和 validation 每次完整
执行。clean build 永远是语义 oracle。当前数据库是有界、可丢弃缓存以及正确失效/并发屏障；按
module/declaration 拆分的 query DAG、增量 HIR/Core 与增量 lowering 仍是下一阶段，只有在 immutable inputs
和 clean build differential test 证明后才可加入。

## Package 与发现

package trust loop、exact identity、manifest/lock/API metadata、source ownership、root confinement 和
`CompositeSourceLoader` 已实现。project 可以导入 exact `package:name@version/path.veac`，package 内请求
必须经过 importer 自己的 lock closure 与 direct dependency edge。

compiler source identity 与物理发现位置是两套正交合同。package module 的逻辑 ID 始终是
`packages/name@exact-version/package-relative-path`；lock 的 root-relative locator 只用于 confinement、inode
与 digest 绑定。vendor 布局不进入 import route、source revision、ModuleInterface、package API 或 nominal
type identity。多个 mount closure 复用同一 exact package 时必须具有相同 portable contract，否则
`CompositeSourceLoader` 自身在 mount 阶段拒绝，不能依赖 CLI 的前置校验。

CLI 只挂载显式提供且独立验证的 package root。package source 携带 `ReadOnlyDependency` authority，project
source 保持 `Project` authority；source edit selection 与 publication 消费该权威。package bytes 仍进入
complete graph revision，因此 package 变化会使旧 edit batch 失效，但 transaction 不能写 dependency 文件。

当前 `package search` 面向显式 local store；还没有网络 registry、install/update、版本求解或全局隐式发现。
这些能力进入 CLI 前必须保持 lock-first、离线可重现，并且不能把 ambient store 状态带入 source identity。

## 拆分顺序

后续物理拆分按依赖方向逐步进行：

```text
veac-syntax -> veac-source -> veac-core -> veac-eval
            -> veac-compiler -> veac-domain -> veac-temporal -> veac-lower
```

每一步先迁 ownership 与 public contract，再由 `veac-lang` 做薄 facade；禁止复制实现后长期双写。迁移必须
保持 workspace tests、clean/cached differential tests、95% 覆盖率和严格少于 200 行的受控文件门禁。
