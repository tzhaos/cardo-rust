<p align="center">
  <img src="assets/logo.svg" width="112" alt="Cardo">
</p>
<h1 align="center">Cardo</h1>
<p align="center">为 Rust 桌面应用提供统一基础设施。</p>
<p align="center">简体中文 · <a href="README.md">English</a> · <a href="LICENSE">MIT</a></p>

Cardo 提供可复用的应用基础设施与 GPUI 组件，已接入 [Plus7z](https://github.com/tzhaos/7zp-rust)。

| Crate | 职责 |
| --- | --- |
| `cardo-runtime` | TOML 配置、SQLite 状态、原子文件存储、本地化、诊断与注册表归属检查 |
| `cardo-ui` | GPUI Kit 设置控件、菜单、提示、文本、字体与主题辅助组件 |

## 存储边界

| 媒介 | 用途 |
| --- | --- |
| TOML | 可手动编辑的偏好与应用配置 |
| SQLite | 历史记录及需要事务的运行状态 |
| JSON | 数据交换，以及 SQLite 内的结构化值 |

产品负责路径、数据结构与业务校验，Cardo 提供持久化能力。详见[存储约定](docs/storage.md)，包括并发、错误和备份行为。不提供旧数据导入或数据库版本转换。

## 接入

将 Cardo 作为 Git submodule 引入，再使用 Cargo 路径依赖：

```toml
[dependencies]
cardo-runtime = { path = "cardo/crates/cardo-runtime" }
cardo-ui = { path = "cardo/crates/cardo-ui" }
```

工具链及依赖均已固定。在 Windows 上构建：

```powershell
cargo build --workspace --locked --release --target x86_64-pc-windows-msvc
```

采用 [MIT 许可](LICENSE)。标识沿用 Plus7z 原有的紫色应用立方体，来源见[素材与依赖说明](THIRD_PARTY.md)。
