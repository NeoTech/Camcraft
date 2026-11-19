# 翻译 CamCraft 界面（contributing translations）

界面语言在 `assets/i18n/<tag>.toml` 里。你**不需要安装 Rust**，改文本即可。

## 格式

```toml
# 注释行以 # 开头
"Save" = "保存"
"Save As" = "另存为"
```

- 左边（key）是代码里写死的**英文原文**，必须一字不差，包括大小写、标点、空格、省略号。
- 右边（value）是翻译。允许 `\"`、`\\`、`\n`、`\t` 四种转义。
- 一行一条；空行和注释会被忽略；任何格式错误都会在启动时被**按行号点名**（不会静默跳过）。
- 同一个 key 出现两次会报错，后出现的生效。

## 规则

1. **只翻界面**。G-code、CL、项目文件、机床 `.mch`、后处理 `.cps`、MCP 工具名、日志、测试 golden 一律不翻（#853）。
2. **术语统一**：`stock` 毛坯 · `setup` 装夹 · `toolpath` 刀路 · `stepdown` 每层切深 · `stepover` 步距 · `stock to leave` 留量 · `WCS` 工件坐标系 · `post` 后处理。
3. **简短优先**：ribbon 单元格在最小窗口下只有约 40 px 宽（约 3 个汉字）。
4. 数字、单位符号、`Ø`、`G54` 之类保持原样。

## 检查

```sh
python3 scripts/i18n/check.py assets/i18n/zh-CN.toml
```

它会报告：格式错误（带行号）、重复 key、空翻译、翻译与原文完全相同（`2D`/`3D` 这类允许）、以及行尾多余空格。
