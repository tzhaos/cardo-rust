# Storage contracts / 存储约定

Cardo supplies mechanisms; products own schemas, paths, retention and semantic validation. Run filesystem and database work outside UI rendering.

## TOML — configuration

`config::Snapshot<T>::load(path, default)` uses defaults only when the file is missing. Unreadable and malformed files return errors with path context. Validate product semantics before calling `save(value)`.

Saving takes a cooperative cross-process lock and compares the current source with the loaded source. A stale snapshot fails rather than overwriting external edits. Reload before retrying. `toml_edit` preserves unrelated fields and comments while applying changed typed values. Saves use atomic replacement in the same directory and retain the previous source as `.toml.bak`.

External editors do not honor Cardo's lock; this is not a transaction with arbitrary external writers. The backup and primary file are separate atomic writes. Keep unknown fields forward-readable, but reject unsupported product schema versions explicitly.

## SQLite — runtime state

`database::Database::open(path, application_id, version, schema)` initializes an empty database. Existing databases must match the exact application identity and schema version; mismatches fail without conversion or reset.

Each operation opens its own connection with a five-second busy timeout, foreign keys and `synchronous=FULL`. Initialization requires WAL. `read` runs a closure with a connection; callers needing a consistent multi-query snapshot must use a transaction. `write` uses an IMMEDIATE transaction and commits only when the closure succeeds. Keep filesystem, network and process work outside the transaction.

Products choose relational columns for values they filter, order or constrain. JSON may hold small typed aggregates inside SQLite; it is not a second independent state file. Bound history and payload size according to product needs.

`backup(destination)` uses SQLite's backup API and refuses to overwrite an existing destination. The destination directory must exist. `check_integrity()` reports database integrity errors. Do not copy a live database file alone while WAL is active. This layer does not encrypt data or provide cross-medium transactions.

## JSON — exchange

Use JSON for external payloads, manifests and export formats. `storage::Store` provides atomic file writes and JSON helpers when a standalone exchange file is required. Product preferences belong in TOML; mutable history and runtime state belong in SQLite. Avoid maintaining the same authoritative value in multiple media.

## Plus7z mapping / 产品落点

| Data / 数据 | Location / 位置 |
| --- | --- |
| Preferences, appearance, theme, language / 偏好、外观、主题、语言 | `%LOCALAPPDATA%\p7z\settings.toml` |
| Ordered history / 有序历史记录 | `state.sqlite3`, relational `history` table |
| Update state and recent destination / 更新状态与最近目标目录 | `state.sqlite3`, JSON values in `state` table |
| Release metadata and external payloads / 发布元数据与外部载荷 | JSON exchange |

配置缺失可使用默认值；读取失败、解析失败、归属或版本不匹配必须报错，不得静默覆盖。产品负责校验与错误文案；日志保留错误链，避免记录密码或命令参数。

不读取旧产品目录，不导入旧 JSON 配置，不做数据库版本转换，不保留兼容层。构建成功只证明编译与链接通过；并发、故障恢复及图形界面行为须单独验证。
