# Nominal Value 与静态 Method

VEAC 的 `struct` 和 closed `enum` 是不可变、强类型的程序值，不是视频属性平铺，也不是另一套
JSON object 语法。它们用于封装算法输入、品牌配置、剪辑决策和可复用结果；Project graph 仍由
后续 domain value 与 `GraphEmit` 原语构造。

## 声明与构造

字段和 variant 的声明顺序就是 canonical layout 顺序：

```veac,fragment
export struct Brand {
  title: text,
  accent: color,
}

export enum Placement {
  Center,
  Corner { x: length, y: length, },
}
```

类型声明可前向引用。TypeId 只由 canonical source ID 与声明名决定；字段变化由独立
`TypeDefinitionDigest` 检测。通过 list、map、tuple、range 或 function type 间接形成的递归 layout 同样会被拒绝。
空 enum、重复字段、重复 variant、unknown type 和 exported API 泄漏 private type 都是编译错误。

构造使用命名字段，initializer 按源码顺序各执行一次，存储再重排为声明顺序：

```veac,fragment
const Brand brand = Brand {
  accent: #32d583ff,
  title: "片头",
};

const Placement placement = Placement.Corner {
  y: 32px,
  x: 24px,
};
```

missing、unknown、duplicate 或错误类型的字段不会被默认或忽略。unit variant 写作
`Placement.Center`，payload variant 必须完整构造。

## Projection 与 Namespace

`.` 是真实 postfix member 运算符：

```veac,fragment
brand.accent
make_brand().title
theme.default_brand.accent
```

lexical binding 的首段优先于同名 module alias。否则 resolver 选择最长的可见 qualified type、
value 或 function prefix，再从左到右执行 field projection 或 method resolution。Core 只保存
TypeId、FieldIndex 和 FunctionId，不按源码字符串查找成员。

## Exhaustive Match

`match` 只接受 closed enum，并要求覆盖每个 variant：

```veac,fragment
fn inset(value: Placement) -> length {
  match value {
    Placement.Center => 0px,
    Placement.Corner { x, y: vertical } => x + vertical,
  }
}
```

payload field 可用 shorthand 绑定，也可通过 `field: binding` 改名。binding 仅在对应 arm 内可见。
最后一个 `_` 可覆盖所有剩余 variant；它会 lower 成明确的 Core arms。重复 variant、另一 enum 的
variant、字段不完整、wildcard 不在末尾和 non-exhaustive match 都会在执行前失败。

每个 arm 都会 resolve、type-check 和 verify，但运行时只执行选中的 arm。所有 arm 返回类型必须
完全一致，未选中 arm 的除零或预算失败不会发生。

## 静态 Method

method 只能扩展同一 source 内声明的 nominal type：

```veac,fragment
impl Brand @presentation {
  export fn heading(self, suffix: text) -> text {
    self.title + suffix
  }

  fn debug_color(self) -> color { self.accent }
}
```

`self` 是不可变 receiver，也是 Core 参数 0。调用先执行 receiver，再从左到右执行显式参数：

```veac,fragment
brand.heading(" / 精华版")
```

`export fn` method 可从 importing module 调用；plain `fn` method 保持 private。exported method
仍可调用定义 module 内的 private method 和 helper function。禁止 foreign extension、inheritance、
runtime dispatch、reflection、method table mutation 和隐式 receiver conversion。

编译器先预注册所有 function 与 method signature，再把全部 body lower 成 resolved Typed HIR。
call graph 从 HIR 中的 FunctionId 构建；普通 function 和 method identity 都固定 Core version，
因此 function-to-method、method-to-function、method-to-method
和嵌套 control flow 使用同一 SCC 与 depth 检查。通过 Surface token 猜 method call graph 不是语义。

## Core、预算与安全

Core v10 发布完整、按 TypeId 排序的 nominal definition table，并提供 `StructConstruct`、
`StructProject`、`EnumConstruct` 与 exhaustive `Match`。verifier 独立检查 definition digest、field/
variant index、payload block ABI、dominance、exhaustiveness、metadata 与 registry closure。
v7 同时固定 domain opset version 与 registry digest；closure body 和直接调用的 Core body 必须与
调用方使用完全相同的 domain ABI identity，runtime 也会在执行前重新核对。

nominal container 与每个 field handle 在分配前执行 atomic reservation。nested value、closure capture、
collection 和 trusted callable input 都会递归核对同一 TypeRegistry；把 function 包进 struct/enum
不能绕过 equality、collection、escape 或 callable admission 规则。

nominal construct 与 projection 是 `Pure`。固定 layout 是 Const shape，payload 贡献 leaf stage；enum
discriminant 属于 shape。Temporal discriminant 不能控制集合 shape、graph topology、identity、order、
resource selection 或其他 Build sink。

## Source-Of-Truth Edit

`source-index` 为 method 发布独立 target：

```json
{
  "module": "brand.veac",
  "path": { "kind": "method", "receiver": "Brand", "method": "heading" }
}
```

`MethodBody` source site 使用 closed `method_body` `BodySite`，JSON 为
`{ "type": "method_body" }`。`body_equals` 与 `set_body` 操作完整
block，事务会重新 parse、resolve、type-check、执行并验证 canonical IR；失败时不改任何源码。

nominal declaration 也有独立、module-qualified target：`struct`、`struct_field`、`enum`、
`enum_variant` 与 `enum_variant_field` 分别保留声明 owner，不以 byte offset 或 registry ordinal
作为身份。source-index v8 在节点的 `declarations` 中给出 nominal member 的精确源码、range 与 closed
`DeclarationSite`，并在 module inventory 发布完整 struct、enum 与 impl 顶层声明。

source-edit v6 的 `declaration_equals`/`set_declaration` 支持 nominal member；
`top_level_declaration_equals`/`set_top_level_declaration` 与 insert/remove declaration 支持完整 struct、
enum 和 impl block。member range 不包含分隔逗号。重命名后，Agent 可在同一个 atomic multi-module
batch 中更新 import、callable 与 constant use site。未覆盖的 constructor、projection、match pattern
或 import use 会在全图重新 parse、type-check、执行、lower 时使整个 preview 失败，所有原始 `.veac`
文件保持不变。每个 impl block 必须声明 `@identity`，完整 block 以 module + receiver + identity 唯一
寻址；method target 仍是 module + receiver + method，因此 method 在 block 之间移动不会改变身份。
