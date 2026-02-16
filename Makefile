.PHONY: check
check:
	$(MAKE) -C runners/embedded check-all
	$(MAKE) -C runners/embedded check-nk3xn FEATURES=develop
	$(MAKE) -C runners/nkpk check
	$(MAKE) -C runners/usbip check

.PHONY: check-components
check-components:
	cargo check --manifest-path components/fm11nc08/Cargo.toml
	cargo check --manifest-path components/lfs-backup/Cargo.toml
	cargo check --manifest-path components/memory-regions/Cargo.toml
	cargo check --manifest-path components/ndef-app/Cargo.toml
	cargo check --manifest-path components/nfc-device/Cargo.toml
	cargo check --manifest-path components/provisioner-app/Cargo.toml

	cargo check --manifest-path components/apps/Cargo.toml
	for feature in nk3 nk3-test nk3-provisioner nkpk nkpk-provisioner ; do \
	echo "apps: $$feature" ; \
	cargo check --manifest-path components/apps/Cargo.toml --features $$feature ; \
	done
	cargo check --manifest-path components/apps/Cargo.toml --all-features

	cargo check --manifest-path components/boards/Cargo.toml
	for feature in board-nk3am board-nk3xn board-nkpk ; do \
	echo "boards: $$feature" ; \
	cargo check --manifest-path components/boards/Cargo.toml --features $$feature ; \
	done

	cargo check --manifest-path components/utils/Cargo.toml
	cargo check --manifest-path components/utils/Cargo.toml --all-features

.PHONY: doc
doc: 
	$(MAKE) -C runners/embedded doc-nk3am
	
.PHONY: lint
lint:
	cargo fmt -- --check
	$(MAKE) -C runners/embedded lint-all
	$(MAKE) -C runners/nkpk lint
	$(MAKE) -C runners/usbip lint

.PHONY: binaries
binaries:
	mkdir -p binaries
	$(MAKE) -C runners/embedded build-all FEATURES=
	cp runners/embedded/artifacts/runner-lpc55-nk3xn.bin binaries/firmware-nk3xn.bin
	cp runners/embedded/artifacts/runner-lpc55-nk3xn.elf binaries/firmware-nk3xn.elf
	cp runners/embedded/artifacts/runner-nrf52-bootloader-nk3am.bin.ihex binaries/firmware-nk3am.ihex
	$(MAKE) -C runners/embedded build-all FEATURES=provisioner
	cp runners/embedded/artifacts/runner-lpc55-nk3xn.bin binaries/provisioner-nk3xn.bin
	cp runners/embedded/artifacts/runner-lpc55-nk3xn.elf binaries/provisioner-nk3xn.elf
	cp runners/embedded/artifacts/runner-nrf52-bootloader-nk3am.bin.ihex binaries/provisioner-nk3am.ihex
	$(MAKE) -C runners/embedded build-all FEATURES=test
	cp runners/embedded/artifacts/runner-lpc55-nk3xn.bin binaries/firmware-nk3xn-test.bin
	cp runners/embedded/artifacts/runner-lpc55-nk3xn.elf binaries/firmware-nk3xn-test.elf
	cp runners/embedded/artifacts/runner-nrf52-bootloader-nk3am.bin.ihex binaries/firmware-nk3am-test.ihex
	$(MAKE) -C runners/nkpk build
	cp runners/nkpk/artifacts/runner-nkpk.elf binaries/firmware-nkpk.elf
	cp runners/nkpk/artifacts/runner-nkpk.bin.ihex binaries/firmware-nkpk.ihex
	$(MAKE) -C runners/nkpk build FEATURES=provisioner
	cp runners/nkpk/artifacts/runner-nkpk.elf binaries/provisioner-nkpk.elf
	cp runners/nkpk/artifacts/runner-nkpk.bin.ihex binaries/provisioner-nkpk.ihex

.PHONY: metrics
metrics: binaries
	repometrics generate > metrics.toml

license.txt:
	cargo run --release --manifest-path utils/collect-license-info/Cargo.toml -- runners/embedded/Cargo.toml "Nitrokey 3" > license.txt

commands.bd:
	cargo run --release --manifest-path utils/gen-commands-bd/Cargo.toml -- runners/embedded/Cargo.toml > $@

manifest.json:
	sed "s/@VERSION@/`git describe --always`/g" utils/manifest.template.json > manifest.json

.PHONY: software-tests
software-tests:
	cd components/apps && cargo test --all-features
	cd components/boards && cargo test
