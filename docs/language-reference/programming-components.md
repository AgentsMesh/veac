# 组件与复用

VEAC 的静态组件是 typed value construction pattern，不是生成 `.veac` 文本的 macro。组件由
module、nominal configuration、factory function 和 immutable method 组成，最终返回 `Item`、
`Layer`、`Sequence`、`Delivery` 等闭合 Domain value。

## 组件定义

```veac,fragment
export struct Card {
  key: identifier,
  at: time,
  duration: time,
  topic: text,
  detail: text,
  mood: Mood,
}

impl Card @presentation {
  export fn title(self, separator: text) -> text {
    self.topic + separator + self.detail
  }

  export fn background(self) -> color {
    match self.mood {
      Mood.Calm => #2b6574ff,
      Mood.Accent { color } => color,
    }
  }
}
```

`Card` 是可比较、可存入 collection、可由函数返回的普通 nominal value。`title` 和 `background`
通过静态 receiver type 解析，不依赖 reflection、prototype 或 runtime method table。配置与产物
分离，允许 Agent 先编辑意图，再在 `main` 中明确构造 graph。

## 模块复用

```veac,fragment
export fn backdrop(card: Card) -> Item {
  item(
    card.key, item_enabled(), during(card.at, card.duration),
    source_generated(generator_solid(card.background())), source_timing_native()
  )
}
```

完整定义位于 `examples/programming-language/brand.veac`，并由文档合同测试通过生产 frontend 构建。
importer 选择 lexical alias，例如 `brand.first_card(2500ms)`。只有 `export` declaration
跨 module boundary；private helper 仍可被 exported method 调用。loader source ID 是 canonical
module identity，同一模块通过多个 alias 导入时共享 verified function registry，不复制源码。

## 批量实例化

组件可以作为 list/map/for 的输入：

```veac,fragment
let titles = map(cards(duration, accent_color), fn(card: brand.Card) -> text effect pure {
  card.title("：")
});
let summary = fold(titles, "", fn(total: text, value: text) -> text effect pure {
  if total == "" { value } else { total + " / " + value }
});
```

`map`/`for` 可以执行有界 `GraphEmit` callback；`filter`/`fold` callback 必须是 `Pure`。所有实例
共享 execution budget，entity key 仍由源码显式提供。重复 key、cross-graph
handle、double ownership 或 topology 超预算会使整个 transaction 回滚。

## 可复用动画

factory 与 method 可以用 attachment expression 把动画绑定到相对 owner，而不写 root absolute path：

```veac,fragment
export fn animated(card: Card) -> Item {
  let visual = backdrop(card);
  animate visual-opacity on clip(visual) {
    sample_curve_ease_in_out(
      clamp(progress * 1.5, 0.0, 1.0), [(0.0, 0.0), (1.0, 1.0)]
    )
  }
}
```

`animate property on target(owner, selectors...) { body }` 是 Build-stage `GraphEmit` expression，返回
原 owner。body 单独编译为 verified `Pure` closure；Item closure 的隐式参数是
`sequence_time`、`clip_time`、`frame`、`progress`、`source_time`，Apply closure 只有
`sequence_time` 与 `frame`。closure 可以捕获 immutable Build 值和调用 module-private/imported
Pure helper，不能改变 graph topology。

attachment 在 graph transaction 中保存 typed owner 与 selector，owner 进入最终 Project 后才解析
absolute logical sink。因此一个 module factory 可实例化任意多个 key，无需拼路径；重复 sink、stale 或
cross-graph handle、已被 owner 消费的 handle、预算耗尽都会 taint 整个 transaction。source-index 以
function/method 内稳定源码顺序发布 ordinal，source-edit 可直接替换原 attachment declaration 或 body，
不会生成、格式化或重新解析另一份 `.veac`。

## 参数与 Slot

组件参数就是函数参数或 nominal field，default 由普通 pure function 表达。可替换媒体或文本不是
语言宏 slot，而是组件构造出的 typed canonical template contract：`slot_video()`、`slot_audio()`、
`slot_visual()`、`slot_text()`、`slot_caption()` 或 `slot_sequence()`。slot kind、fill mode、material
约束和 editable text 状态均为闭合值，并由 `veac-template` 原子填充。

这种设计保留组件化、复用和 typed slot，同时避免 property bag、字符串字段反射、卫生化 ID
拼接、生成源码后重解析，以及第二套 component execution semantics。
