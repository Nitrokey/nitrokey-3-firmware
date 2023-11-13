RUNNER ?= ../../runners/embedded
RUNNER_NK3AM ?= $(RUNNER)/artifacts/runner-nrf52-bootloader-nk3am.bin
RUNNER_NK3XN ?= $(RUNNER)/artifacts/runner-lpc55-nk3xn.bin

.PHONY: all
all:
	@nix-shell --pure --run "make binaries"
	@ls -l *.bin

.PHONY: binaries
binaries:
	$(MAKE) -C $(RUNNER) build-nk3xn
	cp $(RUNNER_NK3XN) firmware-nk3xn.bin
	$(MAKE) -C $(RUNNER) build-nk3xn FEATURES=provisioner
	cp $(RUNNER_NK3XN) provisioner-nk3xn.bin
	$(MAKE) -C $(RUNNER) build-nk3xn FEATURES=test
	cp $(RUNNER_NK3XN) firmware-test-nk3xn.bin
	$(MAKE) -C $(RUNNER) build-nk3am.bl
	cp $(RUNNER_NK3AM) firmware-nk3am.bin
	$(MAKE) -C $(RUNNER) build-nk3am.bl FEATURES=provisioner
	cp $(RUNNER_NK3AM) provisioner-nk3am.bin
	$(MAKE) -C $(RUNNER) build-nk3am.bl FEATURES=test
	cp $(RUNNER_NK3AM) firmware-test-nk3am.bin
