<p align="center">
  <img src="assets/logo.svg" width="112" alt="Cardo">
</p>
<h1 align="center">Cardo</h1>
<p align="center">为 Rust 桌面应用提供统一基础设施。</p>
<p align="center">简体中文 · <a href="README.md">English</a> · <a href="LICENSE">MIT</a></p>

Cardo 为 Rust 桌面软件提供可复用的应用基础设施与 GPUI 组件。

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

应用负责路径、数据结构与业务校验，Cardo 提供持久化能力：

- `config::Snapshot<T>` 加载强类型 TOML，支持原子保存、旧内容备份与外部修改检测。只有文件缺失才使用默认值；读取与解析失败均返回错误。
- `database::Database` 提供 SQLite 事务、备份及完整性检查。已有数据库必须匹配传入的应用标识与结构版本。
- `storage::Store` 提供原子文件写入与 JSON 序列化辅助方法。

存储操作应在 UI 渲染之外执行。文件锁协调遵守同一约定的写入方；配置文件与数据库之间不提供联合事务。

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

采用 [MIT 许可](LICENSE)。素材来源及依赖许可见[素材与依赖说明](THIRD_PARTY.md)。
