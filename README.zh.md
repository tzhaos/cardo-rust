<p align="center"><img src="assets/logo.svg" width="112" alt="Cardo"></p>
<h1 align="center">Cardo</h1>
<p align="center">可组合的 Rust 桌面应用壳。</p>
<p align="center"><a href="README.md">English</a> · <a href="templates/desktop/README.zh.md">创建应用</a> · <a href="docs/application.zh.md">接口说明</a> · <a href="LICENSE">MIT</a></p>
<p align="center"><strong>Windows 优先</strong> · GPUI Kit 0.6.1 · Rust 1.95.0</p>

Cardo 提供窗口与服务生命周期、统一桌面控件、配置和可选更新能力。应用提供身份、业务页面和服务，自行组织导航与布局。

## 提供什么

| Crate | 职责 |
| --- | --- |
| `cardo-runtime` | 取消信号、类型化任务状态、设置草稿和快捷键；可选 TOML 配置、SQLite、本地化与诊断 |
| `cardo-platform` | Windows 单实例、启动消息、系统文件图标、原生文件选择和进程辅助 |
| `cardo-ui` | 面板、控件、标题栏、连体导航、原生菜单、设置、弹窗、可拖动 Toast 和任务交付 |
| `cardo-update` | 发布检查、下载、校验、暂存替换、备份与恢复 |
| `cardo-app` | 应用描述、有序启动、主窗口、启动消息分发和退出 |

[独立桌面模板](templates/desktop/README.zh.md) 包含主页、设置、关于、三种语言、浅深主题和真实可取消的文件任务，拥有独立身份、数据目录和 Cargo workspace。

## 开始使用

```powershell
git clone https://github.com/tzhaos/cardo-rust.git
cd cardo-rust
./templates/desktop/build.ps1
./target/x86_64-pc-windows-msvc/release/cardo-desk.exe
```

需要 Windows x64、Visual Studio C++ Build Tools 和 Windows SDK。模板构建脚本会附带本机工具集中的 Visual C++ 运行库。构建全部框架 crate：

```powershell
cargo build --workspace --all-features --locked --release --target x86_64-pc-windows-msvc
```

通过固定提交的 submodule 引用所需 crate，显式启用服务：

```toml
cardo-app = { path = "cardo/crates/cardo-app", features = ["config", "localization", "diagnostics"] }
cardo-ui = { path = "cardo/crates/cardo-ui", features = ["localization"] }
```

`cardo-runtime` 和 `cardo-app` 的默认 feature 均为空，SQLite 与更新按需启用。TOML 保存可编辑偏好，SQLite 保存事务性状态，JSON 用于交换载荷和数据库内的结构化值。应用负责数据结构、验证、路径和安装策略，详见[接口与存储说明](docs/application.zh.md)。

## 范围

本版以 Windows 为实现目标，不承诺 macOS 或 Linux。业务引擎、业务命令、应用图标及注册策略由应用维护。Cardo 不导入旧数据、不迁移数据库结构。详见[验证记录与缺口](docs/verification.zh.md)。

采用 [MIT](LICENSE)，保留原有紫色立方体。依赖与素材来源见 [THIRD_PARTY.md](THIRD_PARTY.md)。
