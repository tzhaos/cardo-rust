<p align="center">
  <img src="assets/logo.svg" width="112" alt="Cardo">
</p>
<h1 align="center">Cardo</h1>
<p align="center">A shared foundation for Rust desktop applications.</p>

Reusable application infrastructure and GPUI components, extracted from [7zplus](https://github.com/tzhaos/7zp-rust).

| Crate | Responsibility |
| --- | --- |
| `cardo-runtime` | Atomic storage, Fluent localization, diagnostics, and registry ownership helpers |
| `cardo-ui` | GPUI Kit settings controls, menus, tooltips, text, fonts, and theme helpers |

Both crates are maintained in this repository. Consumers can include the repository as a Git submodule and reference the crates with Cargo path dependencies. The Rust toolchain is pinned in `rust-toolchain.toml`.

```powershell
cargo build --workspace --locked
```

Licensed under MIT. See [LICENSE](LICENSE).

The logo reuses 7zplus's purple Application cube. See [artwork provenance](THIRD_PARTY.md).
