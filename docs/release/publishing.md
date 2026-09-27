# Publishing Prosopon

The prosopon repo is a **bilingual monorepo**: it ships Rust crates to crates.io
and TypeScript packages to npm. This runbook documents the release procedure
for both sides.

## Contents of the repo

```
prosopon/
├── Cargo.toml                ← Rust workspace root
├── package.json              ← Turborepo + Bun workspace root
├── turbo.json
├── Makefile                  ← unified entry points (cargo + bun)
├── crates/
│   ├── prosopon-core/        ← crates.io: prosopon-core
│   ├── prosopon-protocol/    ← crates.io: prosopon-protocol
│   ├── prosopon-runtime/
│   ├── prosopon-compositor-text/
│   ├── prosopon-compositor-glass/
│   ├── prosopon-daemon/
│   ├── prosopon-sdk/
│   ├── prosopon-cli/
│   └── prosopon-pneuma/
└── packages/
    └── prosopon-ts/          ← npm: @broomva/prosopon
```

## Versioning policy

- **Rust crates** follow individual `version` in their `Cargo.toml`, aligned
  with `workspace.package.version` for most internal crates. Cross-crate
  compat is SemVer within the 0.x.x prefix.
- **`@broomva/prosopon`** tracks `IR_SCHEMA_VERSION` from
  `crates/prosopon-core/src/lib.rs`. Minor bump when the schema grows a
  non-breaking variant (e.g. RFC-0004 → `0.2.0`). Patch bumps for TS-only
  fixes (client reconnect logic, typing fixes, docs) — append `-patch.N` if
  the IR schema hasn't moved but you still need to ship a TS-side fix.
- **`PROTOCOL_VERSION`** in `prosopon-protocol` bumps on wire-format changes
  only; these are rare and coordinated across both Rust and TS in one
  release.

## TS package release (`@broomva/prosopon`)

### First publish (one-time bootstrap)

1. Authenticate once:

   ```bash
   npm login
   # verify the `@broomva` scope is on your account
   npm whoami
   ```

2. From the repo root:

   ```bash
   make js-smoke     # typecheck + test + build (gate)
   make js-pack      # preview the tarball — dist/ + src/ + README + LICENSE
   ```

3. Publish:

   ```bash
   cd packages/prosopon-ts
   npm publish
   # → runs `prepublishOnly`: clean → build → test
   # → uploads to npm with `--access public` (from publishConfig)
   ```

4. Verify the artifact: <https://www.npmjs.com/package/@broomva/prosopon>

### Subsequent releases

1. Bump `version` in `packages/prosopon-ts/package.json`.
2. Update the compatibility table in `packages/prosopon-ts/README.md` +
   the `## Unreleased` section in this repo's `CHANGELOG.md`.
3. `make js-smoke && make js-pack` to sanity-check.
4. `npm publish` from `packages/prosopon-ts/`.
5. Tag the release:

   ```bash
   git tag prosopon-ts/v$(jq -r .version packages/prosopon-ts/package.json)
   git push --tags
   ```

## Rust crate releases (crates.io)

Individual crates publish in dependency order. For a coordinated release:

```bash
# From the repo root — pre-flight
make control-audit       # smoke + fmt check

# Publish in dependency order (each `cargo publish` verifies build)
cargo publish -p prosopon-core
cargo publish -p prosopon-protocol
cargo publish -p prosopon-runtime
cargo publish -p prosopon-compositor-text
cargo publish -p prosopon-compositor-glass
cargo publish -p prosopon-sdk
cargo publish -p prosopon-pneuma
cargo publish -p prosopon-cli
cargo publish -p prosopon-daemon
```

### Bumping versions

Use `cargo release` (or manually edit each `Cargo.toml`) to move workspace
versions together. Keep `workspace.dependencies` entries in sync:

```toml
# Cargo.toml (workspace root)
prosopon-core = { path = "crates/prosopon-core", version = "0.2.0" }
```

## Coordinated release (Rust + TS)

When a release changes the IR (e.g. a new `Intent` variant via an RFC):

1. Land the Rust change (variant, tests, compositor, SDK helper, version bump
   of `IR_SCHEMA_VERSION`).
2. Regenerate + commit `packages/prosopon-ts/src/generated/` in the same PR
   (or a follow-up).
3. Bump `@broomva/prosopon` to match the new `IR_SCHEMA_VERSION`.
4. Release the Rust crates first (so `crates.io` has the matching
   `prosopon-core` version).
5. Release `@broomva/prosopon` to npm.
6. Downstream consumers (e.g. `broomva.tech`) bump their dependency after
   step 5 lands.

## Troubleshooting

| Symptom | Fix |
|---|---|
| `npm publish` fails with 403 | Check `npm whoami` + confirm `@broomva` scope membership |
| `prepublishOnly` fails at build | Run `make js-smoke` locally, fix errors, retry |
| CI `schema-parity` job fails | Regenerate snapshots: `make js-generate` + commit |
| `cargo publish` "crate already exists" | Bump version in the crate's `Cargo.toml` and workspace deps |
