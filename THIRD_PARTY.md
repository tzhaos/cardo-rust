# Artwork and dependencies

`assets/logo.svg` is the original purple Application cube from `tzhaos/7zp-rust`,
`assets/toolbar/general.svg` at commit `84a3e9c63763cd215f6ac56422d42c67eeb7f4e7`.
It is reused unchanged under the project's MIT license, copyright 2026 Khaos Tian.

Cardo uses GPUI Kit 0.6.1 for presentation, Fluent for localization and tracing
for diagnostics. Rust dependencies retain their own licenses, as declared in
their package manifests. `Cargo.lock` records the standalone build's versions.

SQLite is bundled through `rusqlite` 0.40.2. The rusqlite MIT license is retained
in `licenses/rusqlite.txt`; SQLite's core is in the public domain. TOML parsing
and editing use `toml` and `toml_edit`, whose package licenses remain applicable.
Cardo does not bundle Plus7z's Microsoft Fluent UI SVG icon collection.
