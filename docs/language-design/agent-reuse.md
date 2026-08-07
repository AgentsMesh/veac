# Agent 组件与复用指南

VEAC 的组件系统就是语言本身：module 发布 API，struct/enum 保存不可变配置，Pure function 计算值，
method 或 factory 构造闭合 Domain graph。没有并行的 preset/component macro，也不会生成 `.veac`
文本再解析。

## 发布模块 API

```veac,fragment
module {
  export enum Mood {
    Calm,
    Accent { color: color, },
  }

  export struct Card {
    key: identifier,
    at: time,
    duration: time,
    title: text,
    mood: Mood,
  }

  impl Card @presentation {
    export fn background(self) -> color {
      match self.mood {
        Mood.Calm => #2b6574ff,
        Mood.Accent { color } => color,
      }
    }
  }

  export fn backdrop(card: Card) -> Item {
    item(
      card.key, item_enabled(), during(card.at, card.duration),
      source_generated(generator_solid(card.background())), source_timing_native()
    )
  }
}
```

importer 用 source-relative path 和 lexical alias：

```veac,fragment
import "./brand.veac" as brand;

let card = brand.Card {
  key: identifier("opening"), at: 0s, duration: 3s,
  title: "开场", mood: brand.Mood.Accent { color: #7b315dff, },
};
let opening = brand.backdrop(card);
```

只有 `export` declaration 跨 module boundary。私有 helper 可以被 exported function/method 调用；
imported `main` 不能成为入口。loader source ID 是 canonical module identity，同一模块通过多个 alias
导入时共享 verified definition，不复制源码或 runtime state。

## 选择合适的复用原语

| 需要复用的内容 | 原语 |
| --- | --- |
| 纯常量或计算 | typed `const` / Pure function |
| 一组具名配置 | `struct` |
| 闭合分支 | `enum` + exhaustive `match` |
| 配置关联行为 | `impl` method |
| Item/Layer/Sequence/Delivery 子图 | 返回 Domain value 的 factory/method |
| 多文件 API | `module` + `export` + `import ... as ...` |
| 批量构造 | bounded `map` / `for` / `fold` |
| 随时间变化的叶值 | root `animate` + Pure helper |

函数参数和 nominal field 就是组件参数。default 用普通 Pure function 表达。每次构造仍显式传入
`identifier`，因此 Agent 能从源码看见稳定 identity；compiler 用 typed owner path 派生 canonical ID，
不会卫生化拼接隐藏字符串。

## 批量构造

```veac,executable
struct BatchCard { key: identifier, at: time, fill: color, }

fn render_card(card: BatchCard) -> Item {
  item(
    card.key, item_enabled(), during(card.at, 1s),
    source_generated(generator_solid(card.fill)), source_timing_native()
  )
}

fn track_state_default() -> TrackState {
  track_state(
    track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked()
  )
}

fn main(context: Context) -> Project {
  let cards = [
    BatchCard { key: identifier("first"), at: 0s, fill: #244f73ff, },
    BatchCard { key: identifier("second"), at: 1s, fill: #2f7d69ff, }
  ];
  let mapped = map(cards, fn(card: BatchCard) -> Item effect emit {
    render_card(card)
  });
  let iterated = for card in [
    BatchCard { key: identifier("third"), at: 2s, fill: #b64d58ff, }
  ] { render_card(card) };
  let layer = visual_layer(
    identifier("cards"), 0, placement_free(),
    track_state_default(), track_routing_default()
  ).with_items(mapped).with_items(iterated);
  let timeline = sequence(
    identifier("main"), "批量组件",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
  ).with_layers([layer]);
  project(identifier("batch-components"), project_settings(600))
    .with_sequences([timeline]).entry(timeline)
}
```

collection、iteration、call depth、emitted entity 和 retained value 共用一个 execution ledger。进入
module、function、closure 或下一次 iteration 都不能重置预算。重复 key、cross-graph handle、double
ownership、effect/stage 违规或超预算会回滚整个 graph transaction。plural owner method 会先验证完整
Domain list，再按输入顺序一次接入；它不会通过可变 parent 或 stale handle 放松静态 topology。

## Slot 与可编辑内容

可替换媒体或文本不是 macro slot，而是 factory 构造出的 typed canonical template contract：
`slot_video()`、`slot_audio()`、`slot_visual()`、`slot_text()`、`slot_caption()` 或 `slot_sequence()`。
slot kind、fill mode、material 约束和 editable-text 状态都是闭合值，并由 `veac-template` 原子填充。

这种边界保留组件化、复用和模板填充，同时排除 property bag、字符串字段反射、generated-source
reparse 和第二套 component execution semantics。完整可执行示例见
[`examples/programming-language`](../../examples/programming-language/main.veac)。
