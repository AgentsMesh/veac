# 有界集合与迭代

VEAC 提供三个 closed generic operation 和一个 lexical iteration expression：

```veac,fragment
map(values, fn(value: T) -> U effect pure { transform(value) })
filter(values, fn(value: T) -> bool effect pure { keep(value) })
fold(values, initial, fn(total: U, value: T) -> U effect pure { combine(total, value) })
for value in values { transform(value) }
```

它们不是可导入、重载或覆盖的普通函数。类型检查在 Surface lowering 完成，Core 只保存已解析的
operation ID、typed operands 和 verified callback。运行时不查找名称，也不解释 Surface AST。

## Iterable 合同

iterable 只有三种：

| 输入 | callback 元素类型 | 顺序 |
| --- | --- | --- |
| `list<T>` | `T` | list 原始顺序 |
| `range<int>` | `int` | 半开范围的步进顺序 |
| `map<K, V>` | `(K, V)` | canonical key 顺序 |

`map` 返回 `list<U>`，`filter` 返回 `list<T>`，`fold` 返回 accumulator 类型 `U`；`for` 返回
`list<R>`。callback signature 必须完全匹配，不执行 numeric widening、cast 或动态 dispatch。

`map` 和 `for` 的 callback 可以是 `Pure` 或 `GraphEmit`，后者用于有界、事务化的拓扑生成；
它不会因为返回 immutable graph handle 就被视为 `Pure`。`filter` 和 `fold` 的 callback 必须为
`Pure`，因为 predicate 或 accumulator 不能决定已发射实体的去留。所有 collection callback 都
禁止 `LocalMutation`；独立 mutation 证据不能被更高的 `GraphEmit` summary 遮蔽。effect 在编译期
推导并写入 verified Core，运行时不信任 Surface 声明。function type 显式携带 effect bound，动态
function value 即使经过 list、map、tuple、nominal struct/enum 或嵌套组合，仍保留可验证的 callable
contract。`filter`/`fold` 只接受 `effect pure`；`map`/`for` 接受 `effect pure` 或 `effect emit`，并拒绝
`effect local`、`effect any` 以及任何实际含 `LocalMutation` 的 callback。

空 `[]` 和 `#{}` 会从显式 callback parameter 反推 iterable expected type。`fold` 的空 initial
也从 callback 第一个 parameter 取得 expected type。因此以下表达式都有完整静态类型：

```veac,fragment
map([], fn(value: int) -> text effect pure { "empty" })
map(#{}, fn(entry: (text, int)) -> int effect pure { 1 })
fold([], [], fn(total: list<int>, value: int) -> list<int> effect pure { total })
```

编译器的类型探测使用 checkpoint/rollback，不改变最终 capture 顺序。可执行 Core 仍按 authored
operand 顺序执行：先 iterable；fold 再执行 initial；最后构造 callback。每个 operand 只执行一次。

## `for` 的词法语义

`for name in iterable { body }` 先在外层 scope 求值 iterable，再仅在 body 内绑定 immutable
`name`。binder 会遮蔽同名 builtin、静态函数或外层 local。body 可以继续声明 local、调用 private
helper，并捕获普通 lexical value。

`for` 是立即执行，不创建可逃逸的用户 closure。lowering 会生成带 non-escaping provenance 的
callback；它因此可以调用或捕获外层 function-valued local。Core verifier 同时证明该 SSA value
只被同一个 collection instruction 消费，不能 Return、普通 Invoke、写入结构值或穿过 block edge。
显式 closure 仍不能捕获 function value。

aggregate metadata 会沿 collection element、tuple position、map entry 与 nominal field 保留嵌套
callable contract。function value 可以作为 iterable element、`map`/`for` result 或 `fold`
accumulator；Core verifier 会重算 projection metadata，并拒绝缺失、伪造或与 operand 不一致的 contract。

## 确定性与预算

collection operation 在分配或迭代前计算完整有限 count，并一次原子 reserve aggregate iteration、
collection element 和 logical byte 三个维度。任一维度超限时，不分配 collection，也不提交部分
ledger。range descriptor 本身保持 lazy；只有消费它的 operation 扣除迭代预算。

Graph-emitting callback 是 verified Core，并与调用方共用唯一 graph transaction。`map`/`for`
在第一个 callback 前一次性 reserve iteration 和输出集合预算；每个 domain operation 仍在修改
arena 前原子 reserve 自己的 entity 和 emitted-byte 增量。callback、后续指令、预算、重复 key、
single ownership 或 freeze 任一失败都会 taint 整个 transaction；失败执行不能发布部分图。

所有实体 key 都必须由 constructor 显式提供，迭代不会合成 identity。collection shape、`filter`
predicate，以及 iterable cardinality、实体存在性、key、parent、ordering 和 downstream topology
sink 都必须至多为 Build stage。Temporal value 只能作为不影响 shape、key、ordering、identity、
resource selection 或 topology 的 leaf 留存。预算上限见
[Programming Resource Budget](programming-limits.md)。
