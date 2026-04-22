.PHONY: smoke check test build fmt clippy control-audit clean demo help

help:
	@echo "Prosopon — display server for AI agents"
	@echo ""
	@echo "  make smoke         — check + clippy + test (pre-commit gate)"
	@echo "  make check         — cargo check --workspace --all-targets"
	@echo "  make clippy        — cargo clippy with warnings-as-errors"
	@echo "  make test          — cargo test --workspace"
	@echo "  make build         — cargo build --release"
	@echo "  make fmt           — cargo fmt --all"
	@echo "  make control-audit — smoke + fmt --check (pre-push gate)"
	@echo "  make demo          — render the in-repo demo scene"
	@echo "  make clean         — cargo clean"

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
