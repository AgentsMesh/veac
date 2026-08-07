# VEAC 编程模型

VEAC 在一个受限 source graph 中提供 module、强类型函数、closure、immutable collection、nominal
value 和静态 method。生产语言只有一条 executable pipeline：

```text
Surface -> typed HIR -> verified Core v10 -> bounded evaluator
        -> frozen graph + temporal residualization -> canonical ProjectEnvelope
```

入口必须提供 root-local `fn main(context: Context) -> Project`。完整执行、graph transaction 与直接
lowering 合同见[可执行 Build](executable-build.md)。模块只提供可导入声明，不能代替入口。
需要由 host 提供的数据必须使用强类型 [`input` 声明](build-inputs.md)，不能读取环境变量、文件系统、
网络、时钟或随机数，也不能把 `Context` 当作字符串 property bag。

[`programming-language/main.veac`](../../examples/programming-language/main.veac) 与
[`brand.veac`](../../examples/programming-language/brand.veac) 展示跨模块 struct/enum、method、
closure、`GraphEmit` map 和直接 Project construction；source edit batch 从 `.veac` source of truth
按稳定语义路径替换完整局部 statement，并重建 canonical IR。

## 模块

imported file 使用匿名 `module {}` kind marker，不声明第二份 module 名字：

```veac,fragment
// timing.veac
module {
  fn twice(value: time) -> time { value * 2 }

  export fn intro(value: time) -> time {
    twice(value) + 250ms
  }
}
```

调用方以 `import "./timing.veac" as timing;` 选择 local alias，并通过
`timing.intro(1s)` 访问 exported declaration。canonical module identity 是 loader 提供的
root-relative source ID；private helper 不跨 module boundary，但 exported function/method 可以调用它。

import 必须相对 source root 且无环。absolute path、`..`、symlink、root escape、重复 alias、把 entry
当 module 和 import cycle 都会失败。canonical source ID 是不超过 4,096 bytes 的 UTF-8 `/` 路径；
empty/`.`/`..` segment、`:`、`\`、control character 和 `.veac-source.lock` 不合法。filesystem
loader 只接受 regular file，并拒绝不同 lexical ID 指向同一物理文件。

## 函数、Block 与 Closure

```veac,fragment
fn pad(base: time, extra: time) -> time { base + extra }

fn intro(value: time) -> time {
  var padded: time = pad(value, 250ms);
  if padded < 2s { set padded = 2s; padded } else { padded }
}

fn apply(value: int, operation: fn(int) -> int effect pure) -> int {
  operation(value)
}
```

signature 固定为 `fn name(param: type, ...) -> type { ... }`。参数不可变；block 由零个或多个以
分号结束的 `let`、`var`、`set` statement 和一个无分号 tail expression 组成，tail 决定返回值。
`let` 不可变，`var` 为每次 function/closure activation 创建独立 typed slot，`set` 只能以 exact
type 更新已声明的 `var`。mutable local 不能保存 function value、被 closure capture 或跨 activation
逃逸；失败的 activation 丢弃 slot state，并与 graph transaction 一起 fail closed。`if` 必须带
`else`，两支都 type-check，但只执行选中分支。argument 从左到右各执行一次，无 truthiness 或
implicit cast。

typed closure 写作 `fn(value: int) -> int effect pure { value + offset }`。closure 必须显式声明
`pure`、`local`、`emit` 或 `any` effect contract；编译器仍根据 Core evidence 复核声明，不能用宽松或
伪造的 contract 隐藏实际副作用。它只能 capture 外层 immutable local、
parameter 或 capture；不能 capture ambient external、function declaration、graph builder 或 temporal
topology control。capture 顺序由第一次 resolved use 固定。任意 expression 都能 postfix call，
function/method/closure 最终都以 resolved ID 进入 Core，不在 runtime 按字符串查找。

`pure` 只接收 `Pure`；`local` 接收 `Pure | LocalMutation`；`emit` 接收不含 local mutation 的
`Pure | GraphEmit`；`any` 接收全部 evidence。`filter`/`fold` 要求实际 `Pure`，`map`/`for` 还可
接收不含 `LocalMutation` 的 `GraphEmit`。effect contract 属于 function type identity，但 verifier
始终以 body 和 callable metadata 的实际 evidence 为准。`LocalMutation` 不进入 Temporal residualization。

完整 call graph 支持 forward call，unused body 也必须 parse、resolve、type/effect-check 和 verify。
direct/indirect recursion 均拒绝；call depth、node、value 与 allocation 共享 source-graph budget。

## 类型与值

十种 primitive type 为：

```text
int | scalar | time | length | percent | angle | text | color | bool | identifier
```

结构 type 为 `list<T>`、`range<int>`、`map<text, V>`、`map<identifier, V>`、至少二元的 tuple 和
`fn(T, ...) -> R effect E`。function type 的 effect 是类型 identity 的一部分，嵌套 function type
逐层携带 contract。用户可以声明 module-qualified `struct` 与 closed `enum`。没有 `number`、
`string`、`boolean` 或 `resource` alias，也没有 implicit numeric widening。Domain type 是版本化
standard-library symbol，不属于 primitive spelling 或 lexer keyword。

```veac,fragment
struct Card { title: text, duration: time, }

enum Placement {
  Center,
  Corner { x: length, y: length, },
}

const Card card = Card { duration: 2s, title: "片头", };
```

struct initializer 使用 named field，且 missing、unknown、duplicate 或 wrong-type field 都失败。
closed enum 通过 `match` 穷尽处理；`impl Type` method 是 immutable、nominal、static dispatch，没有
inheritance、prototype、reflection 或 runtime method table。详见
[Nominal value 与 method](programming-nominal.md)。

list/map/tuple literal 分别写作 `[1, 2]`、`#{"intro": 0s}`、`(1920, 1080)`。空 list/map 必须从
expected type 推导。map key 按原始 UTF-8 bytes canonical 排序；结构 equality 递归且类型精确，
function 与 Domain handle 不支持 equality。

## 纯表达式与数值

```veac,fragment
const time duration = clamp(base + 500ms, 2s, 3s);
const time base = 2500ms;
const text title = "同一组件" + "，再次实例化";
```

constant 可 forward reference，dependency cycle 会失败。无后缀 integer 是 checked signed i64
`int`，无后缀 decimal 是 exact `scalar`，unit literal 保持对应 dimension；`1`、`1.0` 与 `1s`
类型不同。算术、comparison、equality、`min`/`max`/`clamp` 要求合同允许的 exact type，不会静默
round。constant 是 module-static reusable declaration，可被 function、method、`main` 和 temporal body
引用；resolver 会闭合穿过 pure function 的 constant dependency 并拒绝间接 cycle，随后把值作为 literal
写入最终 verified Core。所有值直接进入 typed Core，不会先格式化成源码再重解析。

semantic name 为 1 到 128 个 ASCII bytes，匹配
`[A-Za-z_][A-Za-z0-9_]*(-[A-Za-z0-9_]+)*`；`true`/`false` 是 reserved literal。构造
identifier value 使用 `identifier("cover-art")`。因为 `-` 可属于 name，subtraction 写成
`duration - offset`。quoted text 支持 Unicode，evaluated text 上限是 1 MiB。

程序不能读取 ambient filesystem、environment、clock、network、process 或 implicit random state。
所有失败都是 source-located diagnostic，并且不能发布 partial IR。

## 集合与有界迭代

range 写作 `start .. end` 或 `start .. end by step`，类型固定为 `range<int>`，是 finite、lazy、
half-open sequence。zero step 和 count/overflow 会失败，方向背离产生 empty range。

`map`、`filter`、`fold` 与 `for value in iterable { body }` 接受 list/range/map；map element 是
`(key, value)` tuple。它们按 canonical order 单次、顺序执行并共享 iteration/collection budget。
executable topology 中 `map`/`for` callback 可为 `GraphEmit`；`filter`/`fold` callback 必须 `Pure`。
精确 effect、empty inference 与预算见[有界集合与迭代](programming-collections.md)。

## Typed Component Pattern

静态组件由 module、typed factory function、nominal configuration value 和 immutable method 组合，
合同见[组件与复用](programming-components.md)。组件返回普通 Domain value，并在调用点通过明确
owner method 接入 graph。factory/method 内可用 owner-relative `animate` attachment 复用 Pure 动画
closure，freeze 后解析为 canonical absolute sink；没有 source injection、宏展开、字符串 path、隐式
ID 拼接或独立 component runtime。

当前 opset v7 的 214 个 DomainType 与 581 个 operation 覆盖公开编辑机制。所有 public path 仍受
[编程资源预算](programming-limits.md)约束。
