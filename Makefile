.PHONY: smoke check test build fmt clippy control-audit clean demo help \
        js-install js-test js-build js-typecheck js-generate js-smoke js-pack

help:
	@echo "Prosopon — display server for AI agents"
	@echo ""
	@echo "Rust (cargo):"
	@echo "  make smoke         — check + clippy + test (pre-commit gate)"
	@echo "  make check         — cargo check --workspace --all-targets"
	@echo "  make clippy        — cargo clippy with warnings-as-errors"
	@echo "  make test          — cargo test --workspace"
	@echo "  make build         — cargo build --release"
	@echo "  make fmt           — cargo fmt --all"
	@echo "  make control-audit — smoke + fmt --check (pre-push gate)"
	@echo "  make demo          — render the in-repo demo scene"
	@echo "  make clean         — cargo clean"
	@echo ""
	@echo "JS (Bun + Turborepo — packages/prosopon-ts):"
	@echo "  make js-install    — bun install"
	@echo "  make js-smoke      — typecheck + test + build (pre-commit gate)"
	@echo "  make js-typecheck  — tsc --noEmit"
	@echo "  make js-test       — bun test"
	@echo "  make js-build      — tsc → dist/"
	@echo "  make js-generate   — regenerate TS types from committed schemas"
	@echo "  make js-pack       — npm pack --dry-run (preview tarball)"

smoke: check clippy test

check:
	cargo check --workspace --all-targets

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

build:
	cargo build --release

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

control-audit: smoke fmt-check
	@echo "control audit: all green"

demo:
	cargo run -p prosopon-cli --quiet -- demo

clean:
	cargo clean
	@rm -rf node_modules .turbo packages/*/node_modules packages/*/dist packages/*/.turbo packages/*/*.tsbuildinfo

# ---- JS / TS package tasks (packages/prosopon-ts) ----

js-install:
	bun install

js-typecheck:
	bun run --cwd packages/prosopon-ts typecheck

js-test:
	bun run --cwd packages/prosopon-ts test

js-build:
	bun run --cwd packages/prosopon-ts build

js-generate:
	bun run --cwd packages/prosopon-ts generate

js-smoke: js-typecheck js-test js-build

js-pack: js-build
	cd packages/prosopon-ts && npm pack --dry-run
