# v0.3.0 verification

[简体中文](verification.zh.md) · [Home](../README.md)

Recorded on 24 September 2026 on Windows x64. No tests or smoke-test code were added. This record separates source/build checks from observed behavior.

## Build and static checks

- Framework release build with all features on `x86_64-pc-windows-msvc`.
- `cardo-runtime` and `cardo-app` with no default features.
- Independent desktop template release builds with default features and with `update` enabled; its own lockfile and application version are retained.
- Template resources and runtime resources have matching message keys/parameters across all three locales.
- Framework crates and template contain no downstream product dependency or identity.
- Target-window input factories, weak task delivery, pending-save snapshots, instance worker shutdown and update path/manifest checks were reviewed in source.

## Observed behavior

- Cardo Desk ran independently and streamed a 1,067-byte local file through SHA-256. The result matched the OS hash tool.
- Home, Settings and About navigation worked. The initial missing window-control icons and titlebar navigation interception were fixed, rebuilt and observed again.
- Settings saved Simplified Chinese and dark appearance to the template's own TOML file. Save notification appeared and subsequently disappeared; expiry was not measured to millisecond precision.
- An external TOML edit during the session was rejected with a path-bearing conflict message, preserving the edited source.
- A second template launch forwarded a file to the existing window and exited successfully.
- A consuming application exercised native form open, repeated open, Cancel, Escape and Alt+F4; a target-window input was focused. Background completion and passive completion/skip notifications were observed.

## Remaining runtime gaps

- Forced native-window creation failure, complete focus-restoration matrix, rapid replacement races, owner destruction during a worker, and interrupted native picker calls.
- Toast edge dragging, replacement timer races, all DPI/multi-monitor combinations and exhaustive long-text/menu boundaries.
- Full shortcut conflict/disable/recording matrix and every settings validation/save-failure combination.
- Isolated update download cancellation, corrupt package rejection, installer/portable application, interrupted update recovery and rollback have **not** been executed for this extraction. Source review and compilation do not establish those outcomes.
- GPUI native-window/accessibility diagnostics remain in local logs. This release does not claim exhaustive visual, accessibility or cross-platform validation.

Release tags are pushed after local checks. Remote workflow completion is intentionally not polled as part of this acceptance.
