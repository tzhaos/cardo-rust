# Cardo

Shared Rust infrastructure for Windows desktop applications.

| Crate | Responsibility |
| --- | --- |
| `cardo-runtime` | Atomic storage, Fluent localization, diagnostics, and registry ownership helpers |
| `cardo-ui` | GPUI Kit settings controls, menus, tooltips, text, fonts, and theme helpers |

Both crates are maintained in this repository. Consumers can include the repository as a Git submodule and reference the crates with Cargo path dependencies. The Rust toolchain is pinned in `rust-toolchain.toml`.

```powershell
cargo build --workspace --locked
```

Licensed under MIT. See [LICENSE](LICENSE).
