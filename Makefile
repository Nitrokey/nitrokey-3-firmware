.PHONY: check
check:
	$(MAKE) -C runners/embedded check-all
	$(MAKE) -C runners/usbip check

.PHONY: doc
doc: 
	$(MAKE) -C runners/embedded doc-nk3am
	
.PHONY: lint
lint:
	cargo fmt -- --check

license.txt:
	cargo run --release --manifest-path utils/collect-license-info/Cargo.toml -- runners/embedded/Cargo.toml > license.txt

commands.bd:
	cargo run --release --manifest-path utils/gen-commands-bd/Cargo.toml -- \
		runners/embedded/Cargo.toml \
		runners/embedded/profiles/lpc55.toml \
		> $@

manifest.json:
	sed "s/@VERSION@/`git describe --always`/g" utils/manifest.template.json > manifest.json
