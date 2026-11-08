<p align="center">
  <img src="docs/assets/brand/terminal-ui-contract-logo.svg" width="420" alt="Terminal UI Contract" />
</p>

# terminal-ui-contract

面向两种终端交互的 Rust 工作区。**Ratatui 用于全屏 TUI，cliclack 用于行内 CLI 提示。**
两个适配器共享与后端无关的 `terminal-ui-contract` Core。应用可按需引入
`terminal-ui-runtime`，实现协作式只读刷新。

[English](README.md) · 简体中文 · [文档](docs/README.md) · [MIT](LICENSE)

## 选择 crate

| 使用场景 | Cargo 包名 | 获取方式 |
| --- | --- | --- |
| 全屏仪表盘、窗格、焦点、键盘和鼠标 | [`terminal-ui-ratatui`](crates/ratatui/README.md) | [crates.io 0.1.0](https://crates.io/crates/terminal-ui-ratatui/0.1.0) |
| 行内单选、多选、输入、确认和进度 | [`terminal-ui-cliclack`](crates/cliclack/README.md) | 固定 Git 提交，工作区版本为 0.1.0 |
| 共享键位、焦点、字符显示宽度、语言和语义主题 | [`terminal-ui-contract`](src/lib.rs) | [crates.io 0.2.0](https://crates.io/crates/terminal-ui-contract/0.2.0) |
| 后台只读刷新与最近一次成功接纳的快照 | [`terminal-ui-runtime`](crates/runtime/README.md) | [crates.io 0.1.0](https://crates.io/crates/terminal-ui-runtime/0.1.0) |

两个适配器是独立的 Cargo 包，可分别使用。两个适配器和 Runtime 都依赖 Core。
Ratatui 包只在开发依赖中引入 Runtime，用于合成测试场景。应用可按需用
Runtime 接入自己的采集器。整个工作区使用 Rust 1.97.1 或更高版本、Edition 2024 和 MIT 许可证。

## 添加依赖

全屏应用从 Ratatui 适配器开始。需要直接使用共享类型时添加 Core，
需要后台刷新时再添加 Runtime。

```toml
[dependencies]
terminal-ui-ratatui = "=0.1.0"
terminal-ui-contract = "=0.2.0"
# terminal-ui-runtime = "=0.1.0"
```

行内提示使用固定的 Git 提交。适配器重新导出了 `PromptTheme` 所需的
`Locale` 和 `ColorLevel`。

```toml
[dependencies]
terminal-ui-cliclack = { git = "https://github.com/JerrySkywalker/terminal-ui-contract", rev = "a175d97151a9b82d7defa65afe5a9fb5bc56e46e" }
```

如果应用在两个适配器之间传递 `Locale` 等 Core 类型，请让两个适配器、Core
以及用到的 Runtime 指向同一个 Git URL 和提交。即使版本号相同，Cargo 也会把
Git 来源与注册表来源的 Core 视为不同的 crate。依赖 Git 版适配器的应用如果要上架
crates.io，需单独规划发布依赖。见[发布顺序](docs/RELEASE_ORDERING.md)。

## 运行示例

以下示例可在当前源码的工作树中运行。

```sh
# 使用 Ratatui TestBackend 将全屏布局输出为文本
cargo run --locked -p terminal-ui-ratatui --example gallery -- --size 100x32 --no-color

# 在应用独占的交互式终端中体验行内提示
cargo run --locked -p terminal-ui-cliclack --example menu
cargo run --locked -p terminal-ui-cliclack --example wizard
```

[Ratatui 示例](crates/ratatui/README.md)还包含可人工操作的真实终端演示。
行内提示只在交互路径创建 `PromptSession`。分别处理 `PromptOutcome::Submitted`、
`Cancelled` 和 `Error`，由应用负责验证并执行提交的选择。帮助、JSON 与重定向路径
应独立处理。切换两种界面前，先释放当前终端会话。

## 开发工作区

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --locked
cargo package --workspace --locked
```

[Windows 和 Linux CI](https://github.com/JerrySkywalker/terminal-ui-contract/actions/workflows/ci.yml)
运行上述检查。[文档索引](docs/README.md)汇总源码入口、键位配置、历史 API 和验收记录。
