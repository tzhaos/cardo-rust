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

## Desktop services

Direct platform/update dependencies retain license texts under `licenses/`: async-channel 2.5.0 (Apache-2.0 OR MIT), windows 0.61.3 (Apache-2.0 OR MIT), reqwest 0.12.28 (Apache-2.0 OR MIT), semver 1.0.28 (Apache-2.0 OR MIT), sha2 0.10.9 (Apache-2.0 OR MIT), zip 2.4.2 (MIT), and Tokio 1.53.1 (MIT). `desktop-dependencies.txt` combines these notices for distribution. This is a principal-dependency inventory, not a complete transitive license inventory.

The desktop template embeds GPUI Kit 0.6.1 AllAssets, including Lucide glyphs. Its Lucide/Feather license is retained verbatim in `licenses/lucide.txt`; the build script copies it next to the executable. The template build also copies vcruntime140.dll from the locally installed Visual Studio C++ redistributable. Microsoft redistribution terms apply; that binary is not MIT licensed or stored in this repository.
