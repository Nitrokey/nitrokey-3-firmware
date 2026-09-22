RUNNER ?= ../../runners/embedded/
OUTPUT_ELF ?= $(RUNNER)/artifacts/runner-stm32n6-nkso3.elf

.PHONY: build
build:
	$(MAKE) -C "$(RUNNER)" build-nkso3 FEATURES=$(FEATURES)

PYOCD_TARGET ?= stm32n657l0hxq
PYOCD_OPTS ?= -O reset_type=sw_sysresetreq

.PHONY: gdbserver
gdbserver:
	pyocd gdbserver --target $(PYOCD_TARGET) $(PYOCD_OPTS) --semihosting --persist -O semihost_console_type=console

.PHONY: run
run: build
	arm-none-eabi-gdb --quiet --command=run.gdb "$(OUTPUT_ELF)"

.PHONY: size
size: build
	arm-none-eabi-size "$(OUTPUT_ELF)"
