RUNNER ?= ../../runners/embedded/
OUTPUT_ELF ?= $(RUNNER)/artifacts/runner-stm32n6-nkso3.elf

.PHONY: build
build:
	$(MAKE) -C "$(RUNNER)" build-nkso3 FEATURES=$(FEATURES)

.PHONY: gdbserver
gdbserver:
	pyocd gdbserver --target stm32n657x0hxq --semihosting --persist -O semihost_console_type=console

.PHONY: run
run: build
	arm-none-eabi-gdb --quiet --command=run.gdb "$(OUTPUT_ELF)"

.PHONY: size
size: build
	arm-none-eabi-size "$(OUTPUT_ELF)"
