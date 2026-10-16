# slint-math-input

[English](README.md)

基于 Rust 和 Slint 的原生公式输入组件，目标是在排版后的公式中直接编辑，并通过 LaTeX 导入、导出数学内容。

## 当前状态

**项目处于初始化阶段，尚无可用的公式编辑 API，也尚未发布到 crates.io。** 当前提供库骨架、许可证、贡献说明与 CI；形成可用的首版实现后，再开放包发布。

计划支持：

- 数学表达式的结构化编辑与排版。
- 光标移动、选区，以及行为可预期的编辑和撤销。
- 在明确说明的支持范围内导入、导出 LaTeX。
- 作为 Slint 组件嵌入其他应用。

本项目是独立维护的 Slint 第三方组件。

## 开发

需要 Rust 1.85 或更新版本。当前骨架没有 GUI 依赖；接入 Slint 组件时补充各平台桌面环境要求。

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo doc --locked --no-deps
```

贡献与验证方式见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可证

采用 **MIT OR Apache-2.0** 双许可，使用者可选择 [MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE)。
