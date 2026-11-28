# Otya Engine SQLite Adapter

## Scope and package boundary

The initial SQLite adapter lives in the separate `otya-engine-storage-sqlite` crate. It is Linux-only in this implementation and is not a dependency of `otya-engine-core` or the operator CLI. Applications add and link it only when they choose SQLite.

The adapter uses `rusqlite` without its bundled-SQLite feature. Builds require the system SQLite development library and `pkg-config`; the deployed application requires the compatible system SQLite shared library. For Debian/Ubuntu development hosts:

```sh
sudo apt-get install libsqlite3-dev pkg-config
```

The exact system package names differ by distribution. The engine does not download, bundle, run or operate a database server. Keeping SQLite optional means the core and CLI do not gain a mandatory database runtime.

## Current API

The crate currently exposes:

- `SqliteStore::open(path)` and `open_with_options(path, options)`;
- bounded SQLite busy-timeout configuration (100 ms to 120 seconds; default five seconds);
- an Otya application ID and schema-version header;
- a first metadata record in `otya_engine_migrations`;
- `transaction(...)` for explicit immediate transactions and rollback on closure error;
- `integrity_check()` using SQLite's built-in integrity check.

This is an initial storage foundation, not yet the engine's generic storage trait or a complete database product. It creates only Otya metadata; it does not create business tables, users, tenant records, jobs, outbox/inbox tables or provider data models.

## Database location and initialization

Use the selected installation's private data directory. For the default XDG layout, the intended location is:

```text
$XDG_DATA_HOME/otya-engine/engine.db
```

The default when `XDG_DATA_HOME` is unset is `~/.local/share/otya-engine/engine.db`. A named installation uses its own `instances/NAME` data namespace. Resolve the actual path through the engine path API; TOML paths do not expand `~` or environment variables.

The operator first explicitly prepares the installation directories, for example:

```sh
otya paths --create
```

The host application then opens the optional adapter using an absolute database path:

```rust
use otya_engine_storage_sqlite::SqliteStore;

let store = SqliteStore::open("/home/operator/.local/share/otya-engine/engine.db")?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The example path is illustrative; use the actual operator path returned by Otya. The adapter does not create the parent directory.

On first open, the adapter exclusively creates the database file with mode `0600`. Existing entries are never followed as symlinks. Existing files must be regular files owned by the current OS user; the file is hardened to mode `0600`. The parent directory is checked and hardened by the same descriptor-based no-follow Linux filesystem layer. Root execution is refused for this per-user adapter.

## Database identity and preservation

A blank SQLite file with no user objects may be initialized by Otya. The adapter records:

- `application_id = 0x4F545941` (`OTYA`);
- `user_version = 1`;
- the initial metadata migration identifier in `otya_engine_migrations`.

An existing Otya database is opened only when these values match the expected metadata. A database with tables but no Otya identity, an unrelated application ID, an unsupported schema version, missing metadata or an inconsistent migration identifier is rejected. It is not silently claimed, reset, dropped or migrated.

The initial open is guarded by an exclusive SQLite transaction. WAL mode is enabled only after the database identity check and metadata transaction have succeeded.

## Runtime behavior

The connection sets foreign-key enforcement, disables trusted schema, uses full synchronous durability, stores temporary tables in memory and enables WAL. Each call to `transaction` acquires an immediate transaction. Successful callbacks commit; callback or SQL errors cause rollback when the transaction is dropped.

A `rusqlite::Connection` is not a concurrently shared `Sync` connection. Use one connection per worker or protect one connection with the host application's synchronization primitive. WAL and the busy timeout coordinate separate connections; this first adapter does not provide a connection pool or a higher-level concurrency scheduler.

The built-in integrity check returns SQLite's result messages. A healthy database returns exactly `ok`; applications must treat any other result as a failed integrity check.

## What remains incomplete

This crate is a tested engineering foundation, not a production-ready storage layer. It does not yet include:

- a generic Otya storage/transaction trait or adapter conformance suite;
- module-owned business schemas and a general versioned migration runner;
- migration lock recovery, preflight/free-space checks or forward/rollback plans;
- online backup API integration, encrypted/checksummed backup, restore verification or recovery drills;
- database export/import, retention or deletion lifecycle;
- durable idempotency, jobs, outbox, inbox, scheduler or reconciliation;
- multi-connection stress, process-crash, power-loss, disk-full and corruption fault-injection tests;
- Windows/macOS support or a broad Linux distribution/SQLite version matrix;
- CLI database initialization, migration or backup commands.

Do not place production or irreplaceable data in this experimental branch. Do not copy a live WAL database as a casual backup; consistent online backup and tested restore are separate future work.

## Verification status

The current Linux adapter code and tests passed Otya Engine CI [run 193](https://github.com/OtyaHQ/Otya/actions/runs/46141897437) on commit `f560e0a0882d75bd497f64aadf413bf49bb35c3c`. The run passed stable formatting, Clippy, workspace tests, CLI smoke tests, example config validation, release build and Rust 1.85 workspace compile/tests. Coverage includes fresh database initialization, Otya identity, WAL, integrity check, reopen, read-only file permission hardening, refusal/preservation of unrelated SQLite databases, symlink refusal and transaction rollback.

This evidence is for the GitHub-hosted Linux runner and the system SQLite version present there. It is not a release guarantee for every Linux distribution, SQLite build or filesystem. The incomplete items above remain release blockers.
