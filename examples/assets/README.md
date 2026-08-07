# 可执行示例素材

`veac-example-zh.ttf` 是为可执行示例裁剪的静态中文字体夹具。源字体为 Google Fonts 中的
Noto Sans SC 可变字体，先固定到 `wght=400`，再只保留示例画面使用的字符。字体及其派生文件
遵循同目录的 [SIL Open Font License 1.1](OFL-NotoSansSC.txt)。

源文件：

```text
https://github.com/google/fonts/blob/main/ofl/notosanssc/NotoSansSC%5Bwght%5D.ttf
```

各 executable example 在自己的 `assets/` 下持有相同字体 blob，使 canonical project-relative URI
不依赖父目录穿越。Git 对相同 blob 去重。固定摘要为：

```text
64afada094cf447d71fad4ee726799c4fa74917f7d9a1113ed7ea3c4234ff774
```

`executable-audio-caption/assets/tone.wav` 是 48 kHz、单声道、16-bit PCM、660 Hz、2 秒的确定性
测试信号，固定摘要为：

```text
5c3aaa006e4c341feddf589f9eb0ba3b215c60353e491abf8f06f4bee4478cfc
```

这些文件只用于验证资源身份、音频时序和字幕字体绑定，不构成 VEAC 内置素材库。
