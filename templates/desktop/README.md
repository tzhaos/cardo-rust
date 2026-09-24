# Cardo Desk

[简体中文](README.zh.md) · [Cardo](../../README.md)

A complete Windows desktop starting point with its own Cargo workspace, application identity and data directory. It includes Home, Settings and About, theme and language settings, native file selection, single-instance launch forwarding, cancellable streaming SHA-256 calculation, dialogs and brief notifications.

## Build and run

Use Windows x64, the pinned Rust toolchain, Visual Studio C++ Build Tools and the Windows SDK. From a Cardo checkout:

```powershell
./templates/desktop/build.ps1
./target/x86_64-pc-windows-msvc/release/cardo-desk.exe
```

The script builds this independent workspace and copies the Visual C++ runtime from your installed toolset. It needs no downstream application, binary or asset. You can also pass a file path to the executable; a second launch forwards it to the running window.

## Make it your application

1. Copy this directory into your application's repository. Add Cardo as a submodule and change the four path dependencies in `Cargo.toml` to point into that checkout. Adjust `build.ps1`'s Cardo path accordingly.
2. Set your package name/version, `AppDescriptor` identity/name/data directory, title and locale messages. Keep the application version independent of the framework version.
3. Implement `AppServices` and compose your pages with `cardo-ui`. Call `theme::apply` before rendering shared components.
4. Keep domain validation and work in the application. The example reads files on a worker, reports actual bytes and only finishes cancellation when the worker returns.

Settings live in `%LOCALAPPDATA%\cardo-desk\settings.toml`; logs live beside them. Explicit Save captures the submitted draft; errors preserve it. External edits are rejected until you restart and reload. This template does not create a database.

Updates are disabled. `./templates/desktop/build.ps1 -WithUpdate` verifies the optional dependency configuration; it does not enable a release feed. Supply your real `UpdateConfig`, `UpdateJournal` and `InstallAdapter`, maintenance routing and update UI before setting `AppDescriptor.update` to `Some(...)`.

The template uses the Lucide assets bundled by GPUI Kit 0.6.1; keep `licenses/lucide.txt` and the applicable dependency notices when distributing it. No automated test code is included.
