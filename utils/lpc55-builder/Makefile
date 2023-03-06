# TODO: remove hardcoded filesystem boundary in config/flash-erase-firmware.toml

.NOTPARALLEL:

RUNNER ?= ../../runners/embedded/
OUTPUT_BIN ?= $(RUNNER)/artifacts/runner-lpc55-nk3xn.bin
OUTPUT_ELF ?= $(RUNNER)/artifacts/runner-lpc55-nk3xn.elf

.PHONY: build
build:
	$(MAKE) -C "$(RUNNER)" build-nk3xn

.PHONY: jlink
jlink:
	JLinkGDBServer -strict -device LPC55S69 -if SWD -vd

.PHONY: run
run: build
	arm-none-eabi-gdb -q -x ./jlink.gdb "$(OUTPUT_ELF)"

.PHONY: provision-develop
provision-develop:
	# Step 1: build & flash provisioner, configure device
	$(MAKE) build FEATURES=develop,provisioner,$(FEATUERS)
	./scripts/boot-to-bootrom.sh
	./scripts/usbwait.sh 1fc9:0021 20a0:42dd
	$(MAKE) bl-config-cmpa-develop
	$(MAKE) bl-erase-firmware
	$(MAKE) bl-flash
	lpc55 reboot
	./scripts/usbwait.sh 20a0:42b2
	# Step 2: provision certs
	$(MAKE) fw-provision-certs
	./scripts/boot-to-bootrom.sh
	./scripts/usbwait.sh 20a0:42dd
	# Step 3: build & flash final firmware
	$(MAKE) build FEATURES=develop,$(FEATURES)
	$(MAKE) bl-erase-firmware
	$(MAKE) bl-flash
	lpc55 reboot
	./scripts/usbwait.sh 20a0:42b2

.PHONY: provision-release
provision-release:
	# Step 0: reset
	./scripts/boot-to-bootrom.sh
	./scripts/usbwait.sh 1fc9:0021 20a0:42dd
	$(MAKE) bl-config-cmpa-empty
	$(MAKE) bl-erase-firmware
	lpc55 reboot
	./scripts/usbwait.sh 1fc9:0021
	# Step 1: build & flash provisioner, configure CFPA, provision keystore
	$(MAKE) build FEATURES=provisioner,$(FEATUERS)
	$(MAKE) bl-config-cfpa
	$(MAKE) bl-provision-keystore
	$(MAKE) bl-flash
	lpc55 reboot
	./scripts/usbwait.sh 20a0:42b2
	# Step 2: provision certs
	$(MAKE) fw-provision-certs
	./scripts/boot-to-bootrom.sh
	./scripts/usbwait.sh 1fc9:0021
	# Step 3: Erase provisioner, configure CMPA
	$(MAKE) bl-erase-firmware
	$(MAKE) bl-config-cmpa-release
	lpc55 reboot
	./scripts/usbwait.sh 20a0:42dd
	# Step 4: build, sign & flash final firmware
	$(MAKE) sign FEATURES=$(FEATURES)
	$(MAKE) bl-receive-sb2
	lpc55 reboot
	./scripts/usbwait.sh 20a0:42b2

.PHONY: sign
sign: build
	$(MAKE) -C ../.. commands.bd
	cp "$(OUTPUT_BIN)" firmware-nk3xn.bin
	nxpimage mbi export config/sign.yaml
	elftosb -f lpc55xx \
		-k "config/sbkek.txt" -c ../../commands.bd -o firmware-nk3xn.sb2 \
		-s ./certs/nk-firmware-ee1-key.pem \
		-S ./certs/nk-firmware-root-cert.der \
		-S ./certs/nk-firmware-ee1-cert.der \
		-R ./certs/nk-firmware-root-cert.der \
		-R ./certs/nk-firmware-ee2-cert.der \
		-R ./certs/nk-firmware-ee3-cert.der \
		-R ./certs/nk-firmware-ee4-cert.der \
		-h rkht.bin firmware-nk3xn-signed.bin
	nitropy nk3 validate-update --variant lpc55 firmware-nk3xn.sb2

.PHONY: flash
flash: build
	./scripts/boot-to-bootrom.sh
	./scripts/usbwait.sh 1fc9:0021 20a0:42dd
	$(MAKE) bl-erase-firmware
	$(MAKE) bl-flash
	lpc55 reboot

.PHONY: reset
reset:
	./scripts/boot-to-bootrom.sh
	./scripts/usbwait.sh 1fc9:0021 20a0:42dd
	$(MAKE) bl-erase-all
	$(MAKE) bl-config-cmpa-empty
	lpc55 reboot

# The following targets prefixed with bl- require that the device is in bootloader mode.

.PHONY: bl-flash
bl-flash:
	lpc55 write-flash "$(OUTPUT_BIN)"

.PHONY: bl-erase-all
bl-erase-all:
	lpc55 provision ./config/flash-erase-all.toml

.PHONY: bl-erase-firmware
bl-erase-firmware:
	lpc55 provision ./config/flash-erase-firmware.toml

.PHONY: bl-receive-sb2
bl-receive-sb2:
	lpc55 receive-sb-file firmware-nk3xn.sb2

.PHONY: bl-config-cmpa-empty
bl-config-cmpa-empty:
	lpc55 configure factory-settings ./config/cmpa-empty.toml

.PHONY: bl-config-cmpa-develop
bl-config-cmpa-develop:
	lpc55 configure factory-settings ./config/cmpa-develop.toml

.PHONY: bl-config-cmpa-release
bl-config-cmpa-release:
	lpc55 configure factory-settings ./config/cmpa-release.toml

.PHONY: bl-config-cfpa
bl-config-cfpa:
	lpc55 configure customer-settings ./config/cfpa.toml

.PHONY: bl-config-cmpa
bl-config-cmpa:
	lpc55 configure factory-settings ./config/cmpa.toml

.PHONY: bl-provision-keystore
bl-provision-keystore:
	lpc55 provision ./config/keystore.toml

# The following targets prefixed with fw- require that the device is in firmware mode.

.PHONY: fw-provision-certs
fw-provision-certs: data/fido.key data/fido.cert
	# TODO: add Trussed key & cert
	solo2 app provision store-fido-batch-cert data/fido.cert
	solo2 app provision store-fido-batch-key data/fido.key

data/fido.key: data/fido.cert

data/fido.cert:
	mkdir -p data
	solo2 pki dev fido data/fido.key data/fido.cert
