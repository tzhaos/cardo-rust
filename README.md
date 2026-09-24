<p align="center"><img src="assets/logo.svg" width="112" alt="Cardo"></p>
<h1 align="center">Cardo</h1>
<p align="center">A composable application shell for Rust desktop software.</p>
<p align="center"><a href="README.zh.md">简体中文</a> · <a href="templates/desktop/README.md">Start an app</a> · <a href="docs/application.md">API guide</a> · <a href="LICENSE">MIT</a></p>
<p align="center"><strong>Windows first</strong> · GPUI Kit 0.6.1 · Rust 1.95.0</p>

Cardo provides window and service lifecycles, shared desktop controls, configuration and optional updates. Applications supply their identity, pages and business services, and compose their own navigation.

## What is included

| Crate | Responsibility |
| --- | --- |
| `cardo-runtime` | Cancellation, typed task state, settings drafts and shortcuts; optional TOML configuration, SQLite, catalogs and diagnostics |
| `cardo-platform` | Windows single instance, launch delivery, system file icons, native file pickers and process helpers |
| `cardo-ui` | Panels, controls, titlebars, connected navigation, native menus, settings, dialogs, draggable Toasts and task delivery |
| `cardo-update` | Release discovery, downloads, checksum verification, staged replacement, backups and recovery |
| `cardo-app` | Application descriptor, ordered startup, main window, launch messages and shutdown |

The [independent desktop template](templates/desktop/README.md) includes Home, Settings and About, three languages, light/dark themes and a real cancellable file task. It has its own identity, data directory and Cargo workspace.

## Start here

```powershell
git clone https://github.com/tzhaos/cardo-rust.git
cd cardo-rust
./templates/desktop/build.ps1
./target/x86_64-pc-windows-msvc/release/cardo-desk.exe
```

Requires Windows x64, Visual Studio C++ Build Tools and Windows SDK. The template build script includes the locally installed Visual C++ runtime. To build all framework crates:

```powershell
cargo build --workspace --all-features --locked --release --target x86_64-pc-windows-msvc
```

Reference the required crates through a pinned submodule and opt into services explicitly:

```toml
cardo-app = { path = "cardo/crates/cardo-app", features = ["config", "localization", "diagnostics"] }
cardo-ui = { path = "cardo/crates/cardo-ui", features = ["localization"] }
```

`cardo-runtime` and `cardo-app` have empty default feature sets. SQLite and updates are optional. TOML owns editable preferences; SQLite owns transactional state; JSON carries exchange payloads and structured database values. Applications own schemas, validation, data paths and installation policy. See the [API and storage guide](docs/application.md).

## Scope

Windows is the supported implementation target for this release. macOS and Linux are not promised. Domain engines, business commands, application artwork and registry policy belong to applications. Cardo does not import old data or migrate schemas. See [verification and remaining gaps](docs/verification.md).

[MIT](LICENSE). The original purple cube is retained; dependency and artwork provenance is recorded in [THIRD_PARTY.md](THIRD_PARTY.md).
