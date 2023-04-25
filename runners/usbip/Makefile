.PHONY: all
all: check lint

.PHONY: check
check:
	cargo check
	cargo check --features test
	cargo check --features provisioner

.PHONY: lint
lint:
	cargo clippy --no-deps
	cargo fmt -- --check
