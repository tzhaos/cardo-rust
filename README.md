<p align="center">
  <img src="assets/logo.svg" width="112" alt="Cardo">
</p>
<h1 align="center">Cardo</h1>
<p align="center">A shared foundation for Rust desktop applications.</p>

<p align="center"><a href="README.zh.md">简体中文</a> · English · <a href="LICENSE">MIT</a></p>

Reusable application infrastructure and GPUI components, used by [Plus7z](https://github.com/tzhaos/7zp-rust).

| Crate | Responsibility |
| --- | --- |
| `cardo-runtime` | TOML configuration, SQLite state, atomic file storage, localization, diagnostics and registry ownership |
| `cardo-ui` | GPUI Kit settings controls, menus, tooltips, text, fonts, and theme helpers |

## Storage

| Medium | Purpose |
| --- | --- |
| TOML | Human-editable preferences and application configuration |
| SQLite | History and transactional runtime state |
| JSON | Exchange payloads and structured values inside SQLite |

Products own their schemas, validation and paths. Cardo supplies persistence primitives. See [storage contracts](docs/storage.md) for concurrency, errors and backup behavior. There is no legacy import or schema conversion.

## Use

Include Cardo as a Git submodule and reference its crates:

```toml
[dependencies]
cardo-runtime = { path = "cardo/crates/cardo-runtime" }
cardo-ui = { path = "cardo/crates/cardo-ui" }
```

The Rust toolchain and dependencies are pinned. Build on Windows with:

```powershell
cargo build --workspace --locked --release --target x86_64-pc-windows-msvc
```

Licensed under MIT. See [LICENSE](LICENSE).

The logo reuses Plus7z's original purple Application cube. See [artwork and dependency provenance](THIRD_PARTY.md).
