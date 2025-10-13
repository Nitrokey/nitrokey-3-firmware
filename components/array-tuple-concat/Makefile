# Copyright (C) 2023 Nitrokey GmbH
# SPDX-License-Identifier: CC0-1.0

.PHONY: check lint test ci miri

check:
	cargo check

test:
	cargo test

miri:
	cargo +nightly miri test

lint:
	reuse lint
	cargo fmt --check
	cargo clippy --all-targets

ci: lint check test miri

