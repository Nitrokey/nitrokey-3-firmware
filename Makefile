.PHONY: check
check:
	$(MAKE) -C runners/embedded check-all FEATURES=nk3
	$(MAKE) -C runners/embedded check-all FEATURES=test
	$(MAKE) -C runners/embedded check-all FEATURES=provisioner
	$(MAKE) -C runners/nkpk check
	$(MAKE) -C runners/usbip check

.PHONY: doc
doc: 
	$(MAKE) -C runners/embedded doc-nk3am
	
.PHONY: lint
lint:
	cargo fmt -- --check
	$(MAKE) -C runners/embedded lint-all FEATURES=nk3
	$(MAKE) -C runners/embedded lint-all FEATURES=test
	$(MAKE) -C runners/embedded lint-all FEATURES=provisioner
	$(MAKE) -C runners/nkpk lint
	$(MAKE) -C runners/usbip lint

.PHONY: binaries
binaries:
	mkdir -p binaries
	$(MAKE) -C runners/embedded build-all FEATURES=nk3
	cp runners/embedded/artifacts/runner-lpc55-nk3xn.bin binaries/firmware-nk3xn.bin
	cp runners/embedded/artifacts/runner-lpc55-nk3xn.elf binaries/firmware-nk3xn.elf
	cp runners/embedded/artifacts/runner-nrf52-bootloader-nk3am.bin.ihex binaries/firmware-nk3am.ihex
	$(MAKE) -C runners/embedded build-all FEATURES=provisioner
	cp runners/embedded/artifacts/runner-lpc55-nk3xn.bin binaries/provisioner-nk3xn.bin
	cp runners/embedded/artifacts/runner-lpc55-nk3xn.elf binaries/provisioner-nk3xn.elf
	cp runners/embedded/artifacts/runner-nrf52-bootloader-nk3am.bin.ihex binaries/provisioner-nk3am.ihex
	$(MAKE) -C runners/embedded build-all FEATURES=nk3,test
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
