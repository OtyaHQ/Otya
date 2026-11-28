# Otya Engine Configuration Lifecycle

## Initial creation

On Linux, create the starter configuration for the default local installation:

\`\`\`sh
otya config init
\`\`\`

For a separate local instance:

\`\`\`sh
otya config init --instance payments
\`\`\`

The initializer writes:

- Default instance: \`$XDG_CONFIG_HOME/otya-engine/otya-engine.toml\`, falling back to \`~/.config/otya-engine/otya-engine.toml\`.
- Named instance: \`$XDG_CONFIG_HOME/otya-engine/instances/NAME/otya-engine.toml\`, with the same fallback base directory.

It prepares only the selected configuration directory. It does not create data, state, cache or runtime directories, start a service, create a database, contact a provider, or make a network request.

The directory is verified through directory descriptors without following symlink components and is restricted to mode \`0700\`. The config file is created exclusively with mode \`0600\`. If the target file already exists—including when it is a symlink—the command fails without opening, following, truncating or replacing it. Root execution is refused for this per-user operation.

The command may emit JSON for scripts:

\`\`\`sh
otya config init --instance payments --json
\`\`\`

## Starter storage mode

The generated starter is an embedded-mode configuration using in-memory storage. Validation emits a warning because memory storage is volatile: it does not preserve state after the process exits and is not suitable for durable workloads.

This is deliberate. The configuration schema can currently describe a SQLite path, but the SQLite storage adapter and transaction layer are not implemented. A valid SQLite-shaped TOML file must not be mistaken for a working persistent database integration. Do not place production data in this experimental engine.

## Validate after editing

Use the explicit file path shown by the initializer:

\`\`\`sh
otya config validate --file ~/.config/otya-engine/otya-engine.toml
\`\`\`

For a named instance:

\`\`\`sh
otya config validate --file ~/.config/otya-engine/instances/payments/otya-engine.toml
\`\`\`

The validator parses the TOML and checks the current schema and semantic constraints. It does not connect to providers, read secret values, open a database, run migrations or prove that a selected backend is installed.

Configuration must not contain API keys, passwords, signing secrets or tokens. Provider credentials belong in a future operator-controlled secret-store integration or another explicitly secured host facility.

## Preservation and current limitations

The initializer is create-only. It never overwrites an existing configuration and there is no \`--force\` flag. To replace an existing file, an operator must first preserve and inspect it; the current CLI does not provide atomic replacement, automatic backup, rollback or schema migration.

A normal write error triggers cleanup of the newly created incomplete file. A process crash or power loss during the first write can still leave an incomplete file. Inspect and preserve such a file before taking any manual recovery action. Crash-safe temporary-file installation, atomic replacement, backup/restore and schema migrations are later lifecycle milestones.

This feature is currently Linux-only. Cross-distribution installation and filesystem acceptance testing is still required before production use.
