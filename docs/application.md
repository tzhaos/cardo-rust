# Application and service guide

[Home](../README.md) · [简体中文](application.zh.md)

## Composition

`cardo-runtime` has no GPUI dependency. Platform depends on runtime; UI depends on runtime and optionally platform for `system-icons`; update depends on runtime/platform. App composes these services. Framework crates do not depend on application crates.

| Feature | Enable on | Adds |
| --- | --- | --- |
| `config` | runtime / app | Typed TOML snapshots and `ConfigService<T>` |
| `database` | runtime / app | Bundled SQLite |
| `localization` | runtime / ui / app | Project Fluent catalogs and Cardo messages |
| `diagnostics` | runtime / app | File diagnostics |
| `system-icons` | ui | Platform image adaptation |
| `update` | app | Optional update service dependency |

Defaults are empty. Using `cardo-update` enables runtime localization for its messages. GPUI Kit is fixed at 0.6.1. Cardo does not bundle Microsoft's Fluent UI icons; Project Fluent is its localization engine.

## Startup

`AppDescriptor` supplies stable ID, display name, application version, data directory, window sizes, single-instance preference and optional update configuration. Update identity/version must match the application.

`AppServices` runs in this order:

1. Optional diagnostics initialize under the supplied data directory.
2. `initialize` explicitly loads configuration and catalogs.
3. `maintenance` routes helper modes and returns without a main window or regular instance delivery.
4. `prepare` performs normal startup preparation.
5. The instance service acquires ownership or forwards arguments and exits.
6. `initialize_ui` applies fonts/theme before `create` constructs the main page.
7. `launch` receives initial and subsequent requests in the originating window.
8. Window closure quits the application; `shutdown` runs after the UI event loop returns.

Keep argument parsing, registration and business routing in these hooks. The optional update descriptor does not automatically install an updater or expose update UI; construct `UpdateService` with application adapters. The [template](../templates/desktop/README.md) demonstrates the complete non-update path.

## Windows and interaction

Set `theme::set_presentation` and apply `ThemeStyle` before rendering shared controls. `app_frame`, `panel`, `settings_page`, `chrome` and `controls` compose layouts without fixing a navigation scheme. Apps supply brand colors, artwork and semantic dialog sizes.

`DialogHost::open` receives an owner, owner window/content bounds, sizes, focus handle, target-window content factory, renderer and close policy. Create input entities in that factory and install returned content in the owner before leaving the opening callback. Rendering is guarded during window construction. Placement is centered over measured content and clamped to visible display bounds. Native creation errors propagate; cancel related work and report failure.

Closing distinguishes Submit, Cancel, Escape, Window and OwnerReleased. The host owns native handles, observation/release subscriptions, focus restoration and generation guards. When replacing a dialog, dispose previous business state and invalidate its generation before installing new state; stale deferred callbacks cannot affect the new dialog. Apps decide whether active work allows dismissal and own business cleanup.

`ToastHost` owns replacement, generation, a three-second timer and drag offset. Forward pointer movement/release and supply content bounds. Each message resets position and expiry. Paths, actions and detailed errors belong in dialogs. Native menus retain window/generation ownership; settings selectors use themed popovers.

## Tasks, settings and commands

`CancellationToken` requests cooperative cancellation. `TaskState<P>` separates Running, Cancelling and Finished and carries actual application progress. UI `task::spawn` delivers typed results to a weak owner. Dropping the handle requests cancellation and detaches delivery so the worker can finish and release resources. Keep temporary resources in the worker. `refresh_while` updates only while real progress remains active.

`SettingsDraft<T>` captures the submitted snapshot, tracks in-flight saves and field errors, retains edits on failure and commits the submitted snapshot on success. Apps block navigation/closure during saves and own validation and side effects. `ConfigService<T>` serializes access to a TOML snapshot; it does not transact across files, SQLite or registry changes.

`CommandDescriptor<A>` and overrides represent default, disabled and explicit bindings. Use command identities for availability/execution and the same resolved bindings for menus and dispatch. Shared conflict checks, recording and input protection accept application restrictions. Never branch on translated labels.

`CatalogSet` owns resources and selected language. Load application catalogs explicitly, then configuration, then select the locale; catalog initialization must not initialize settings implicitly. Cardo's separate `cardo-*` namespace contains English, Simplified Chinese and Traditional Chinese resources.

## Storage boundaries

| Medium | Use | Guarantees and limits |
| --- | --- | --- |
| TOML | Editable preferences | Only missing files use defaults. Read/parse errors include paths. Atomic writes preserve comments, retain previous-content backups and reject external edits. |
| SQLite | History and runtime state | Exact application ID/schema matching; short transactions. Finish and pending-record removal share a transaction. No migrations/imports. |
| JSON | Protocol/release payloads, structured SQLite values | Application-owned serialization, not another preference store. |

`database::Database` supplies integrity checks, transactions and backups with WAL, FULL synchronization and a five-second busy timeout. Run storage outside render. Apps define schemas, paths, retention and error presentation. Registry ownership helpers do not choose product registry paths or association policy.

## Platform and updates

`instance::Instance` owns its receiver and Windows pipe worker until drop; no UI polling is needed. System icons return independent image data. File pickers return `Result<Option<T>>`, distinguishing selection, cancellation and failure. Process helpers act on supplied executable identities.

`UpdateConfig` requires version/repository, asset names, archive root, allowed relative files, executable/helper files, staging/jobs locations and download limit. `UpdateJournal` reads/saves pending work and outcomes; `finish` atomically writes an outcome and removes pending work. `InstallAdapter` owns installation identity, installer invocation, extra backups, registration restoration and version verification.

Cardo discovers stable releases, limits downloads, verifies SHA-256, checks manifests and paths, validates process identities, backs up replacements and restores failures. Products supply their own installer/policy. Configure real release information before enabling updates. Verify apply/recovery in isolated installations; compilation is not runtime acceptance.
