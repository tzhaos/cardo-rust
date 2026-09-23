<p align="center">
  <img src="assets/logo.svg" width="112" alt="Cardo">
</p>
<h1 align="center">Cardo</h1>
<p align="center">A shared foundation for Rust desktop applications.</p>

<p align="center"><a href="README.zh.md">简体中文</a> · English · <a href="LICENSE">MIT</a></p>

Reusable application infrastructure and GPUI components for Rust desktop software.

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

Applications own their schemas, validation and paths. Cardo supplies persistence primitives:

- `config::Snapshot<T>` loads typed TOML and saves atomically with a previous-content backup and external-edit detection. Only missing files use defaults; read and parse failures return errors.
- `database::Database` provides SQLite transactions, backups and integrity checks. Existing databases must match the supplied application identity and schema version.
- `storage::Store` provides atomic file writes and JSON serialization helpers.

Run storage operations outside UI rendering. File locks coordinate cooperating writers; configuration files and databases do not share a transaction.

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

See [artwork and dependency provenance](THIRD_PARTY.md).
