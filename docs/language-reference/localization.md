# 多语言项目

VEAC 把“选择哪一个交付语言”和“本地化内容如何组织”分成两个正交合同：

- locale 是 root entry 的显式 Build input，参与 executable identity；
- 文案、字体、布局和素材 variant 是普通 module、enum、struct 与纯函数；
- graph build 结束时，canonical IR 已经是一个确定 locale 的闭合项目。

这套模型不增加 `locale`、`catalog` 或 `translate` 关键字，也不在 evaluator 中嵌入翻译服务。

## 闭合 locale 选择

项目应声明自己的可交付 locale 集合，而不是把任意 host 字符串带入 graph：

```veac,fragment
// localization.veac
module {
  export enum Locale { en, ar, zh-Hans, }

  export struct Copy {
    title: text,
    detail: text,
  }

  export fn copy(locale: Locale) -> Copy {
    match locale {
      Locale.en => Copy { title: "Make it yours", detail: "150+ widgets", },
      Locale.ar => Copy { title: "العب مباشرة على", detail: "شاشتك الرئيسية", },
      Locale.zh-Hans => Copy { title: "随心定制", detail: "150+ 桌面小组件", },
    }
  }
}
```

VEAC declaration name 原生允许 kebab-case，因此常见 BCP-47 tag 可以直接作为 variant。优先使用
`zh-Hans`、`pt-BR` 这类交付 spelling，不要先改成 `zh_Hans`、`pt_BR` 再在 host 层反向映射。

root entry 导入该 enum，并将它声明为 parameter：

```veac,fragment
import "./localization.veac" as l10n;

input parameter locale: l10n.Locale;

fn main(context: Context) -> Project {
  let copy = l10n.copy(locale);
  project(identifier("localized-video"), project_settings(600))
}
```

Build input 只接受 payloadless nominal enum。带字段 variant、struct、未知 variant、缺失 binding 和
类型不一致都会在 graph transaction 发布前失败。`match` 必须穷尽，因此添加 enum variant 后，所有
catalog resolver 都必须同步更新才能重新通过编译。

## CLI 输入

交互式或 Makefile 构建优先使用可重复的直接输入：

```bash
veac check main.veac --input locale=zh-Hans
veac build main.veac --input locale=zh-Hans --emit-ir build/zh-Hans/project.json
```

CLI 先解析 source graph，再按已声明的 expected type 解码 `NAME=VALUE`。它不根据字面量猜类型；
第一个 `=` 分隔名字和值，因此 text 值可以继续包含 `=`。重复 inline 名、未知名和非法值都会失败。

自动化系统仍可提供 versioned `--inputs` manifest。两者同时出现时，manifest 是 base，inline binding
按名字显式覆盖；最终 typed binding 按 canonical 名字排序，来源和命令行顺序不改变 input digest。

Agent 或构建系统可以在不提供 locale 值时先发现完整接口：

```bash
veac source-index main.veac | jq '.build_inputs[] | select(.name == "locale")'
```

`source-index` 会给出 role、nominal TypeId、definition digest 和合法 enum variants，避免在 VEAC enum
之外再维护一份无法校验的 locale 清单。

## Catalog 组件

推荐让每个 locale module 返回同一个 nominal 组件：

```veac,fragment
export struct LocaleProfile {
  copy: Copy,
  primary_font: Resource,
  fallback_fonts: list<Resource>,
  title_size: length,
  caption_width: length,
}
```

项目可以把以下差异放进 profile，而不复制时间线：

- 已审核文案和稳定 message identifier；
- 字体资源、fallback 顺序、字号、行高和文字框尺寸；
- 本地化图片、视频、配音和字幕 Resource；
- RTL/LTR 对应的逻辑布局参数；
- locale 特定时长、语速、镜头选择和 delivery 文件名。

资源 identity 必须在 locale module 中固定 SHA-256。不要把 locale 拼进路径后复用错误的 digest，也不要
把绝对素材目录写入 source。素材位置继续由 canonical consumer 的 `--material-root` 提供。

## 执行语义

locale input 是 Build-stage value，可以决定普通值、资源选择和 graph topology。它不能成为未绑定的
ambient 环境变量。一次执行只选择一个 locale，并产生一个 canonical project：

```text
source graph + typed locale input
  -> exhaustive catalog selection
  -> one verified graph transaction
  -> one locale-resolved canonical IR
  -> plan / package / render
```

未选择分支不会发射资源或 item；IR 不应包含其他 locale 的 URI、host manifest 路径或素材根绝对路径。
同一 source revision 和同一 typed locale 必须 byte-stable；不同 locale 必须得到不同的 declared input
digest 和 executable identity。

## 构建矩阵

VEAC CLI 的一次 build 有意只产生一个项目。多 locale 批量交付属于 host orchestration：

```make
LOCALES := en ar zh-Hans

build-locales:
	@for locale in $(LOCALES); do \
	  $(MAKE) build LOCALE=$$locale || exit; \
	done
```

建议输出按 locale 隔离：

```text
build/locales/<locale>/runtime/
output/locales/<locale>/
audit/locales/<locale>/
```

普通 CI 应静态构建全部 locale，但只渲染 Latin、CJK、RTL 等代表 variant。全量重型 FFmpeg 交付放在
手动或定时发布任务中。

## 原生文本

现有 `FontStack`、Unicode shaping、fallback、双向排序和 grapheme segmentation 可以渲染多脚本文本。
项目仍需显式选择字体资源、fallback 与 layout；不要依赖 host 字体或 CWD。将翻译预先烘焙为图片只能
验证 locale resource selection，不能替代原生文本排版测试。

后续跨项目通用能力应进入版本化 stdlib/IR，而不是新语法：

- canonical BCP-47 `LanguageTag` value；
- `text_align_start()` / `text_align_end()` 与显式 base direction；
- Text、Caption、Audio 和 Delivery 上的 language metadata；
- CLDR plural category、number/date formatting 等 pure operation。

翻译、ASR、TTS 和 dubbing 仍属于外部 provider。provider 应返回可审阅 proposal，并通过
SourceEditBatch 修改 locale module 的 source of truth；不能在 render 时联网或静默改写 canonical IR。
