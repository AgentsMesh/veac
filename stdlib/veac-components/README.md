# VEAC 标准组件包

这是第一版可执行的标准组件包切片。它把常见的标题卡片、文本插槽和预览交付收敛为
名义配置、纯方法、带默认值的 typed factory，以及明确的 owner method。包内 `.veac` 是
唯一源码；`veac.package.json`、`veac.package.lock` 和 `veac.package.api.json` 只是可验证的
发布合同，不能反向生成源码。

## 能力

- `CardConfig`：稳定 key、时间范围、文案和闭合色调；`label`、`background` 是纯方法。
- `card`、`title`：带默认时长的 factory，调用方可以使用命名参数覆盖 slot。
- `card_item`、`title_item`：返回闭合 `Item`，通过 typed source 和 visual/template 方法接入。
- `preview`：把时间线、画布和帧率绑定为单一预览交付。

包入口是 `main.veac`。`components/`、`layout.veac`、`text.veac`、`motion.veac`、`media.veac`
和 `delivery.veac` 保持单一职责；它们均由锁文件固定并经过 compiler-derived API 校验。

```bash
veac package inspect stdlib/veac-components
veac package api stdlib/veac-components
```
