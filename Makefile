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
	cargo run --release --manifest-path utils/collect-license-info/Cargo.toml -- runners/embedded/Cargo.toml "Nitrokey 3" > license.txt

commands.bd:
	cargo run --release --manifest-path utils/gen-commands-bd/Cargo.toml -- runners/embedded/Cargo.toml > $@

manifest.json:
	sed "s/@VERSION@/`git describe --always`/g" utils/manifest.template.json > manifest.json
