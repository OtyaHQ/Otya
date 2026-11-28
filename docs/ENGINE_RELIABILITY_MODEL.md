# Otya Engine Reliability Model

Reliability is a set of verified behaviors under failure, not a claim that the engine never fails.

## 1. Failure domains

Design and test for:
- process crash, forced termination and machine reboot;
- power loss during a write, migration or upgrade;
- full disk, read-only filesystem and permission changes;
- corrupted or unavailable storage;
- duplicate requests and concurrent callers;
- expired or rotated credentials;
- provider rate limits, outages, malformed callbacks and timeouts;
- partial network connectivity and DNS/TLS failures;
- clock skew and host clock changes;
- incompatible module and schema versions;
- resource exhaustion and hostile input.

## 2. Data safety

Every durable module states its persistence requirement. The engine must never silently place critical records in volatile memory.

Storage adapter conformance tests cover:
- atomic commit and rollback;
- isolation level and concurrent write behavior;
- uniqueness and idempotency under race;
- durable commit semantics;
- connection loss during transaction;
- migration lock and schema version;
- integrity checks and backup compatibility.

SQLite is appropriate as an optional embedded single-node adapter, not as a universal distributed database. Use the SQLite Online Backup API or a supported backup mechanism rather than casually copying a live WAL database file. PostgreSQL is an optional customer-managed choice for multi-process or multi-host workloads.

## 3. External operations

Use an outbox to atomically record intent with local state, and an inbox to deduplicate verified incoming provider events.

Each external operation stores:
- stable operation ID and idempotency key;
- canonical request digest;
- provider and adapter version;
- created/updated timestamps and attempt number;
- provider reference when known;
- state: queued, dispatching, succeeded, failed, unknown, cancelled or needs_review;
- safe error category and retry decision.

A timeout after dispatch is UNKNOWN, not FAILED. Do not resend a non-idempotent action until the provider is queried or other evidence establishes that the first attempt was not accepted. Providers may have finite idempotency windows; the engine retains its own history.

Retries are bounded, use exponential backoff with jitter, respect Retry-After and stop on permanent errors. Circuit breakers prevent repeated calls to failing providers. Dead-letter items have a review/replay workflow.

Exactly-once remote effects are not guaranteed. Use idempotency, reconciliation and explicit uncertainty.

## 4. Process and job lifecycle

- Graceful shutdown stops intake, renews/finishes or releases leases, flushes safe state and exits within a configured deadline.
- Startup recovers expired leases and resumes only operations whose replay policy is safe.
- Jobs have leases, heartbeats, bounded concurrency and maximum execution duration.
- A scheduler uses persisted schedules and unique occurrence keys to avoid duplicate runs.
- A dead-letter queue is durable and inspectable; replay requires authorization and preserves the original operation identity.

## 5. Backup and restore

A backup is not complete until a restore has been tested.

The backup manifest records engine version, module versions, storage type, schema versions, creation time, included data sets, file sizes and checksums. Secrets are excluded by default. Customer-controlled encryption keys are supported for sensitive archives.

Required operations:
- consistent online backup;
- backup integrity verification;
- restore to an isolated destination;
- schema/module compatibility check;
- restore report and data-count comparison;
- documented RPO and RTO selected by the operator;
- backup retention and off-site copy configured by the operator.

Do not claim an RPO/RTO until tested under a declared workload.

## 6. Migrations and upgrades

Migrations are versioned, ordered, checksummed and owned by modules. The migration runner:
1. acquires a storage-specific migration lock;
2. verifies preconditions and free space;
3. checks a backup requirement for risky changes;
4. applies one migration at a time;
5. records durable migration state;
6. verifies postconditions;
7. stops safely on uncertainty.

Backward compatibility is preferred. A binary rollback is not automatically a data rollback. If a migration changes data irreversibly, require a restore strategy or a forward-fix plan.

## 7. Observability and privacy

Local-first defaults:
- structured local logs with rotation and bounded disk use;
- readiness/liveness and dependency health;
- metrics for queue age, retries, failures, latency, storage and resource usage;
- correlation IDs across API, jobs and provider callbacks;
- redaction of credentials, OTPs, tokens, personal data and message bodies;
- diagnostic bundle generated only on operator request and previewed before export;
- telemetry disabled by default.

OpenTelemetry APIs/exporters may be optional adapters. Do not make an Otya-hosted telemetry account a requirement.

## 8. Reliability test programme

- unit and property tests for core invariants;
- integration tests for each storage and provider adapter;
- concurrency/race tests for idempotency and module isolation;
- fault injection for process death, timeouts, disk-full and provider failures;
- crash-recovery tests at every transaction/migration boundary;
- backup/restore drills on clean hosts;
- load and soak tests with published reference hardware;
- security tests for authorization, signature replay and resource limits.

Each release publishes the exact tests run, platform matrix, known limitations and recovery instructions.
