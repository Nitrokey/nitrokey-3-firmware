CARGO_TARGET_DIR ?= ../../target

.PHONY: gdbserver
gdbserver:
	pyocd gdbserver --target stm32n657x0hxq --semihosting --persist -O semihost_console_type=console

.PHONY: run
run:
	cargo build --release && arm-none-eabi-gdb --quiet --command=run.gdb ${CARGO_TARGET_DIR}/thumbv8m.main-none-eabi/release/nkso3
