# Otya Engine SQLite Adapter

**Status:** implemented and CI-tested on Linux as an experimental optional adapter; not production accepted.

## Package boundary

The adapter lives in the separate `otya-engine-storage-sqlite` crate. The Rust core defines the backend-neutral storage contract and does not depend on SQLite. The Linux operator CLI links the adapter on Linux to provide explicit `otya storage` commands; non-Linux core consumers do not gain a SQLite dependency.

The adapter uses `rusqlite` without its bundled-SQLite feature. Linux builds require the system SQLite development library and `pkg-config`; deployed Linux applications require a compatible system SQLite shared library. For Debian/Ubuntu development hosts:

```sh
sudo apt-get install libsqlite3-dev pkg-config
```

The engine does not download, bundle, run or operate a database server. SQLite is an optional single-operator local persistence adapter, not a distributed database.

## Core storage contract

The core exposes an object-safe `Storage` trait and `StorageTransaction` interface. The SQLite adapter implements that contract.

A storage transaction supports:
- `get(namespace, key)` for an optional byte value;
- `put(namespace, key, value)` for insert or replace;
- `delete(namespace, key)` with an existed/not-existed result;
- bounded, bytewise-ordered `scan_prefix(namespace, prefix, limit)`;
- explicit `commit`; dropping an uncommitted transaction rolls it back.

Core limits are enforced before database operations:
- namespace: 1–128 lowercase ASCII letters, digits, dots, underscores or hyphens;
- key: maximum 4 KiB;
- value: maximum 16 MiB;
- prefix scan: maximum 10,000 results.

Namespaces are module-owned names, not company, tenant or end-user IDs. The host application retains ownership of user identities and authorization.

## Filesystem and database identity

Use the selected installation's private data directory. For the default XDG layout, the intended location is:

```text
$XDG_DATA_HOME/otya-engine/engine.db
```

The default when `XDG_DATA_HOME` is unset is `~/.local/share/otya-engine/engine.db`. A named installation uses its own `instances/NAME` data namespace. Resolve the actual path through the engine path API; TOML paths do not expand `~` or environment variables.

Prepare the selected installation directories first, for example:

```sh
otya paths --create
```

Configure an absolute database path and run the CLI commands below. The adapter:
- creates the database file exclusively with mode `0600`;
- verifies existing files as operator-owned regular files and rejects symlinks;
- validates and hardens the parent directory through Linux no-follow filesystem helpers;
- refuses root execution for this per-user adapter;
- opens SQLite with `SQLITE_OPEN_NOFOLLOW` and without SQLite's create flag.

A private parent directory (0700) also protects SQLite's WAL and shared-memory sidecars. Processes sharing the same OS UID share that filesystem trust boundary; the adapter does not claim to isolate mutually hostile same-UID processes.

The database records `application_id = 0x4F545941` (`OTYA`), `user_version`, and stable migration identifiers in `otya_engine_migrations`. A database with unrelated objects, a foreign application ID, missing metadata or an unsupported future schema is rejected and is not silently claimed or reset.

## Schema and migration behavior

Schema version 1 created the initial Otya migration metadata. Schema version 2 adds the namespaced `otya_engine_kv` table. The v1-to-v2 change and migration record are applied in an exclusive SQLite transaction. If that transaction fails, SQLite rolls back the schema update and version change together.

`SqliteStore::open` may create a new empty database and apply the initial schema or the known v1-to-v2 upgrade. It refuses to overwrite unrelated database contents.

`SqliteStore::open_existing` only opens an existing database already at the supported schema version. It does not create a missing file or apply a migration. The CLI's check command uses this non-initializing path.

This is an initial migration, not yet a general migration framework. Future schema changes require migration planning, locks/state, preflight, interrupted-migration recovery and compatibility tests.

## Durability and concurrency behavior

The connection enables WAL journaling, foreign-key enforcement, disables trusted schema, uses `synchronous=FULL`, and sets a bounded busy timeout (100 ms to 120 seconds; default five seconds).

Storage transactions start with `BEGIN IMMEDIATE`, so write-lock acquisition happens at transaction start. One connection is protected by a mutex, and an active transaction holds that lock until commit or rollback. SQLite file locking and the busy timeout coordinate other connections/processes; this does not replace multi-process stress testing or application-level idempotency and conflict handling.

The adapter also exposes SQLite's integrity check. A healthy database returns exactly `ok`; other results must be treated as a failed integrity check.

## CLI initialization and health check

Use an explicit persistent configuration:

```toml
schema_version = 1

[engine]
mode = "embedded"

[storage]
backend = "sqlite"
path = "/absolute/operator/data/otya-engine/engine.db"
busy_timeout_ms = 5000
```

Then run:

```sh
otya config validate --file /absolute/path/otya-engine.toml
otya storage init --config /absolute/path/otya-engine.toml
otya storage check --config /absolute/path/otya-engine.toml
```

`storage init` is the explicit create/migrate action. It can create an empty database or apply the supported v1-to-v2 upgrade; it rejects unrelated database contents. `storage check` does not create a missing file or apply migrations; it validates an existing current-schema database and performs an integrity check.

The starter configuration selects memory mode, which is volatile. It has no persistent file to initialize. Do not mistake successful TOML validation with an initialized persistent database.

## What remains incomplete

This adapter is an engineering foundation, not a production-ready persistence system. It does not yet include:
- a general migration runner and module-owned business schemas;
- multi-process concurrency stress, full disk, read-only filesystem, crash and power-loss fault-injection tests;
- SQLite Online Backup API integration, encrypted/checksummed backups, restore verification and recovery drills;
- database export/import, retention and deletion lifecycle;
- durable idempotency, job queue, outbox, inbox, scheduler or reconciliation tables;
- customer-managed PostgreSQL adapter and cross-backend conformance suite;
- a broad Linux distribution and SQLite-version matrix, or Android/iOS SQLite adapter.

Do not place production or irreplaceable data in this experimental branch. Never treat a casual copy of a live WAL database file as a consistent backup.

## Verification status

The latest full Otya Engine CI run [#265](https://github.com/OtyaHQ/Otya/actions/runs/46143575958) passed formatting, Clippy, all workspace tests, CLI diagnostics, SQLite initialization/check smoke tests, example configuration validation, release build and Rust 1.85 compile/tests on the GitHub-hosted Linux runner.

This is CI evidence for that Linux runner and its system SQLite package. It is not release certification for all Linux distributions, SQLite versions, filesystems or production workloads. The recovery, backup, stress and platform acceptance gaps above remain release blockers.
