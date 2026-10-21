# slint-math-input

[English](README.md)

基于 Rust 和 Slint 的原生公式输入组件，目标是在排版后的公式中直接编辑，并通过 LaTeX 导入、导出数学内容。

## 当前状态

**当前为源码预览版 `0.1.0-preview.3`，尚未发布到 crates.io。** 已实现数学编辑核心、共享布局与命中定位、Slint 输入表面和原生演示；继续保留 `publish = false`，实现状态与验证状态分别记录。

已实现：

- 分数、根式、上下标、下方居中的极限条件、导数和偏导数、组合数、伸缩定界符、重音和上下方注记、数学字母样式，以及有界/无界大算子、矩阵、多行对齐和分段式。
- 键盘与鼠标定位、跨结构选区、替换、边界删除、剪贴板交换与撤销重做。
- 保留半成品、光标和选区的版本化 JSON 草稿，校验恢复及原子文件保存。
- 明确子集内的 LaTeX 导入导出；严格导出会报告空位和未完成括号。
- 可嵌入的 `.slint` 组件和独立 Rust 控制器，接入原生文本输入、焦点及滚动。
- 可复用的鼠标数学键盘：数字、字母与可滚动的函数/公式目录，支持结构插入、选区导航、剪贴板、撤销重做与收起。
- 默认组合好的 `MathEditor`，同时包含公式输入区和数学键盘；完整窗体模板使用相同接口，宿主可替换窗体形态或自行排列两部分。

SMI 负责数学编辑、与编辑一致的显示和命中定位，以及内容交换。宿主负责窗口、应用布局、保存策略和提交后的业务处理。

能力、Desmos 依据、自动验证和剩余检查见[支持表](docs/support.md)，宿主接入见[集成说明](docs/embedding.md)。

本项目是独立维护的 Slint 第三方组件。

## 开发

需要 Rust 1.92 或更新版本。默认 `desktop` 特性使用 Slint 1.18.1、Winit 和软件渲染；Windows 需要 MSVC 工具与 Windows SDK。宿主可关闭默认特性，仅启用 `slint-ui` 并选择自己的 Slint 后端；关闭全部界面特性可单独使用核心。平台准备参考 [Slint 官方说明](https://docs.slint.dev/latest/docs/slint/guide/getting-started/)。

```sh
cargo run --locked --example demo
cargo test --locked --no-default-features
cargo check --locked --no-default-features --features slint-ui --lib
```

演示默认显示数学键盘，可切换“123 / ABC / 函数”。“选择”配合方向键扩展选区，“退出”返回上一层结构；点击按键后可接着用实体键盘输入。
函数页使用固定分类栏，可直接切换结构、常用函数、微积分、三角、统计概率、矩阵、希腊字母、关系集合和符号常数；每次只显示一个分类，长分类内部滚动，分类栏和底部导航始终可用。界面移除字体样式、下划线及重复函数别名；核心仍支持全部 319 个排版入口，供程序输入和 LaTeX 交换使用。
全部内置入口见[公式输入目录](docs/catalog.md)。

完整方案从[默认窗体模板](templates/default-window.slint)和 [Rust 接线示例](examples/support/mod.rs)开始。自定义窗口可从 `@math-input` 导入 `MathEditor`；也可分别使用 `MathInput` 与 `MathKeypad`。

“保存草稿”将草稿保存到 `local_artifacts/demo-draft.json`，再次启动时恢复。“LaTeX / 草稿”展开内容面板，可导入 LaTeX 或恢复粘贴的 JSON 草稿。

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo doc --locked --no-deps
```

贡献与验证方式见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可证

采用 **MIT OR Apache-2.0** 双许可，使用者可选择 [MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE)。
STIX Two Math 字体采用单独的 [SIL OFL 许可](assets/OFL-STIX.txt)，[字体来源](assets/README.md)与依赖许可证分别保留。
