CARGO_TARGET_DIR ?= ../../target

EXTRA_FEATURES := ""
GDB ?= "arm-none-eabi-gdb"

.PHONY: gdbserver
gdbserver:
	pyocd gdbserver --target stm32n657x0hxq --semihosting --persist -O semihost_console_type=console

.PHONY: build
build:
	cargo build --release --features $(EXTRA_FEATURES)

.PHONY: run
run: build
	${GDB} --quiet --command=run.gdb ${CARGO_TARGET_DIR}/thumbv8m.main-none-eabi/release/nkso3
